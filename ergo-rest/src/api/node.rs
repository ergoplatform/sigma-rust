//! Ergo node REST API endpoints

use bounded_integer::BoundedU16;
use bounded_vec::BoundedVec;
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
    Ok(set_req_headers(rb, node)
        .send()
        .await?
        .json::<Vec<PeerInfo>>()
        .await?)
}

#[cfg(target_arch = "wasm32")]
/// Given a list of seed nodes, recursively determine a Vec of all known peer nodes.
pub async fn peer_discovery(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    use futures::channel::mpsc;
    use futures::SinkExt;
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
    }

    let (tx_peer, mut rx_peer) = mpsc::channel::<Msg>(50);

    // For every URL received, spawn a task which checks if the corresponding node is active. If so,
    // request peers. In all cases, a message is sent out (enum `Msg` above) to filter out future
    // redundant URL requests.
    let rx_url_stream = rx_url
        .map(move |mut url| {
            let mut tx_peer = tx_peer.clone();
            async move {
                println!("Processing {}", url);
                spawn_local(async move {
                    // Query node at url.
                    #[allow(clippy::unwrap_used)]
                    url.set_port(Some(9053)).unwrap();
                    #[allow(clippy::unwrap_used)]
                    let node_conf = NodeConf {
                        addr: PeerAddr(url.socket_addrs(|| Some(9053)).unwrap()[0]),
                        api_key: None,
                        timeout: Some(timeout),
                    };

                    // If active, look up its peers.
                    if get_info(node_conf).await.is_ok() {
                        println!("active nodeConf: {:?}", node_conf);
                        if let Ok(peers) = get_peers_all(node_conf).await {
                            // It's important to send this message before the `AddActiveNode` message
                            // below, to ensure an `count` variable; see (**) below.
                            tx_peer.send(Msg::CheckPeers(peers)).await.unwrap();
                        }

                        tx_peer.send(Msg::AddActiveNode(url.clone())).await.unwrap();
                    } else {
                        tx_peer.send(Msg::AddInactiveNode(url)).await.unwrap();
                    }
                });
            }
        })
        .buffer_unordered(max_parallel_requests.get() as usize); // Allow for parallel requests

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

    'loop_: while let Some(p) = rx_peer.next().await {
        match p {
            Msg::AddActiveNode(mut url) => {
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                println!("added {}", url);
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
                for peer in peers {
                    let mut url = peer.addr.as_http_url();
                    #[allow(clippy::unwrap_used)]
                    url.set_port(None).unwrap();
                    if !visited_peers.contains(&url) {
                        let _ = tx_url.send(url.clone()).await;
                        visited_peers.insert(url);
                        count += 1;
                    }
                }
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

#[cfg(not(target_arch = "wasm32"))]
/// Given a list of seed nodes, recursively determine a Vec of all known peer nodes.
pub async fn peer_discovery(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    use futures::{StreamExt, TryStreamExt};
    use tokio::sync::mpsc;
    let mut seeds_set: HashSet<Url> = HashSet::new();

    for mut seed_url in seeds {
        #[allow(clippy::unwrap_used)]
        seed_url.set_port(None).unwrap();
        seeds_set.insert(seed_url);
    }

    let (tx_url, rx_url) = mpsc::channel::<Url>(50);

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

    let (tx_peer, mut rx_peer) = mpsc::channel::<Msg>(50);

    // For every URL received, spawn a task which checks if the corresponding node is active. If so,
    // request peers. In all cases, a message is sent out (enum `Msg` above) to filter out future
    // redundant URL requests.
    let rx_url_stream = tokio_stream::wrappers::ReceiverStream::new(rx_url)
        .map(move |mut url| {
            let tx_peer = tx_peer.clone();
            async move {
                println!("Processing {}", url);
                let handle = tokio::spawn(async move {
                    // Query node at url.
                    #[allow(clippy::unwrap_used)]
                    url.set_port(Some(9053)).unwrap();
                    #[allow(clippy::unwrap_used)]
                    let node_conf = NodeConf {
                        addr: PeerAddr(url.socket_addrs(|| Some(9053)).unwrap()[0]),
                        api_key: None,
                        timeout: Some(timeout),
                    };

                    // If active, look up its peers.
                    if get_info(node_conf).await.is_ok() {
                        println!("active nodeConf: {:?}", node_conf);
                        if let Ok(peers) = get_peers_all(node_conf).await {
                            // It's important to send this message before the `AddActiveNode` message
                            // below, to ensure an `count` variable; see (**) below.
                            tx_peer.send(Msg::CheckPeers(peers)).await?;
                        }

                        tx_peer.send(Msg::AddActiveNode(url.clone())).await?;
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

    'loop_: while let Some(p) = rx_peer.recv().await {
        match p {
            Msg::AddActiveNode(mut url) => {
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                println!("added {}", url);
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
                for peer in peers {
                    let mut url = peer.addr.as_http_url();
                    #[allow(clippy::unwrap_used)]
                    url.set_port(None).unwrap();
                    if !visited_peers.contains(&url) {
                        let _ = tx_url.send(url.clone()).await;
                        visited_peers.insert(url);
                        count += 1;
                    }
                }
            }
        }
    }

    drop(tx_url);

    match e.await {
        Ok(Ok(())) => Ok(visited_active_peers
            .difference(&seeds_set)
            .into_iter()
            .cloned()
            .collect()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(PeerDiscoveryError::JoinError),
    }
}

#[derive(Debug)]
/// Peer discovery error
pub enum PeerDiscoveryError {
    /// `Url` error
    UrlError,
    /// mpsc sender error
    MpscSender,
    /// tokio::spawn `JoinError`
    JoinError,
    /// task spawn error
    TaskSpawn,
}

/// Stub that can be removed once `bounded-vec` is updated with this type
pub type NonEmptyVec<T> = BoundedVec<T, 1, { usize::MAX }>;

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
                BoundedU16::new(10).unwrap(),
                Duration::from_millis(800),
            )
            .await
            .unwrap()
        });
        // Currently there are no non-seed nodes with an active REST API!
        assert!(res.is_empty())
    }
}
