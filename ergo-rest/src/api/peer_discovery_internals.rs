//! This module contains the implementation of `peer_discovery`.  It's structured as 2 separate
//! tasks:
//!
//!  - Task 1 is responsible for tracking which nodes are active/inactive and making sure that any
//!    given ergo node is queried exactly once.
//!  - Task 2's job is to wait for a URL from task 1, make the actual HTTP requests to that URL, and
//!    to report the result back to task 1.
//! ```text
//!                              <ergo node URL>
//!               __________________________________________________
//!              |                                                  |
//!              |                                                  v
//!  /----------------------\                   /----------------------\
//!  | 1. Track node status |                   | 2. HTTP request task |
//!  \----------------------/                   \----------------------/
//!              ^                                                  |
//!              |__________________________________________________|
//!                <active node| non-active node| list of peers>   
//! ```
use crate::error::PeerDiscoveryError;
use crate::{api::node::get_info, NodeConf, NodeError, PeerInfo};
use async_trait::async_trait;
use bounded_integer::BoundedU16;
use bounded_vec::NonEmptyVec;
use ergo_chain_types::PeerAddr;
use std::fmt::Debug;
use std::{collections::HashSet, time::Duration};
use url::Url;

use super::{build_client, set_req_headers};

pub(crate) async fn peer_discovery_inner(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    let buffer_size = usize::max(max_parallel_requests.get() as usize, seeds.len());
    #[cfg(not(target_arch = "wasm32"))]
    let (tx_msg, rx_msg) =
        tokio::sync::mpsc::channel::<super::peer_discovery_internals::Msg>(buffer_size);
    #[cfg(not(target_arch = "wasm32"))]
    let (tx_url, rx_url) = tokio::sync::mpsc::channel::<Url>(buffer_size);
    #[cfg(not(target_arch = "wasm32"))]
    let url_stream = tokio_stream::wrappers::ReceiverStream::new(rx_url);
    #[cfg(not(target_arch = "wasm32"))]
    let msg_stream = tokio_stream::wrappers::ReceiverStream::new(rx_msg);

    #[cfg(target_arch = "wasm32")]
    let (tx_msg, rx_msg) = futures::channel::mpsc::channel::<Msg>(buffer_size);
    #[cfg(target_arch = "wasm32")]
    let (tx_url, rx_url) = futures::channel::mpsc::channel::<Url>(buffer_size);
    #[cfg(target_arch = "wasm32")]
    let url_stream = rx_url;
    #[cfg(target_arch = "wasm32")]
    let msg_stream = rx_msg;

    super::peer_discovery_internals::peer_discovery_impl(
        seeds,
        max_parallel_requests,
        tx_msg,
        msg_stream,
        tx_url,
        url_stream,
        timeout,
    )
    .await
}

/// Implementation of `peer_discovery`.
async fn peer_discovery_impl<
    SendMsg: 'static + ChannelInfallibleSender<Msg> + Clone + Send + Sync,
    SendUrl: 'static + ChannelInfallibleSender<Url> + ChannelTrySender<Url> + Clone + Send + Sync,
