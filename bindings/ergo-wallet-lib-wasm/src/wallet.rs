use wasm_bindgen::prelude::*;

use crate::{
    box_coll::ErgoBoxes, ergo_state_ctx::ErgoStateContext, transaction::Transaction,
    transaction::UnsignedTransaction,
};

/// TBD
#[wasm_bindgen]
pub struct Wallet();

#[wasm_bindgen]
impl Wallet {
    /// Create wallet instance loading secret key from mnemonic
    pub fn from_mnemonic(_mnemonic_phrase: &str, _mnemonic_pass: &str) -> Wallet {
        Wallet()
    }

    /// Sign a transaction:
    /// `boxes_to_spend` - unspent boxes [`ErgoBoxCandidate`] used as inputs in the transaction
    #[wasm_bindgen]
    pub fn sign_transaction(
        &self,
        _state_context: ErgoStateContext,
        tx: UnsignedTransaction,
        boxes_to_spend: ErgoBoxes,
        data_boxes: ErgoBoxes,
    ) -> Result<Transaction, JsValue> {
        // not implemented, see https://github.com/ergoplatform/sigma-rust/issues/34
        Err(JsValue::from_str("Not yet implemented"))
    }
}
