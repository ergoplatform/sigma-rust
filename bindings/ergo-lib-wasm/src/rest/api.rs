//! Wasm API for ergo_rest::api

use wasm_bindgen::prelude::*;

use super::node_conf::NodeConf;
use crate::error_conversion::to_js;

#[wasm_bindgen]
/// GET on /info endpoint
pub async fn get_info(node: NodeConf) -> Result<JsValue, JsValue> {
    // TODO: check if node is null in JS after the call
    ergo_lib::ergo_rest::api::get_info(node.into())
        .await
        .map_err(to_js)
        .map(|info| JsValue::from_str(&info.name))
}
