//! Chrome implementation of `peer_discovery`.
//!
//! Why? Chrome has a problem with hanging on to [`preflight`] requests where the server does not
//! respond. Every browser request we make of an ergo node will be preceded by a preflight request.
//! These preflight requests are automatically initiated by the browser, and it is not possible for
//! us to interact with it.
//!
//! If a server doesn't respond to any request, then Chrome waits on the preflight request from
//! anywhere from 1.3 to 2.4 minutes. Chrome also doesn't have a high ceiling for parallel requests
//! and running our first implementation of `peer_discovery` in the `non_chrome` submodule brings
//! Chrome to a halt, preventing it from making any further requests until enough of the preflights
//! have been marked as failed. This is quite unfortunate as Firefox and Safari almost immediately
//! drops such preflight requests. See [`this issue`] for more discussion and example runs.
//!
//! So this implementation is largely the same as in `non_chrome` except that we throttle the number
//! of parallel requests made. The majority of ergo nodes do not respond to REST requests, so there
//! will be a large number of preflight requests 'taking up space' and dramatically reducing
//! throughput.
//!
//! # Differences with the original implementation in `non_chrome`
//! - Fine-grained counting and throttling of the number of active parallel requests. It's not
//!   perfect; the preflights are unobservable but we can simulate when they end based on empirical
//!   data.
//! - Task 2 no longer receives URLs but rather specific node requests to be made (either for /info
//!   or /peers/all endpoints)
//! - Instead of `peer_stack` of URLs in task 1, we use a priority queue so that any requests to
//!   /peers/all is pushed through first.
//!
//! [`preflight`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS#preflighted_requests
//! [`this issue`]: https://github.com/ergoplatform/sigma-rust/issues/581#issuecomment-1160564378
use super::PeerDiscoverySettings;
use crate::api::peer_discovery_internals::get_peers_all;
use crate::error::PeerDiscoveryError;
use crate::{api::node::get_info, NodeConf, NodeError, PeerInfo};
use bounded_integer::BoundedU16;
use bounded_vec::NonEmptyVec;
use ergo_chain_types::PeerAddr;
use std::fmt::Debug;
use std::{collections::HashSet, time::Duration};
use url::Url;

