//! Wallet-like features
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

use crate::{
    box_coll::ErgoBoxes, ergo_state_ctx::ErgoStateContext, secret_key::SecretKeys,
    transaction::Transaction, transaction::UnsignedTransaction,
};

/// A collection of secret keys. This simplified signing by matching the secret keys to the correct inputs automatically.
#[wasm_bindgen]
pub struct Wallet(ergo_lib::wallet::Wallet);

#[wasm_bindgen]
impl Wallet {
    /// Create wallet instance loading secret key from mnemonic
    #[wasm_bindgen]
    pub fn from_mnemonic(_mnemonic_phrase: &str, _mnemonic_pass: &str) -> Wallet {
        todo!()
    }

    /// Create wallet using provided secret key
    #[wasm_bindgen]
    pub fn from_secrets(secret: &SecretKeys) -> Wallet {
        Wallet(ergo_lib::wallet::Wallet::from_secrets(
            secret.clone().into(),
        ))
    }

    /// Sign a transaction:
    /// `tx` - transaction to sign
    /// `boxes_to_spend` - boxes corresponding to [`UnsignedTransaction::inputs`]
    /// `data_boxes` - boxes corresponding to [`UnsignedTransaction::data_inputs`]
    #[wasm_bindgen]
    pub fn sign_transaction(
        &self,
        _state_context: &ErgoStateContext,
        tx: &UnsignedTransaction,
        boxes_to_spend: &ErgoBoxes,
        data_boxes: &ErgoBoxes,
    ) -> Result<Transaction, JsValue> {
        let boxes_to_spend: Vec<chain::ergo_box::ErgoBox> = boxes_to_spend.clone().into();
        let data_boxes: Vec<chain::ergo_box::ErgoBox> = data_boxes.clone().into();
        let tx_context = ergo_lib::wallet::signing::TransactionContext {
            spending_tx: tx.clone().into(),
            boxes_to_spend,
            data_boxes,
        };
        self.0
            .sign_transaction(tx_context, &_state_context.clone().into())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(Transaction::from)
    }
}
