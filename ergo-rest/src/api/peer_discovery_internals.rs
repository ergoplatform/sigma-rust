//! This module contains the `peer_discovery` implementation. The original implementation is found
//! in the `non_chrome` sub-module, which can be used on tokio as well as web-browsers that are not
//! Chrome (tested on Firefox and Safari). However there are certain limitations in Chrome that
//! require us to have a custom implementation.
#[cfg(target_arch = "wasm32")]
mod chrome;
mod non_chrome;

use bounded_integer::BoundedU16;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
pub(crate) use chrome::peer_discovery_inner_chrome;
#[cfg(target_arch = "wasm32")]
pub use chrome::ChromePeerDiscoveryScan;
pub(crate) use non_chrome::peer_discovery_inner;

use crate::{NodeConf, NodeError, PeerInfo};

use super::{build_client, set_req_headers};

/// GET on /peers/all endpoint
async fn get_peers_all(node: NodeConf) -> Result<Vec<PeerInfo>, NodeError> {
    #[allow(clippy::unwrap_used)]
    let url = node.addr.as_http_url().join("peers/all").unwrap();
    let client = build_client(&node)?;
    let rb = client.get(url);
    let response = set_req_headers(rb, node).send().await?;
    Ok(response.json::<Vec<PeerInfo>>().await?)
}

struct PeerDiscoverySettings {
    max_parallel_tasks: BoundedU16<1, { u16::MAX }>,
    task_2_buffer_length: usize,
    global_timeout: Duration,
    timeout_of_individual_node_request: Duration,
}
