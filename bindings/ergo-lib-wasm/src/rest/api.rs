//! Wasm API for ergo_rest::api

use wasm_bindgen::prelude::*;

use super::node_conf::NodeConf;
use crate::{block_header::BlockId, error_conversion::to_js, nipopow::NipopowProof};
use bounded_vec::NonEmptyVec;
use std::time::Duration;

#[wasm_bindgen]
/// GET on /info endpoint
pub async fn get_info(node: NodeConf) -> Result<JsValue, JsValue> {
    // TODO: check if node is not null in JS after the call (because it implements Copy)
    ergo_lib::ergo_rest::api::node::get_info(node.into())
        .await
        .map_err(to_js)
        .map(|info| JsValue::from_str(&info.name))
}

#[wasm_bindgen]
/// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
pub async fn get_nipopow_proof_by_header_id(
    node: NodeConf,
    min_chain_length: u32,
    suffix_len: u32,
    header_id: BlockId,
) -> Result<NipopowProof, JsValue> {
    ergo_lib::ergo_rest::api::node::get_nipopow_proof_by_header_id(
        node.into(),
        min_chain_length,
        suffix_len,
        header_id.into(),
    )
    .await
    .map_err(to_js)
    .map(NipopowProof::from)
}

/// List of peer urls returned from `peer_discovery`. We need this wrapper struct because the
/// `wasm_bindgen` macro currently cannot deal with `Result<Box<[T]>, JsValue>`, for any value `T`
/// that can be converted into a `JsValue` (`Result<Box<[web_sys::Url]>, JsValue>` would be a
/// convenient return type for `peer_discovery`).
#[wasm_bindgen]
pub struct PeerUrls(Vec<web_sys::Url>);

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

#[wasm_bindgen]
/// Given a list of seed nodes, recursively determine a Vec of all known peer nodes.
pub async fn peer_discovery(
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
    for url in res {
        peer_urls.push(web_sys::Url::new(url.as_str())?);
    }
    Ok(PeerUrls(peer_urls))
}
