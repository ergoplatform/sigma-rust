//! Ergo node REST API endpoints

use async_trait::async_trait;
use bounded_integer::BoundedU16;
use bounded_vec::NonEmptyVec;
use ergo_chain_types::BlockId;
use ergo_chain_types::PeerAddr;
use ergo_nipopow::NipopowProof;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::RequestBuilder;
use std::collections::HashSet;
use std::fmt::Debug;
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

#[derive(Debug)]
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

/// Given a list of seed nodes, recursively determine a Vec of all known peer nodes.
pub async fn peer_discovery(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    let buffer_size = usize::max(max_parallel_requests.get() as usize, seeds.len());
    #[cfg(not(target_arch = "wasm32"))]
    let (tx_peer, rx_peer) = tokio::sync::mpsc::channel::<Msg>(buffer_size);
    #[cfg(not(target_arch = "wasm32"))]
    let (tx_url, rx_url) = tokio::sync::mpsc::channel::<Url>(buffer_size);
    #[cfg(not(target_arch = "wasm32"))]
    let url_stream = tokio_stream::wrappers::ReceiverStream::new(rx_url);

    #[cfg(target_arch = "wasm32")]
    let (tx_peer, rx_peer) = futures::channel::mpsc::channel::<Msg>(buffer_size);
    #[cfg(target_arch = "wasm32")]
    let (tx_url, rx_url) = futures::channel::mpsc::channel::<Url>(buffer_size);
    #[cfg(target_arch = "wasm32")]
    let url_stream = rx_url;

    peer_discovery_inner(
        seeds,
        max_parallel_requests,
        tx_peer,
        rx_peer,
        tx_url,
        url_stream,
        timeout,
    )
    .await
}

/// Given a stream that receives URLs of full ergo nodes, spawn a task which checks if it is active.
/// If so, request its peers. In all cases, a message (enum `Msg`) is sent out to notify the
/// listener.
fn spawn_http_request_task<
    SendMsg: ChannelInfallibleSender<Msg> + Clone + Send + Sync + 'static,
>(
    tx_peer: SendMsg,
    url_stream: impl futures::Stream<Item = Url> + Send + 'static,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    request_timeout_duration: Duration,
) {
    use futures::StreamExt;

    // Note that `tokio` - the de facto standard async runtime - is not supported on WASM. We need
    // to spawn tasks for HTTP requests, and for WASM we rely on the `wasm_bindgen_futures` crate.
    #[cfg(not(target_arch = "wasm32"))]
    let spawn_fn = tokio::spawn;

    #[cfg(target_arch = "wasm32")]
    let spawn_fn = wasm_bindgen_futures::spawn_local;

    let mapped_stream = url_stream
        .map(move |mut url| {
            let tx_peer = tx_peer.clone();
            async move {
                let _handle = spawn_fn(async move {
                    // Query node at url.
                    #[allow(clippy::unwrap_used)]
                    url.set_port(Some(9053)).unwrap();
                    #[allow(clippy::unwrap_used)]
                    let node_conf = NodeConf {
                        addr: PeerAddr(url.socket_addrs(|| Some(9053)).unwrap()[0]),
                        api_key: None,
                        timeout: Some(request_timeout_duration),
                    };

                    // If active, look up its peers.
                    if get_info(node_conf).await.is_ok() {
                        match get_peers_all(node_conf).await {
                            Ok(peers) => {
                                // Note that the unwraps on `tx_peer.send(..)` below will not fail.
                                // The send call can only fail if the receiver either drops or it
                                // calls its `closed` method. We ensure that this won't happen, see
                                // (**) below.

                                // It's important to send this message before the `AddActiveNode` message
                                // below, to ensure an `count` variable; see (**) below.
                                tx_peer.infallible_send(Msg::CheckPeers(peers)).await;
                                tx_peer
                                    .infallible_send(Msg::AddActiveNode(url.clone()))
                                    .await;
                            }
                            Err(_) => {
                                #[allow(clippy::unwrap_used)]
                                tx_peer.infallible_send(Msg::AddInactiveNode(url)).await;
                            }
                        }
                    } else {
                        #[allow(clippy::unwrap_used)]
                        tx_peer.infallible_send(Msg::AddInactiveNode(url)).await;
                    }
                });
            }
        })
        .buffer_unordered(max_parallel_requests.get() as usize); // Allow for parallel requests

    // Note: We need to define another binding to the spawn function to get around the Rust type
    // checker.
    #[cfg(not(target_arch = "wasm32"))]
    let spawn_fn_new = tokio::spawn;

    #[cfg(target_arch = "wasm32")]
    let spawn_fn_new = wasm_bindgen_futures::spawn_local;

    // (*) Run stream to completion.
    spawn_fn_new(mapped_stream.for_each(|_| async move {}));
}

