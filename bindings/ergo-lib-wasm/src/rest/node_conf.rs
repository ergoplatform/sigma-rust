//! Wasm API for ergo_rest::NodeConf
use std::str::FromStr;

use ergo_lib::ergo_chain_types::PeerAddr;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::error_conversion::to_js;

/// Node configuration
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, Copy, From, Into)]
pub struct NodeConf(ergo_lib::ergo_rest::NodeConf);

#[wasm_bindgen]
impl NodeConf {
    /// Create a node configuration
    /// addr - a string in a format 'ip_address:port'
    #[wasm_bindgen(constructor)]
    pub fn new(addr: &str) -> Result<NodeConf, JsValue> {
        let peer_addr = PeerAddr::from_str(addr).map_err(to_js)?;
        Ok(ergo_lib::ergo_rest::NodeConf {
            addr: peer_addr,
            api_key: None,
        }
        .into())
    }
}
