//! Ergo node REST API endpoints

use bounded_integer::BoundedU16;
use bounded_vec::NonEmptyVec;
use ergo_chain_types::BlockId;
use ergo_chain_types::PeerAddr;
use ergo_nipopow::NipopowProof;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::RequestBuilder;
use std::collections::HashSet;
use std::time::Duration;
use url::Url;

use crate::NodeConf;
use crate::NodeError;
use crate::NodeInfo;
use crate::PeerInfo;
use thiserror::Error;

fn set_req_headers(rb: RequestBuilder, node: NodeConf) -> RequestBuilder {
    rb.header("accept", "application/json")
        .header("api_key", node.get_node_api_header())
        .header(CONTENT_TYPE, "application/json")
}

fn build_client(node_conf: &NodeConf) -> Result<Client, reqwest::Error> {
    let builder = reqwest::Client::builder();
    if let Some(t) = node_conf.timeout {
        builder.timeout(t).build()
    } else {
        builder.build()
    }
}

/// GET on /info endpoint
pub async fn get_info(node: NodeConf) -> Result<NodeInfo, NodeError> {
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join("info").unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    Ok(set_req_headers(rb, node)
        .send()
        .await?
        .json::<NodeInfo>()
        .await?)
}

/// GET on /peers/all endpoint
async fn get_peers_all(node: NodeConf) -> Result<Vec<PeerInfo>, NodeError> {
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join("peers/all").unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    let response = set_req_headers(rb, node).send().await?;
    Ok(response.json::<Vec<PeerInfo>>().await?)
}

#[cfg(not(target_arch = "wasm32"))]
/// Given a list of seed nodes, recursively determine a Vec of all known peer nodes. Note that there
/// are `println` calls below that are commented out. Uncommenting them will show the requests that
/// are made, especially when made in parallel.
pub async fn peer_discovery(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    use tokio::sync::mpsc::error::TrySendError;

    use futures::{StreamExt, TryStreamExt};
    use tokio::sync::mpsc;
    let mut seeds_set: HashSet<Url> = HashSet::new();

    let buffer_size = usize::max(max_parallel_requests.get() as usize, seeds.len());
    for mut seed_url in seeds {
        #[allow(clippy::unwrap_used)]
        seed_url.set_port(None).unwrap();
        seeds_set.insert(seed_url);
    }

    let (tx_url, rx_url) = mpsc::channel::<Url>(buffer_size);

    enum Msg {
        /// Indicates that the ergo node at the given URL is active. This means that a GET request
        /// to the node's /info endpoint responds with code 200 OK.
        AddActiveNode(Url),
        /// Indicates that the ergo node at the given URL is inactive. This means that a GET request
        /// to the node's /info endpoint does not respond with code 200 OK.
        AddInactiveNode(Url),
        /// A list of peers of an active ergo node, returned from a GET on the /peers/all endpoint.
        CheckPeers(Vec<PeerInfo>),
    }

    let (tx_peer, mut rx_peer) = mpsc::channel::<Msg>(buffer_size);

    // For every URL received, spawn a task which checks if the corresponding node is active. If so,
    // request peers. In all cases, a message is sent out (enum `Msg` above) to filter out future
    // redundant URL requests.
    let rx_url_stream = tokio_stream::wrappers::ReceiverStream::new(rx_url)
        .map(move |mut url| {
            let tx_peer = tx_peer.clone();
            async move {
                let handle = tokio::spawn(async move {
                    // Query node at url.
                    #[allow(clippy::unwrap_used)]
                    url.set_port(Some(9053)).unwrap();
                    //println!("Processing {}", url);
                    #[allow(clippy::unwrap_used)]
                    let node_conf = NodeConf {
                        addr: PeerAddr(url.socket_addrs(|| Some(9053)).unwrap()[0]),
                        api_key: None,
                        timeout: Some(timeout),
                    };

                    // If active, look up its peers.
                    if get_info(node_conf).await.is_ok() {
                        match get_peers_all(node_conf).await {
                            Ok(peers) => {
                                // It's important to send this message before the `AddActiveNode` message
                                // below, to ensure an `count` variable; see (**) below.
                                tx_peer.send(Msg::CheckPeers(peers)).await?;
                                tx_peer.send(Msg::AddActiveNode(url.clone())).await?;
                            }
                            Err(_) => {
                                tx_peer.send(Msg::AddInactiveNode(url)).await?;
                            }
                        }
                    } else {
                        tx_peer.send(Msg::AddInactiveNode(url)).await?;
                    }
                    Result::<(), mpsc::error::SendError<Msg>>::Ok(())
                });

                handle.await.map_err(|_| PeerDiscoveryError::MpscSender)
            }
        })
        .buffer_unordered(max_parallel_requests.get() as usize); // Allow for parallel requests

    // (*) Run stream to completion.
    let e = tokio::spawn(rx_url_stream.try_for_each(
        |x: Result<(), mpsc::error::SendError<Msg>>| async move {
            match x {
                Ok(()) => Ok(()),
                Err(_) => Err(PeerDiscoveryError::MpscSender),
            }
        },
    ));

    for url in &seeds_set {
        tx_url
            .send(url.clone())
            .await
            .map_err(|_| PeerDiscoveryError::MpscSender)?;
    }

    // (**) This variable represents the number of URLs that need to be checked to see whether it
    // corresponds to an active Ergo node. `count` is crucial to allow this function to terminate,
    // as once it reaches zero we break the loop below. This leads us to drop `tx_url`, which is the
    // sender side of the receiver stream `rx_url_stream`, allowing the spawned task (*) to end.
    let mut count = seeds_set.len();

    let mut visited_active_peers = HashSet::new();
    let mut visited_peers = HashSet::new();

    // Stack of peers to evaluate. Used as a growable buffer for when the (tx_url, rx_url) channel
    // gets full.
    let mut peer_stack: Vec<PeerInfo> = vec![];

    'loop_: while let Some(p) = rx_peer.recv().await {
        // Try pushing as many peers as can be allowed in the (tx_url, rx_url) channel
        while let Some(peer) = peer_stack.pop() {
            let mut url = peer.addr.as_http_url();
            #[allow(clippy::unwrap_used)]
            url.set_port(None).unwrap();
            if !visited_peers.contains(&url) {
                match tx_url.try_send(url.clone()) {
                    Ok(_) => {
                        visited_peers.insert(url);
                        count += 1;
                    }
                    Err(TrySendError::Full(_)) => {
                        // Push it back on the stack, try again later.
                        peer_stack.push(peer);
                        break;
                    }
                    Err(TrySendError::Closed(_)) => {
                        return Err(PeerDiscoveryError::MpscSender);
                    }
                }
            }
        }
        match p {
            Msg::AddActiveNode(mut url) => {
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                //println!("Active node {}", url);
                visited_active_peers.insert(url.clone());
                visited_peers.insert(url);
                count -= 1;
                if count == 0 {
                    break 'loop_;
                }
            }
            Msg::AddInactiveNode(mut url) => {
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                visited_peers.insert(url);
                count -= 1;
                if count == 0 {
                    break 'loop_;
                }
            }
            Msg::CheckPeers(peers) => {
                peer_stack.extend(peers);
            }
        }
    }

    drop(tx_url);
    match e.await {
        Ok(Ok(())) => {
            let coll: Vec<_> = visited_active_peers
                .difference(&seeds_set)
                .into_iter()
                .cloned()
                .collect();
            Ok(coll)
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(PeerDiscoveryError::JoinError),
    }
}

