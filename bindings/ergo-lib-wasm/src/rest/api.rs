//! Wasm API for ergo_rest::api
//!
//! Note that all the functions for GET requests are not async and furthermore return an instannce
//! of `js_sys::Promise`. The reason is some of the args are passed in by reference, which is a
//! problem since futures need to have a 'static lifetime. The workaround is to clone the args and
//! pass it into an `async move` block, and convert that into a JS promise directly (described in
//! https://github.com/rustwasm/wasm-bindgen/issues/1858).

use wasm_bindgen::prelude::*;

use super::node_conf::NodeConf;
use crate::{
    block_header::{BlockHeader, BlockId},
    error_conversion::to_js,
    nipopow::NipopowProof,
    transaction::TxId,
};
use bounded_vec::NonEmptyVec;
use std::time::Duration;

#[wasm_bindgen]
/// GET on /info endpoint
pub fn get_info(node: &NodeConf) -> js_sys::Promise {
    // Note that we can't pass in `node` by value as it will fail on the JS side if used again,
    // despite the Copy implementation.  The problem is a bit mysterious; after calling this
    // function, `node` isn't null on the JS side but when used again it will crash on the rust side
    // (it complains of a null pointer there).
    //
    // A related issue is here: https://github.com/rustwasm/wasm-bindgen/issues/2204
    #[allow(clippy::clone_on_copy)]
    let node_cloned = node.0.clone();
    wasm_bindgen_futures::future_to_promise(async move {
        let info = ergo_lib::ergo_rest::api::node::get_info(node_cloned)
            .await
            .map_err(to_js)
            .map(super::node_info::NodeInfo::from)?;
        Ok(wasm_bindgen::JsValue::from(info))
    })
}

#[wasm_bindgen]
/// GET on /blocks/{header_id}/header endpoint
pub fn get_header(node: &NodeConf, header_id: &BlockId) -> js_sys::Promise {
    let header_id_cloned = header_id.0.clone();
    #[allow(clippy::clone_on_copy)]
    let node_cloned = node.0.clone();
    wasm_bindgen_futures::future_to_promise(async move {
        let header = ergo_lib::ergo_rest::api::node::get_header(node_cloned, header_id_cloned)
            .await
            .map_err(to_js)
            .map(BlockHeader::from)?;
        Ok(wasm_bindgen::JsValue::from(header))
    })
}

#[wasm_bindgen]
/// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
pub fn get_nipopow_proof_by_header_id(
    node: &NodeConf,
    min_chain_length: u32,
    suffix_len: u32,
    header_id: &BlockId,
) -> js_sys::Promise {
    let header_id_cloned = header_id.0.clone();
    #[allow(clippy::clone_on_copy)]
    let node_cloned = node.0.clone();
    wasm_bindgen_futures::future_to_promise(async move {
        let proof = ergo_lib::ergo_rest::api::node::get_nipopow_proof_by_header_id(
            node_cloned,
            min_chain_length,
            suffix_len,
            header_id_cloned,
        )
        .await
        .map_err(to_js)
        .map(NipopowProof::from)?;
        Ok(wasm_bindgen::JsValue::from(proof))
    })
}

#[wasm_bindgen]
/// GET on /blocks/{header_id}/proofFor/{tx_id} to request the merkle proof for a given transaction
/// that belongs to the given header ID.
pub fn get_blocks_header_id_proof_for_tx_id(
    node: &NodeConf,
    header_id: &BlockId,
    tx_id: &TxId,
) -> js_sys::Promise {
    let header_id_cloned = header_id.0.clone();
    let tx_id_cloned = tx_id.0.clone();
    #[allow(clippy::clone_on_copy)]
    let node_cloned = node.0.clone();
    wasm_bindgen_futures::future_to_promise(async move {
        let merkle_proof = ergo_lib::ergo_rest::api::node::get_blocks_header_id_proof_for_tx_id(
            node_cloned,
            header_id_cloned,
            tx_id_cloned,
        )
        .await
        .map_err(to_js)
        .map(|m| m.map(crate::merkleproof::MerkleProof))?;
        Ok(wasm_bindgen::JsValue::from(merkle_proof))
    })
}