async fn peer_discovery_inner<
    RecvMsg: ChannelReceiver<Msg>,
    SendMsg: 'static + ChannelInfallibleSender<Msg> + Clone + Send + Sync,
    SendUrl: 'static + ChannelInfallibleSender<Url> + ChannelTrySender<Url> + Clone + Send + Sync,
>(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    tx_peer: SendMsg,
    mut rx_peer: RecvMsg,
    tx_url: SendUrl,
    url_stream: impl futures::Stream<Item = Url> + Send + 'static,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    let mut seeds_set: HashSet<Url> = HashSet::new();

    for mut seed_url in seeds {
        #[allow(clippy::unwrap_used)]
        seed_url.set_port(None).unwrap();
        seeds_set.insert(seed_url);
    }

    spawn_http_request_task(tx_peer, url_stream, max_parallel_requests, timeout);

    for url in &seeds_set {
        tx_url.infallible_send(url.clone()).await;
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
                    Err(TrySendError::Full) => {
                        // Push it back on the stack, try again later.
                        peer_stack.push(peer);
                        break;
                    }
                    Err(TrySendError::Closed) => {
                        return Err(PeerDiscoveryError::MpscSender);
                    }
                }
            }
        }
        match p {
            Msg::AddActiveNode(mut url) => {
                #[allow(clippy::unwrap_used)]
                url.set_port(None).unwrap();
                println!("Active node {}", url);
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

    println!("Total # nodes visited: {}", visited_peers.len());

    drop(tx_url);
    let coll: Vec<_> = visited_active_peers
        .difference(&seeds_set)
        .into_iter()
        .cloned()
        .collect();
    Ok(coll)
}

/// This trait abstracts over the `send` method of channel senders
#[async_trait]
trait ChannelInfallibleSender<T> {
    /// A send that cannot fail.
    async fn infallible_send(&self, value: T);
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl<T: Debug + Send> ChannelInfallibleSender<T> for tokio::sync::mpsc::Sender<T> {
    async fn infallible_send(&self, value: T) {
        #[allow(clippy::unwrap_used)]
        let _ = self.send(value).await.unwrap();
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl<T> ChannelInfallibleSender<T> for futures::channel::mpsc::Sender<T> {
    async fn infallible_send(&self, value: T) {
        use futures::sink::SinkExt;
        #[allow(clippy::unwrap_used)]
        self.send(value).await.unwrap()
    }
}

/// This trait abstracts over the `try_send` method of channel senders
trait ChannelTrySender<T> {
    fn try_send(&self, value: T) -> Result<(), TrySendError>;
}

/// Errors that can return from `try_send(..)` calls are converted into the following enum.
enum TrySendError {
    /// Receiver's buffer is full
    Full,
    /// Receiver is no longer active. Either it was specifically closed or dropped.
    Closed,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> ChannelTrySender<T> for tokio::sync::mpsc::Sender<T> {
    fn try_send(&self, value: T) -> Result<(), TrySendError> {
        use tokio::sync::mpsc::error::TrySendError as TokioTrySendError;
        match self.try_send(value) {
            Ok(()) => Ok(()),
            Err(TokioTrySendError::Full(_)) => Err(TrySendError::Full),
            Err(TokioTrySendError::Closed(_)) => Err(TrySendError::Closed),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<T> ChannelTrySender<T> for futures::channel::mpsc::Sender<T> {
    fn try_send(&self, value: T) -> Result<(), TrySendError> {
        match futures::channel::mpsc::Sender::try_send(&self, value) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.is_full() {
                    Err(TrySendError::Full)
                } else {
                    Err(TrySendError::Closed)
                }
            }
        }
    }
}

/// This trait abstracts over channel receivers
#[async_trait]
trait ChannelReceiver<T> {
    /// Receive a value
    async fn recv(&mut self) -> Option<T>;
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl<T: Send> ChannelReceiver<T> for tokio::sync::mpsc::Receiver<T> {
    async fn recv(&mut self) -> Option<T> {
        self.recv().await
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl<T: Send> ChannelReceiver<T> for futures::channel::mpsc::Receiver<T> {
    async fn recv(&mut self) -> Option<T> {
        use futures::StreamExt;
        self.next().await
    }
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
