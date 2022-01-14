//! Wasm API for ergo_rest::api

use std::time::Duration;

use wasm_bindgen::prelude::*;

use super::node_conf::NodeConf;
use crate::error_conversion::to_js;

#[wasm_bindgen]
/// GET on /info endpoint
pub async fn get_info(node: NodeConf, timeout_sec: i32) -> Result<JsValue, JsValue> {
    // TODO: check if node is not null in JS after the call (because it implements Copy)
    ergo_lib::ergo_rest::api::node::get_info(node.into(), Duration::from_secs(timeout_sec as u64))
        .await
        .map_err(to_js)
        .map(|info| JsValue::from_str(&info.name))
}