/// List of peer urls returned from `peer_discovery`. We need this wrapper struct because the
/// `wasm_bindgen` macro currently cannot deal with `Result<Box<[T]>, JsValue>`, for any value `T`
/// that can be converted into a `JsValue` (`Result<Box<[web_sys::Url]>, JsValue>` would be a
/// convenient return type for `peer_discovery`).
#[wasm_bindgen]
pub struct PeerUrls(pub(crate) Vec<web_sys::Url>);

#[wasm_bindgen]
impl PeerUrls {
    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> web_sys::Url {
        self.0[index].clone()
    }
}

/// Given a list of seed nodes, search for peer nodes with an active REST API on port 9053.
///  - `seeds` represents a list of ergo node URLs from which to start peer discovery.
///  - `max_parallel_tasks` represents the maximum number of tasks to spawn for ergo node HTTP
///    requests. Note that the actual number of parallel HTTP requests may well be higher than this
///    number.
///  - `timeout` represents the amount of time that is spent search for peers. Once the timeout
///    value is reached, return with the vec of active peers that have been discovered up to that
///    point in time.
///  - `is_chrome` **MUST** be set to true if running this function on a Chromium-based browser.
///    There are some limitations on this platform regarding network requests. Please see the
///    documentation for [`peer_discovery_chrome`].
#[wasm_bindgen]
pub async fn peer_discovery(
    seeds: Box<[web_sys::Url]>,
    max_parallel_requests: u16,
    timeout_sec: u32,
    is_chrome: bool,
) -> Result<PeerUrls, JsValue> {
    if is_chrome {
        peer_discovery_chrome(seeds, max_parallel_requests, timeout_sec).await
    } else {
        peer_discovery_non_chrome(seeds, max_parallel_requests, timeout_sec).await
    }
}

/// IMPORTANT: do not call this function on Chromium, as it will likely mess with the browser's
/// ability to make HTTP requests. Use `peer_discovery_chrome` instead.
async fn peer_discovery_non_chrome(
    seeds: Box<[web_sys::Url]>,
    max_parallel_requests: u16,
    timeout_sec: u32,
) -> Result<PeerUrls, JsValue> {
    let mut converted_seeds = vec![];
    for seed in &*seeds {
        let str: String = seed.to_string().into();
        converted_seeds.push(url::Url::parse(&str).map_err(to_js)?);
    }
    let seeds = NonEmptyVec::from_vec(converted_seeds).map_err(to_js)?;
    let n = u16::max(max_parallel_requests, 1);
    #[allow(clippy::unwrap_used)]
    let max_parallel_requests = bounded_integer::BoundedU16::new(n).unwrap();
    let timeout = Duration::from_secs(timeout_sec as u64);
    let res = ergo_lib::ergo_rest::api::node::peer_discovery(seeds, max_parallel_requests, timeout)
        .await
        .map_err(to_js)?;
    let mut peer_urls = vec![];
    for mut url in res {
        #[allow(clippy::unwrap_used)]
        url.set_port(Some(9053)).unwrap();
        peer_urls.push(web_sys::Url::new(url.as_str())?);
    }
    Ok(PeerUrls(peer_urls))
}

