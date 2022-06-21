//! Chrome implementation of `peer_discovery`
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
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    timeout: Duration,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    let settings = PeerDiscoverySettings {
        max_parallel_requests,
        task_2_buffer_length: 50,
        global_timeout: timeout,
        timeout_of_individual_node_request: Duration::from_secs(6),
    };

    let (tx_msg, rx_msg) = futures::channel::mpsc::channel::<ChromeMsg>(256);
    let (tx_url, rx_url) =
        futures::channel::mpsc::channel::<NodeRequest>(settings.task_2_buffer_length);
    let url_stream = rx_url;
    let msg_stream = rx_msg;

    peer_discovery_impl_chrome(seeds, tx_msg, msg_stream, tx_url, url_stream, settings).await
}

/// Implementation of `peer_discovery`.
async fn peer_discovery_impl_chrome(
    seeds: NonEmptyVec<Url>,
    tx_msg: futures::channel::mpsc::Sender<ChromeMsg>,
    msg_stream: futures::channel::mpsc::Receiver<ChromeMsg>,
    mut tx_url: futures::channel::mpsc::Sender<NodeRequest>,
    url_stream: futures::channel::mpsc::Receiver<NodeRequest>,
    settings: PeerDiscoverySettings,
) -> Result<Vec<Url>, PeerDiscoveryError> {
    use futures::future::FutureExt;
    use futures::{SinkExt, StreamExt};

    let max_parallel_requests = settings.max_parallel_requests.get() as usize;
    let timeout_of_individual_node_request = settings.timeout_of_individual_node_request.clone();
    let mut seeds_set: HashSet<Url> = HashSet::new();

    for mut seed_url in seeds {
        #[allow(clippy::unwrap_used)]
        seed_url.set_port(None).unwrap();
        seeds_set.insert(seed_url);
    }

    // Task 2 from the schematic above
    spawn_http_request_task_chrome(
        tx_msg,
        url_stream,
        settings.max_parallel_requests,
        settings.timeout_of_individual_node_request,
    );

    // Start with requests to seed nodes.
    for url in &seeds_set {
        tx_url.send(NodeRequest::Info(url.clone())).await;
    }

    // (*) This variable represents the number of URLs that need to be checked to see whether it
    // corresponds to an active Ergo node. `count` is crucial to allow this function to terminate,
    // as once it reaches zero we break the loop below. This leads us to drop `tx_url`, which is the
    // sender side of the receiver stream `rx_url_stream`, allowing task 1 to end.
    let mut count = seeds_set.len();

    let mut chrome_request_count = seeds_set.len() * 2;

    let mut visited_active_peers = HashSet::new();
    let mut visited_peers = HashSet::new();

    use std::collections::BinaryHeap;
    // Stack of peers to evaluate. Used as a growable buffer for when the (tx_url, rx_url) channel
    // gets full.
    let mut peer_stack: BinaryHeap<NodeRequest> = BinaryHeap::new();

    // Here we spawn a task that triggers a signal after `settings.global_timeout` has elapsed.
    let rx_timeout_signal = {
        let (tx, rx) = futures::channel::oneshot::channel::<()>();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::wasm_timer::Delay::new(settings.global_timeout).await;
            let _ = tx.send(());
        });
        rx.into_stream()
    };

    // In addition to listening for `Msg`s from the HTTP request task, we need to watch for the
    // timeout signal so we can exit early. The solution is to combine the streams.
    enum C {
        RxMsg(ChromeMsg),
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
                // Try pushing as many peers as can be allowed in the (tx_url, rx_url) channel
                while let Some(peer) = peer_stack.pop() {
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
                            match tx_url.try_send(peer.clone()) {
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
                                        peer_stack.push(peer);
                                        break;
                                    } else if e.is_disconnected() {
                                        return Err(PeerDiscoveryError::MpscSender);
                                    }
                                    unreachable!()
                                }
                            }
                        } else {
                            peer_stack.push(peer);
                            break;
                        }
                    }
                }
                match p {
                    ChromeMsg::AddActiveNode(mut url) => {
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
                    ChromeMsg::InfoRequestSucceeded(url) => {
                        chrome_request_count -= 2;
                        peer_stack.push(NodeRequest::PeersAll(url));
                        console_log!(
                            "/info succeeded. count: {}, chrome count: {}, # visited: {}",
                            count,
                            chrome_request_count,
                            visited_peers.len(),
                        );
                    }
                    ChromeMsg::InfoRequestFailedWithoutTimeout(mut url) => {
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
                    ChromeMsg::InfoRequestFailedWithTimeout(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 1;
                        console_log!(
                            "/info failed WITH timeout. count: {}, chrome count: {}, # visited: {}",
                            count,
                            chrome_request_count,
                            visited_peers.len(),
                        );
                    }
                    ChromeMsg::PeersAllRequestFailedWithoutTimeout(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 2;
                        if count == 0 && chrome_request_count == 0 {
                            break 'loop_;
                        }
                    }
                    ChromeMsg::PeersAllRequestFailedWithTimeout(mut url) => {
                        #[allow(clippy::unwrap_used)]
                        url.set_port(None).unwrap();
                        visited_peers.insert(url);
                        count -= 1;

                        chrome_request_count -= 1;
                    }
                    ChromeMsg::PreflightRequestFailed => {
                        chrome_request_count -= 1;
                        console_log!(
                            "preflight req failed, chrome count: {}, node count: {}, # visited: {}",
                            chrome_request_count,
                            count,
                            visited_peers.len(),
                        );
                        if count == 0 && peer_stack.is_empty() {
                            break 'loop_;
                        }
                    }
                    ChromeMsg::CheckPeers(mut peers) => {
                        use rand::seq::SliceRandom;
                        use rand::thread_rng;
                        peers.shuffle(&mut thread_rng());
                        if add_peers {
                            peer_stack.extend(
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
                //peer_stack.clear();
                break;
            }
        }
    }

    drop(tx_url);
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
    let _ = crate::wasm_timer::Delay::new(Duration::from_secs(80)).await;
    Ok(coll)
}

/// Given a stream that receives URLs of full ergo nodes, spawn a task (task 2 in the schematic
/// above) which checks if it is active.  If so, request its peers. In all cases, a message (enum
/// `Msg`) is sent out to notify the listener.
fn spawn_http_request_task_chrome(
    tx_peer: futures::channel::mpsc::Sender<ChromeMsg>,
    url_stream: impl futures::Stream<Item = NodeRequest> + Send + 'static,
    max_parallel_requests: BoundedU16<1, { u16::MAX }>,
    request_timeout_duration: Duration,
) {
    use futures::{SinkExt, StreamExt};
    use wasm_bindgen_futures::spawn_local;

    let mapped_stream = url_stream
        .map(move |node_request| {
            let mut tx_peer = tx_peer.clone();
            async move {
                // `tokio::spawn` returns a `JoinHandle` which we make sure to drop. If we don't drop
                // and instead await on it, performance suffers greatly (~ 5x slower). In WASM case
                // we don't need to worry because `wasm_bindgen_futures::spawn_local` returns ().
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
                    match node_request {
                        NodeRequest::Info(url) => match get_info(node_conf).await {
                            Ok(_) => {
                                let _ = tx_peer.send(ChromeMsg::InfoRequestSucceeded(url)).await;
                            }
                            Err(e) => {
                                if let NodeError::ReqwestError(r) = e {
                                    if r.to_string().starts_with(
                                        "error sending request: JsValue(AbortError: \
                                             The user aborted a request.",
                                    ) {
                                        let _ = tx_peer
                                            .send(ChromeMsg::InfoRequestFailedWithTimeout(url))
                                            .await;
                                        spawn_local(async move {
                                            let _ = crate::wasm_timer::Delay::new(
                                                Duration::from_secs(80),
                                            )
                                            .await;
                                            let _ = tx_peer
                                                .send(ChromeMsg::PreflightRequestFailed)
                                                .await;
                                        });
                                    } else {
                                        #[allow(clippy::unwrap_used)]
                                        let _ = tx_peer
                                            .send(ChromeMsg::InfoRequestFailedWithoutTimeout(url))
                                            .await;
                                    }
                                } else {
                                    #[allow(clippy::unwrap_used)]
                                    let _ = tx_peer
                                        .send(ChromeMsg::InfoRequestFailedWithoutTimeout(url))
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
                                    let _ = tx_peer.send(ChromeMsg::CheckPeers(peers)).await;
                                    let _ =
                                        tx_peer.send(ChromeMsg::AddActiveNode(url.clone())).await;
                                }
                                Err(e) => {
                                    if let NodeError::ReqwestError(r) = e {
                                        if r.to_string().starts_with(
                                            "error sending request: JsValue(AbortError: \
                                             The user aborted a request.",
                                        ) {
                                            let _ = tx_peer
                                                .send(ChromeMsg::PeersAllRequestFailedWithTimeout(
                                                    url,
                                                ))
                                                .await;
                                            spawn_local(async move {
                                                let _ = crate::wasm_timer::Delay::new(
                                                    Duration::from_secs(80),
                                                )
                                                .await;
                                                let _ = tx_peer
                                                    .send(ChromeMsg::PreflightRequestFailed)
                                                    .await;
                                            });
                                        }
                                    } else {
                                        let _ = tx_peer
                                            .send(ChromeMsg::PeersAllRequestFailedWithoutTimeout(
                                                url,
                                            ))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                });
            }
        })
        .buffer_unordered(max_parallel_requests.get() as usize); // Allow for parallel requests

    let spawn_fn_new = wasm_bindgen_futures::spawn_local;

    // (*) Run stream to completion.
    spawn_fn_new(mapped_stream.for_each(|_| async move {}));
}

/// Used in the implementation of `peer_discovery`
#[derive(Debug)]
pub(crate) enum ChromeMsg {
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
// Note: PeersAll > Info, see https://doc.rust-lang.org/stable/std/cmp/trait.Ord.html#derivable
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
