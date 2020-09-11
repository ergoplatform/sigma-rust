use wasm_bindgen::prelude::*;

use crate::{
    box_coll::ErgoBoxes, ergo_state_ctx::ErgoStateContext, secret_key::SecretKey,
    transaction::Transaction, transaction::UnsignedTransaction,
};

/// TBD
#[wasm_bindgen]
pub struct Wallet(sigma_tree::wallet::Wallet);

#[wasm_bindgen]
impl Wallet {
    /// Create wallet instance loading secret key from mnemonic
    #[wasm_bindgen]
    pub fn from_mnemonic(_mnemonic_phrase: &str, _mnemonic_pass: &str) -> Wallet {
        todo!()
    }

    #[wasm_bindgen]
    pub fn from_secret(secret: SecretKey) -> Wallet {
        Wallet(sigma_tree::wallet::Wallet::from_secrets(
            vec![secret.into()],
        ))
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
        self.0
            .sign_transaction(
                tx.into(),
                boxes_to_spend.into().as_slice(),
                data_boxes.into().as_slice(),
                &_state_context.into(),
            )
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(Transaction::from)
    }
}
