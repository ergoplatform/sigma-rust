//! Wasm API for ergo_rest::api

use wasm_bindgen::prelude::*;

use super::node_conf::NodeConf;
use crate::{block_header::BlockId, error_conversion::to_js, nipopow::NipopowProof};

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