#[cfg(target_arch = "wasm32")]
/// Given a list of seed nodes, recursively determine a Vec of all known peer nodes.
///
/// This version is essentially the same as the tokio-based, non-WASM version above except that
/// the `spawn_local` function can only spawn tasks for futures that return `()`. This means that
/// we can't do the usual error handling. We can handle some errors by sending out occurrences
/// through a spawned channel.
pub async fn peer_discovery(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    use futures::channel::mpsc;
    use futures::{SinkExt, StreamExt};
    use wasm_bindgen_futures::spawn_local;

    let mut seeds_set: HashSet<Url> = HashSet::new();

    for mut seed_url in seeds {
        #[allow(clippy::unwrap_used)]
        seed_url.set_port(None).unwrap();
        seeds_set.insert(seed_url);
    }

    let (mut tx_url, rx_url) = mpsc::channel::<Url>(50);

    enum Msg {
        /// Indicates that the ergo node at the given URL is active. This means that a GET request
        /// to the node's /info endpoint responds with code 200 OK.
        AddActiveNode(Url),
        /// Indicates that the ergo node at the given URL is inactive. This means that a GET request
        /// to the node's /info endpoint does not respond with code 200 OK.
        AddInactiveNode(Url),
        /// A list of peers of an active ergo node, returned from a GET on the /peers/all endpoint.
        CheckPeers(Vec<PeerInfo>),
        /// Error that can arise in the `rx_url_stream` future.
        StreamError(PeerDiscoveryError),
    }

    let (tx_peer, mut rx_peer) = mpsc::channel::<Msg>(50);
    // For every URL received, spawn a task which checks if the corresponding node is active. If so,
    // request peers. In all cases, a message is sent out (enum `Msg` above) to filter out future
    // redundant URL requests.
    let rx_url_stream = rx_url
        .map(move |mut url| {
            let mut tx_peer = tx_peer.clone();
            async move {
                spawn_local(async move {
                    // NOTE: we use `tx_peer` to notify the receiver @ `'loop` below of some errors.
                    // However all `tx_peer.send(..)` calls below return a `Result<..>` that we
                    // cannot explicitly deal with due to `spawn_local` being only able to spawn
                    // futures with a `()` return type, hence the unwrap calls. It shouldn't be a
                    // problem though since the channel only stays within `peer_discovery` and
                    // everything is single-threaded here.

                    // Query node at url.
                    #[allow(clippy::unwrap_used)]
                    url.set_port(Some(9053)).unwrap();
                    let addr = if let Ok(addresses) = url.socket_addrs(|| Some(9053)) {
                        if !addresses.is_empty() {
                            PeerAddr(addresses[0])
                        } else {
                            #[allow(clippy::unwrap_used)]
                            tx_peer
                                .send(Msg::StreamError(PeerDiscoveryError::UrlError))
                                .await
                                .unwrap();
                            return;
                        }
                    } else {
                        tx_peer
                            .send(Msg::StreamError(PeerDiscoveryError::UrlError))
                            .await
                            .unwrap();
                        return;
                    };
                    let node_conf = NodeConf {
                        addr,
                        api_key: None,
                        timeout: Some(timeout),
                    };

                    // If active, look up its peers.
                    if get_info(node_conf).await.is_ok() {
                        match get_peers_all(node_conf).await {
                            Ok(peers) => {
                                // It's important to send this message before the `AddActiveNode` message
                                // below, to ensure an `count` variable; see (**) below.
                                #[allow(clippy::unwrap_used)]
                                tx_peer.send(Msg::CheckPeers(peers)).await.unwrap();
                                #[allow(clippy::unwrap_used)]
                                tx_peer.send(Msg::AddActiveNode(url.clone())).await.unwrap();
                            }
                            Err(_) => {
                                #[allow(clippy::unwrap_used)]
                                tx_peer.send(Msg::AddInactiveNode(url)).await.unwrap();
                            }
                        }
                    } else {
                        #[allow(clippy::unwrap_used)]
                        tx_peer.send(Msg::AddInactiveNode(url)).await.unwrap();
                    }
                });
            }
        })
        .buffer_unordered(max_parallel_requests.get() as usize);

    // (*) Run stream to completion.
    spawn_local(rx_url_stream.for_each(|_| async {}));

    for url in &seeds_set {
        tx_url
            .send(url.clone())
            .await
            .map_err(|_| PeerDiscoveryError::MpscSender)?;
    }

    // (**) This variable represents the number of URLs that need to be checked to see whether it
    // corresponds to an active Ergo node. `count` is crucial to allow this function to terminate,
    // as once it reaches zero we break the loop below. This leads us to drop `tx_url`, which is the
    // sender side of the receiver stream `rx_url_stream`, allowing the spawned task (*) to end.
    let mut count = seeds_set.len();

    let mut visited_active_peers = HashSet::new();
    let mut visited_peers = HashSet::new();

    // Stack of peers to evaluate. Used as a growable buffer for when the (tx_url, rx_url) channel
    // gets full.
    let mut peer_stack: Vec<PeerInfo> = vec![];

    'loop_: while let Some(p) = rx_peer.next().await {
        // Try pushing as many peers as can be allowed in the (tx_url, rx_url) channel
        loop {
            if let Some(peer) = peer_stack.pop() {
                let mut url = peer.addr.as_http_url();
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                if !visited_peers.contains(&url) {
                    match tx_url.try_send(url.clone()) {
                        Ok(_) => {
                            visited_peers.insert(url);
                            count += 1;
                        }
                        Err(e) => {
                            if e.is_full() {
                                // Push it back on the stack, try again later.
                                peer_stack.push(peer);
                                break;
                            } else if e.is_disconnected() {
                                return Err(PeerDiscoveryError::MpscSender);
                            } else {
                                return Err(PeerDiscoveryError::MpscSenderOther);
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }
        match p {
            Msg::AddActiveNode(mut url) => {
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                visited_active_peers.insert(url.clone());
                visited_peers.insert(url);
                count -= 1;
                if count == 0 {
                    break 'loop_;
                }
            }
            Msg::AddInactiveNode(mut url) => {
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                visited_peers.insert(url);
                count -= 1;
                if count == 0 {
                    break 'loop_;
                }
            }
            Msg::CheckPeers(peers) => {
                peer_stack.extend(peers);
            }
            Msg::StreamError(e) => {
                return Err(e);
            }
        }
    }

    drop(tx_url);

    Ok(visited_active_peers
        .difference(&seeds_set)
        .into_iter()
        .cloned()
        .collect())
}

#[derive(Debug, Error)]
/// Peer discovery error
pub enum PeerDiscoveryError {
    /// `Url` error
    #[error("URL error")]
    UrlError,
    /// mpsc sender error
    #[error("MPSC sender error")]
    MpscSender,
    /// other mpsc sender error
    #[error("Other MPSC sender error")]
    MpscSenderOther,
    /// tokio::spawn `JoinError`
    #[error("Join error")]
    JoinError,
    /// task spawn error
    #[error("Task spawn error")]
    TaskSpawn,
}

/// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
pub async fn get_nipopow_proof_by_header_id(
    node: NodeConf,
    min_chain_length: u32,
    suffix_len: u32,
    header_id: BlockId,
) -> Result<NipopowProof, NodeError> {
    if min_chain_length == 0 || suffix_len == 0 {
        return Err(NodeError::InvalidNumericalUrlSegment);
    }
    let header_str = String::from(header_id.0);
    let mut path = "nipopow/proof/".to_owned();
    path.push_str(&*min_chain_length.to_string());
    path.push('/');
    path.push_str(&*suffix_len.to_string());
    path.push('/');
    path.push_str(&*header_str);
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join(&*path).unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    Ok(set_req_headers(rb, node)
        .send()
        .await?
        .json::<NipopowProof>()
        .await?)
}

// pub async fn get_blocks_header_id_proof_for_tx_id(
//     _node: NodeConf,
//     _header_id: BlockId,
//     _tx_id: TxId,
// ) -> Result<Option<MerkleProof>, NodeError> {
//     todo!()
// }

#[allow(clippy::unwrap_used)]
#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::str::FromStr;
    use std::time::Duration;

    use ergo_chain_types::PeerAddr;

    use super::*;

    #[test]
    fn test_get_info() {
        let runtime_inner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let node_conf = NodeConf {
            addr: PeerAddr::from_str("213.239.193.208:9053").unwrap(),
            api_key: None,
            timeout: Some(Duration::from_secs(5)),
        };
        let res = runtime_inner.block_on(async { get_info(node_conf).await.unwrap() });
        assert_ne!(res.name, "");
    }

    #[test]
    fn test_get_peers_all() {
        let runtime_inner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let node_conf = NodeConf {
            addr: PeerAddr::from_str("213.239.193.208:9053").unwrap(),
            api_key: None,
            timeout: Some(Duration::from_secs(5)),
        };
        let res = runtime_inner.block_on(async { get_peers_all(node_conf).await.unwrap() });
        assert!(!res.is_empty())
    }

    #[test]
    fn test_get_nipopow_proof_by_header_id() {
        use ergo_chain_types::{BlockId, Digest32};
        let header_id = BlockId(
            Digest32::try_from(String::from(
                "9bcb535c2d05fbced6de3d73c63337d6deb64af387438fa748d66ddf3d33ee89",
            ))
            .unwrap(),
        );
        let runtime_inner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let node_conf = NodeConf {
            addr: PeerAddr::from_str("213.239.193.208:9053").unwrap(),
            api_key: None,
            timeout: Some(Duration::from_secs(5)),
        };
        let m = 3;
        let k = 4;
        let res = runtime_inner.block_on(async {
            get_nipopow_proof_by_header_id(node_conf, m, k, header_id)
                .await
                .unwrap()
        });
        assert!(!res.prefix.is_empty());
        assert_eq!(res.m, m);
        assert_eq!(res.k, k);
    }

    #[test]
    fn test_peer_discovery() {
        let seeds: Vec<_> = [
            "http://213.239.193.208:9030",
            "http://159.65.11.55:9030",
            "http://165.227.26.175:9030",
            "http://159.89.116.15:9030",
            "http://136.244.110.145:9030",
            "http://94.130.108.35:9030",
            "http://51.75.147.1:9020",
            "http://221.165.214.185:9030",
            "http://51.81.185.231:9031",
            "http://217.182.197.196:9030",
            "http://62.171.190.193:9030",
            "http://173.212.220.9:9030",
            "http://176.9.65.58:9130",
            "http://213.152.106.56:9030",
        ]
        .iter()
        .map(|s| Url::from_str(s).unwrap())
        .collect();
        let runtime_inner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let res = runtime_inner.block_on(async {
            peer_discovery(
                NonEmptyVec::from_vec(seeds).unwrap(),
                BoundedU16::new(30).unwrap(),
                Duration::from_millis(2000),
            )
            .await
            .unwrap()
        });
        assert!(!res.is_empty())
    }
}