/// Given a list of seed nodes, search for peer nodes with an active REST API on port 9053.
///  - `seeds` represents a list of ergo node URLs from which to start peer discovery.
///  - `max_parallel_requests` represents the maximum number of HTTP requests that can be made in
///    parallel. It's not possible to give a definitive upper bound 
///  - `timeout` represents the amount of time that is spent searching for peers PLUS a waiting
///    period of 80 seconds to give Chrome the time to relinquish failed preflight requests. Must be
///    at least 90 seconds. Once the timeout value is reached, return with the vec of active peers
///    that have been discovered up to that point in time.
///
/// NOTE: intended to be used only on Chromium based browsers. It works on Firefox and Safari, but
/// using `peer_discovery` above gives better performance. Why? See below.
/// 
/// ## Technical details
/// Chrome has a problem with hanging on to [`preflight`] requests where the server does not
/// respond. Every browser request we make of an ergo node will be preceded by a preflight request.
/// These preflight requests are automatically initiated by the browser, and it is not possible for
/// us to interact with it.
//
/// If a server doesn't respond to any request, then Chrome waits on the preflight request from
/// anywhere from 1.3 to 2.4 minutes. Chrome also doesn't have a high ceiling for parallel requests
/// and running our first implementation of `peer_discovery` in the `non_chrome` submodule brings
/// Chrome to a halt, preventing it from making any further requests until enough of the preflights
/// have been marked as failed. This is quite unfortunate as Firefox and Safari almost immediately
/// drops such preflight requests. See [`this issue`] for more discussion and example runs.
//
/// So this implementation is largely the same as in `non_chrome` except that we throttle the number
/// of parallel requests made. The majority of ergo nodes do not respond to REST requests, so there
/// will be a large number of preflight requests 'taking up space' and dramatically reducing
/// throughput.
pub async fn peer_discovery_chrome(
    seeds: Box<[web_sys::Url]>,
    max_parallel_requests: u16,
    timeout_sec: u32,
) -> Result<PeerUrls, JsValue> {
    let mut converted_seeds = vec![];
    for seed in &*seeds {
        let str: String = seed.to_string().into();
        converted_seeds.push(url::Url::parse(&str).map_err(to_js)?);
    }
    let seeds = NonEmptyVec::from_vec(converted_seeds).map_err(to_js)?;
    let n = u16::max(max_parallel_requests, 1);
    #[allow(clippy::unwrap_used)]
    let max_parallel_requests = bounded_integer::BoundedU16::new(n).unwrap();
    let timeout = Duration::from_secs(timeout_sec as u64);
    let res = ergo_lib::ergo_rest::api::node::peer_discovery_chrome(
        seeds,
        max_parallel_requests,
        timeout,
    )
    .await
    .map_err(to_js)?;
    let mut peer_urls = vec![];
    for mut url in res {
        #[allow(clippy::unwrap_used)]
        url.set_port(Some(9053)).unwrap();
        peer_urls.push(web_sys::Url::new(url.as_str())?);
    }
    Ok(PeerUrls(peer_urls))
}

/// An incremental (reusable) version of [`peer_discovery_chrome`] which allows for peer discovery
/// to be split into separate sub-tasks.
///
/// NOTE: intended to be used only on Chromium based browsers. It works on Firefox and Safari, but
/// using `peer_discovery` above gives better performance.
#[wasm_bindgen]
pub fn incremental_peer_discovery_chrome(
    scan: &super::chrome_peer_discovery_scan::ChromePeerDiscoveryScan,
    max_parallel_requests: u16,
    timeout_sec: u32,
) -> js_sys::Promise {
    let scan_cloned = scan.clone();
    wasm_bindgen_futures::future_to_promise(async move {
        let n = u16::max(max_parallel_requests, 1);
        #[allow(clippy::unwrap_used)]
        let max_parallel_requests = bounded_integer::BoundedU16::new(n).unwrap();
        let timeout = Duration::from_secs(timeout_sec as u64);
        let updated_scan = ergo_lib::ergo_rest::api::node::incremental_peer_discovery_chrome(
            scan_cloned.0,
            max_parallel_requests,
            timeout,
        )
        .await
        .map_err(to_js)
        .map(super::chrome_peer_discovery_scan::ChromePeerDiscoveryScan::from)?;
        Ok(wasm_bindgen::JsValue::from(updated_scan))
    })
}