>(
    seeds: NonEmptyVec<Url>,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    tx_msg: SendMsg,
    msg_stream: impl futures::Stream<Item = Msg> + Send + 'static,
    mut tx_url: SendUrl,
    url_stream: impl futures::Stream<Item = Url> + Send + 'static,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    use futures::future::FutureExt;
    use futures::StreamExt;

    let mut seeds_set: HashSet<Url> = HashSet::new();

    for mut seed_url in seeds {
        #[allow(clippy::unwrap_used)]
        seed_url.set_port(None).unwrap();
        seeds_set.insert(seed_url);
    }

    // Task 2 from the schematic above
    spawn_http_request_task(
        tx_msg,
        url_stream,
        max_parallel_requests,
        Duration::from_secs(2),
    );

    // Start with requests to seed nodes.
    for url in &seeds_set {
        tx_url.infallible_send(url.clone()).await;
    }

    // (*) This variable represents the number of URLs that need to be checked to see whether it
    // corresponds to an active Ergo node. `count` is crucial to allow this function to terminate,
    // as once it reaches zero we break the loop below. This leads us to drop `tx_url`, which is the
    // sender side of the receiver stream `rx_url_stream`, allowing task 1 to end.
    let mut count = seeds_set.len();

    let mut visited_active_peers = HashSet::new();
    let mut visited_peers = HashSet::new();

    // Stack of peers to evaluate. Used as a growable buffer for when the (tx_url, rx_url) channel
    // gets full.
    let mut peer_stack: Vec<PeerInfo> = vec![];

    // Here we spawn a task that triggers a signal after `timeout` has elapsed.
    #[cfg(target_arch = "wasm32")]
    let rx_timeout_signal = {
        let (tx, rx) = futures::channel::oneshot::channel::<()>();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_timer::Delay::new(timeout).await;
            let _ = tx.send(());
        });
        rx.into_stream()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let rx_timeout_signal = {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            let _ = tokio::time::sleep(timeout).await;
            let _ = tx.send(());
        });
        rx.into_stream()
    };

    // In addition to listening for `Msg`s from the HTTP request task, we need to watch for the
    // timeout signal so we can exit early. The solution is to combine the streams.
    enum C {
        RxMsg(Msg),
        RxTimeoutSignal,
    }

    type CombinedStream = std::pin::Pin<Box<dyn futures::stream::Stream<Item = C> + Send>>;

    let streams: Vec<CombinedStream> = vec![
        msg_stream.map(C::RxMsg).boxed(),
        rx_timeout_signal.map(|_| C::RxTimeoutSignal).boxed(),
    ];
    let mut combined_stream = futures::stream::select_all(streams);

    'loop_: while let Some(n) = combined_stream.next().await {
        match n {
            C::RxMsg(p) => {
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
            C::RxTimeoutSignal => {
                break 'loop_;
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

/// Given a stream that receives URLs of full ergo nodes, spawn a task (task 2 in the schematic
/// above) which checks if it is active.  If so, request its peers. In all cases, a message (enum
/// `Msg`) is sent out to notify the listener.
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
            let mut tx_peer = tx_peer.clone();
            async move {
                // `tokio::spawn` returns a `JoinHandle` which we make sure to drop. If we don't drop
                // and instead await on it, performance suffers greatly (~ 5x slower). In WASM case
                // we don't need to worry because `wasm_bindgen_futures::spawn_local` returns ().
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
                                // It's important to send this message before the `AddActiveNode`
                                // message below, to ensure an accurate `count` variable in task 1;
                                // see (*) above in `peer_discovery_inner`.
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

/// GET on /peers/all endpoint
async fn get_peers_all(node: NodeConf) -> Result<Vec<PeerInfo>, NodeError> {
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join("peers/all").unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    let response = set_req_headers(rb, node).send().await?;
    Ok(response.json::<Vec<PeerInfo>>().await?)
}

/// Used in the implementation of `peer_discovery`
#[derive(Debug)]
pub(crate) enum Msg {
    /// Indicates that the ergo node at the given URL is active. This means that a GET request
    /// to the node's /info endpoint responds with code 200 OK.
    AddActiveNode(Url),
    /// Indicates that the ergo node at the given URL is inactive. This means that a GET request
    /// to the node's /info endpoint does not respond with code 200 OK.
    AddInactiveNode(Url),
    /// A list of peers of an active ergo node, returned from a GET on the /peers/all endpoint.
    CheckPeers(Vec<PeerInfo>),
}

/// This trait abstracts over the `send` method of channel senders, assuming no failure.
#[async_trait]
trait ChannelInfallibleSender<T> {
    /// A send that cannot fail.
    async fn infallible_send(&mut self, value: T);
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl<T: Debug + Send> ChannelInfallibleSender<T> for tokio::sync::mpsc::Sender<T> {
    async fn infallible_send(&mut self, value: T) {
        // If error results, just discard it.
        let _ = self.send(value).await;
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl<T: Debug + Send> ChannelInfallibleSender<T> for futures::channel::mpsc::Sender<T> {
    async fn infallible_send(&mut self, value: T) {
        use futures::sink::SinkExt;
        // If error results, just discard it.
        let _ = self.send(value).await;
    }
}

/// This trait abstracts over the `try_send` method of channel senders
trait ChannelTrySender<T> {
    fn try_send(&mut self, value: T) -> Result<(), TrySendError>;
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
    fn try_send(&mut self, value: T) -> Result<(), TrySendError> {
        use tokio::sync::mpsc::error::TrySendError as TokioTrySendError;
        match tokio::sync::mpsc::Sender::try_send(self, value) {
            Ok(()) => Ok(()),
            Err(TokioTrySendError::Full(_)) => Err(TrySendError::Full),
            Err(TokioTrySendError::Closed(_)) => Err(TrySendError::Closed),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<T> ChannelTrySender<T> for futures::channel::mpsc::Sender<T> {
    fn try_send(&mut self, value: T) -> Result<(), TrySendError> {
        match futures::channel::mpsc::Sender::try_send(self, value) {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::str::FromStr;

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
}
