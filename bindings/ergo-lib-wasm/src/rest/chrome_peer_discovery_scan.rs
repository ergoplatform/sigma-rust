//! A Chrome peer discovery scan stores the results of potentially-multiple calls to
//! `peer_discovery_chrome`. It allows the user the ability to break up peer discovery into smaller
//! sub-tasks.

use super::api::PeerUrls;
use crate::error_conversion::to_js;
use bounded_vec::NonEmptyVec;
use derive_more::{From, Into};
use wasm_bindgen::prelude::*;

/// Node info
#[wasm_bindgen]
#[derive(Debug, Clone, From, Into)]
pub struct ChromePeerDiscoveryScan(
    pub(crate) ergo_lib::ergo_rest::api::node::ChromePeerDiscoveryScan,
);

#[wasm_bindgen]
impl ChromePeerDiscoveryScan {
    /// Create new scan. Note: `seeds` must not be empty.
    #[wasm_bindgen(constructor)]
    pub fn new(seeds: Box<[web_sys::Url]>) -> Result<ChromePeerDiscoveryScan, JsValue> {
        let mut converted_seeds = vec![];
        for seed in &*seeds {
            let str: String = seed.to_string().into();
            converted_seeds.push(url::Url::parse(&str).map_err(to_js)?);
        }
        let seeds = NonEmptyVec::from_vec(converted_seeds).map_err(to_js)?;
        Ok(ergo_lib::ergo_rest::api::node::ChromePeerDiscoveryScan::new(seeds).into())
    }

    /// Returns list of non-seed peers with an active REST API.
    pub fn active_peers(&self) -> Result<PeerUrls, JsValue> {
        let mut peer_urls = vec![];
        for mut url in self.0.active_peers() {
            url.set_port(Some(9053)).unwrap();
            peer_urls.push(web_sys::Url::new(url.as_str())?);
        }
        Ok(PeerUrls(peer_urls))
    }
}
