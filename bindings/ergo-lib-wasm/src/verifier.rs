//! Verifier

use std::convert::TryFrom;

use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use wasm_bindgen::{prelude::*, JsValue};

use crate::{address::Address, error_conversion::to_js};

/// Verify that the signature is presented to satisfy SigmaProp conditions.
#[wasm_bindgen]
pub fn verify_signature(
    address: &Address,
    message: &[u8],
    signature: &[u8],
) -> Result<bool, JsValue> {
    if let Address(ergo_lib::ergotree_ir::chain::address::Address::P2Pk(d)) = address.clone() {
        let sb = SigmaBoolean::try_from(d).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        ergo_lib::ergotree_interpreter::sigma_protocol::verifier::verify_signature(
            sb, message, signature,
        )
        .map_err(to_js)
    } else {
        Err(JsValue::from_str(
            "wallet::verify_signature: Address:P2Pk expected",
        ))
    }
}