// Uncomment the following to enable logging on WASM through the `console_log` macro. Taken from
// https://rustwasm.github.io/wasm-bindgen/examples/console-log.html#srclibrs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
// Note that this is using the `log` function imported above during
// `bare_bones`
($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub(crate) async fn peer_discovery_inner_chrome(
    seeds: NonEmptyVec<Url>,
    max_parallel_tasks: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    if timeout.as_secs() < 90 {
        return Err(PeerDiscoveryError::TimeoutTooShort);
    }

    // Note that 80 seconds is allocated to waiting for preflight requests to naturally timeout by
    // Chrome. The remaining time is spent looking for peers.
    let global_timeout = timeout.checked_sub(Duration::from_secs(80)).unwrap();
    let settings = PeerDiscoverySettings {
        max_parallel_tasks,
        task_2_buffer_length: 50,
        global_timeout,
        timeout_of_individual_node_request: Duration::from_secs(6),
    };

    let (tx_msg, rx_msg) = futures::channel::mpsc::channel::<Msg>(256);
    let (tx_node_request, rx_node_request) =
        futures::channel::mpsc::channel::<NodeRequest>(settings.task_2_buffer_length);
    let node_request_stream = rx_node_request;
    let msg_stream = rx_msg;

    peer_discovery_impl_chrome(
        seeds,
        tx_msg,
        msg_stream,
        tx_node_request,
        node_request_stream,
        settings,
    )
    .await
}

/// Implementation of `peer_discovery`.
async fn peer_discovery_impl_chrome(
    seeds: NonEmptyVec<Url>,
    tx_msg: futures::channel::mpsc::Sender<Msg>,
    msg_stream: futures::channel::mpsc::Receiver<Msg>,
    mut tx_node_request: futures::channel::mpsc::Sender<NodeRequest>,
    node_request_stream: futures::channel::mpsc::Receiver<NodeRequest>,
    settings: PeerDiscoverySettings,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    use futures::future::FutureExt;
    use futures::{SinkExt, StreamExt};

    let max_parallel_requests = settings.max_parallel_tasks.get() as usize;
    let mut seeds_set: HashSet<Url> = HashSet::new();

    for mut seed_url in seeds {
        #[allow(clippy::unwrap_used)]
        seed_url.set_port(None).unwrap();
        seeds_set.insert(seed_url);
    }

    // Task 2 from the schematic above
    spawn_http_request_task_chrome(
        tx_msg,
        node_request_stream,
        settings.max_parallel_tasks,
        settings.timeout_of_individual_node_request,
    );

    // Start with requests to seed nodes.
    for url in &seeds_set {
        tx_node_request
            .send(NodeRequest::Info(url.clone()))
            .await
            .map_err(|_| PeerDiscoveryError::MpscSender)?;
    }

    // (*) This variable represents the number of URLs that need to be checked to see whether it
    // corresponds to an active Ergo node. `count` is crucial to allow this function to terminate,
    // as once it and `chrome_request_count` reaches zero we break the loop below. This leads us to
    // drop `tx_node_request`, which is the sender side of the receiver stream
    // `node_request_stream`, allowing task 1 to end.
    let mut count = seeds_set.len();

    // This variable tracks the number of active requests opened by Chrome. Every request we make of
    // an ergo node requires a 'preflight' request first. This variable tracks such requests too.
    // It's used to restrict the total number of active requests on Chrome.
    let mut chrome_request_count = seeds_set.len() * 2;

    let mut visited_active_peers = HashSet::new();
    let mut visited_peers = HashSet::new();

    use std::collections::BinaryHeap;
    // A collection of node requests to initiate in task 2. We use a BinaryHeap here to ensure that
    // `NodeRequest::PeersAll` messages get processed first.
    let mut pending_requests: BinaryHeap<NodeRequest> = BinaryHeap::new();

    // Here we spawn a task that triggers a signal after `settings.global_timeout` has elapsed.
    let rx_timeout_signal = {
        let (tx, rx) = futures::channel::oneshot::channel::<()>();
        wasm_bindgen_futures::spawn_local(async move {
            crate::wasm_timer::Delay::new(settings.global_timeout)
                .await
                .expect("wasm_timer::Delay: can't spawn global timeout");
            tx.send(()).unwrap();
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

    // This variable equals to true as long as we're checking for new peer nodes. It is set to false
    // once the global timeout is reached.
    let add_peers = true;
    'loop_: while let Some(n) = combined_stream.next().await {
        match n {
            C::RxMsg(p) => {
                // Try pushing as many peers as can be allowed in the (tx_node_request, rx_node_request) channel
                while let Some(peer) = pending_requests.pop() {
                    let mut url = peer.get_url().clone();
                    #[allow(clippy::unwrap_used)]
                    url.set_port(None).unwrap();
                    let room_for_requests = (max_parallel_requests > chrome_request_count)
                        && max_parallel_requests - chrome_request_count >= 2;
                    let peers_all = if let NodeRequest::PeersAll(_) = peer {
                        true
                    } else {
                        false
                    };
                    if peers_all || !visited_peers.contains(&url) {
                        if room_for_requests {
                            match tx_node_request.try_send(peer.clone()) {
                                Ok(_) => {
                                    chrome_request_count += 2;
                                    if !peers_all {
                                        count += 1;
                                    }
                                    console_log!(
                                        "Adding {}. count: {}, chrome count: {}, # visited: {}, visited_active: {}",
                                        url.to_string(),
                                        count,
                                        chrome_request_count,
                                        visited_peers.len(),
                                        visited_active_peers.len(),
                                    );
                                    visited_peers.insert(url);
                                }
                                Err(e) => {
                                    // Push it back on the stack, try again later.
                                    if e.is_full() {
                                        pending_requests.push(peer);
                                        break;
                                    } else if e.is_disconnected() {
                                        return Err(PeerDiscoveryError::MpscSender);
                                    }
                                    unreachable!()
                                }
                            }
                        } else {
                            pending_requests.push(peer);
                            break;
                        }
                    }
                }
                match p {
                    Msg::AddActiveNode(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_active_peers.insert(url.clone());
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 2;
                        console_log!(
                            "/peers/all succeeded. count: {}, chrome count: {}, # visited: {}",
                            count,
                            chrome_request_count,
                            visited_peers.len(),
                        );
                        if count == 0 && chrome_request_count == 0 {
                            break 'loop_;
                        }
                    }
                    Msg::InfoRequestSucceeded(url) => {
                        chrome_request_count -= 2;
                        pending_requests.push(NodeRequest::PeersAll(url));
                        console_log!(
                            "/info succeeded. count: {}, chrome count: {}, # visited: {}",
                            count,
                            chrome_request_count,
                            visited_peers.len(),
                        );
                    }
                    Msg::InfoRequestFailedWithoutTimeout(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 2;
                        console_log!(
                            "/info failed with no timeout. count: {}, chrome count: {}, # visited: {}",
                            count,
                            chrome_request_count,
                            visited_peers.len(),
                        );
                        if count == 0 && chrome_request_count == 0 {
                            break 'loop_;
                        }
                    }
                    Msg::InfoRequestFailedWithTimeout(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 1;
                        console_log!(
                            "/info failed WITH timeout. node count: {}, chrome count: {}, # visited: {}",
                            count,
                            chrome_request_count,
                            visited_peers.len(),
                        );
                    }
                    Msg::PeersAllRequestFailedWithoutTimeout(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 2;
                        if count == 0 && chrome_request_count == 0 {
                            break 'loop_;
                        }
                    }
                    Msg::PeersAllRequestFailedWithTimeout(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 1;
                    }
                    Msg::PreflightRequestFailed => {
                        chrome_request_count -= 1;
                        console_log!(
                            "Preflight request failed (by simulation), node count: {}, chrome count: {}, # visited: {}",
                            count,
                            chrome_request_count,
                            visited_peers.len(),
                        );
                        if count == 0 && pending_requests.is_empty() {
                            break 'loop_;
                        }
                    }
                    Msg::CheckPeers(mut peers) => {
                        use rand::seq::SliceRandom;
                        use rand::thread_rng;
                        peers.shuffle(&mut thread_rng());
                        if add_peers {
                            pending_requests.extend(
                                peers
                                    .into_iter()
                                    .map(|p| NodeRequest::Info(p.addr.as_http_url())),
                            );
                        }
                    }
                }
            }
            C::RxTimeoutSignal => {
                //add_peers = false;
                //pending_requests.clear();
                break;
            }
        }
    }

    drop(tx_node_request);
    let coll: Vec<_> = visited_active_peers
        .difference(&seeds_set)
        .into_iter()
        .cloned()
        .collect();

    // Uncomment for debugging
    console_log!(
        "Total # nodes visited: {}, # peers found: {}",
        visited_peers.len(),
        coll.len()
    );
    console_log!("Waiting 80sec for Chrome to relinquish pending HTTP requests");
    crate::wasm_timer::Delay::new(Duration::from_secs(80)).await?;
    Ok(coll)
}

/// Given a stream that receives URLs of full ergo nodes, spawn a task (task 2 in the schematic
/// above) which checks if it is active.  If so, request its peers. In all cases, a message (enum
/// `Msg`) is sent out to notify the listener.
fn spawn_http_request_task_chrome(
    tx_msg: futures::channel::mpsc::Sender<Msg>,
    node_request_stream: impl futures::Stream<Item = NodeRequest> + Send + 'static,
    max_parallel_tasks: BoundedU16<1, { u16::MAX }>,
    request_timeout_duration: Duration,
) {
    use futures::{SinkExt, StreamExt};
    use wasm_bindgen_futures::spawn_local;

    let mapped_stream = node_request_stream
        .map(move |node_request| {
            let mut tx_msg = tx_msg.clone();
            async move {
                spawn_local(async move {
                    let mut url = node_request.get_url().clone();
                    #[allow(clippy::unwrap_used)]
                    url.set_port(Some(9053)).unwrap();
                    #[allow(clippy::unwrap_used)]
                    let node_conf = NodeConf {
                        addr: PeerAddr(url.socket_addrs(|| Some(9053)).unwrap()[0]),
                        api_key: None,
                        timeout: Some(request_timeout_duration),
                    };

                    // When a timeout is initiated, the message below is how
                    // Chrome represents the timeout error.
                    let chrome_timeout_str = "error sending request: JsValue(AbortError: \
                                             The user aborted a request.";
                    match node_request {
                        NodeRequest::Info(url) => match get_info(node_conf).await {
                            Ok(_) => {
                                let _ = tx_msg.send(Msg::InfoRequestSucceeded(url)).await;
                            }
                            Err(e) => {
                                if let NodeError::ReqwestError(r) = e {
                                    if r.to_string().starts_with(chrome_timeout_str) {
                                        let _ = tx_msg
                                            .send(Msg::InfoRequestFailedWithTimeout(url))
                                            .await;
                                        spawn_local(async move {
                                            crate::wasm_timer::Delay::new(Duration::from_secs(80))
                                                .await
                                                .unwrap();
                                            let _ = tx_msg.send(Msg::PreflightRequestFailed);
                                        });
                                    } else {
                                        #[allow(clippy::unwrap_used)]
                                        let _ = tx_msg
                                            .send(Msg::InfoRequestFailedWithoutTimeout(url))
                                            .await;
                                    }
                                } else {
                                    #[allow(clippy::unwrap_used)]
                                    let _ = tx_msg
                                        .send(Msg::InfoRequestFailedWithoutTimeout(url))
                                        .await;
                                }
                            }
                        },
                        NodeRequest::PeersAll(url) => {
                            match get_peers_all(node_conf).await {
                                Ok(peers) => {
                                    // It's important to send this message before the `AddActiveNode`
                                    // message below, to ensure an accurate `count` variable in task 1;
                                    // see (*) above in `peer_discovery_inner`.
                                    tx_msg.send(Msg::CheckPeers(peers)).await.unwrap();
                                    tx_msg.send(Msg::AddActiveNode(url.clone())).await.unwrap();
                                }
                                Err(e) => {
                                    if let NodeError::ReqwestError(r) = e {
                                        if r.to_string().starts_with(chrome_timeout_str) {
                                            let _ = tx_msg
                                                .send(Msg::PeersAllRequestFailedWithTimeout(url))
                                                .await;

                                            // This task simulates the waiting of a preflight
                                            // request that will timeout from no response.
                                            spawn_local(async move {
                                                crate::wasm_timer::Delay::new(Duration::from_secs(
                                                    80,
                                                ))
                                                .await
                                                .unwrap();

                                                let _ =
                                                    tx_msg.send(Msg::PreflightRequestFailed).await;
                                            });
                                        }
                                    } else {
                                        let _ = tx_msg
                                            .send(Msg::PeersAllRequestFailedWithoutTimeout(url))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                });
            }
        })
        .buffer_unordered(max_parallel_tasks.get() as usize); // Allow for parallel requests

    let spawn_fn_new = wasm_bindgen_futures::spawn_local;

    // (*) Run stream to completion.
    spawn_fn_new(mapped_stream.for_each(|_| async move {}));
}

/// Used in the implementation of `peer_discovery`.
#[derive(Debug)]
pub(crate) enum Msg {
    /// Indicates that the ergo node at the given URL is active. This means that a GET request
    /// to the node's /info endpoint responds with code 200 OK.
    AddActiveNode(Url),
    /// /info returns code 200
    InfoRequestSucceeded(Url),
    /// Indicates that the ergo node at the given URL is inactive. This means that a GET request to
    /// the node's /info endpoint does not respond with code 200 OK. In addition a response to the
    /// preflight request was given (connection refused).
    InfoRequestFailedWithoutTimeout(Url),
    /// Indicates that the ergo node at the given URL is inactive. This means that a GET request to
    /// the node's /info endpoint does not respond with code 200 OK.
    InfoRequestFailedWithTimeout(Url),
    /// /peers/all failed with timeout
    PeersAllRequestFailedWithTimeout(Url),
    /// /peers/all failed without timeout
    PeersAllRequestFailedWithoutTimeout(Url),
    /// Preflight request failed
    PreflightRequestFailed,
    /// A list of peers of an active ergo node, returned from a GET on the /peers/all endpoint.
    CheckPeers(Vec<PeerInfo>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// Represents HTTP requests to be made of an ergo node. Note: PeersAll > Info, see
// https://doc.rust-lang.org/stable/std/cmp/trait.Ord.html#derivable
enum NodeRequest {
    /// /info endpoint
    Info(Url),
    /// /peers/all endpoint
    PeersAll(Url),
}

impl NodeRequest {
    fn get_url(&self) -> &Url {
        match self {
            NodeRequest::Info(url) => url,
            NodeRequest::PeersAll(url) => url,
        }
    }
}
