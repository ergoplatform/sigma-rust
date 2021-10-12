//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use super::UnsignedTransaction;
use crate::box_coll::ErgoBoxes;
use crate::ergo_state_ctx::ErgoStateContext;
use crate::error_conversion::to_js;

use ergo_lib::chain::transaction::reduced::reduce_tx;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use wasm_bindgen::prelude::*;

/// Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
/// After an unsigned transaction is reduced it can be signed without context.
/// Thus, it can be serialized and transferred for example to Cold Wallet and signed
/// in an environment where secrets are known.
/// see EIP-19 for more details -
/// <https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md>
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone)]
pub struct ReducedTransaction(ergo_lib::chain::transaction::reduced::ReducedTransaction);

#[wasm_bindgen]
impl ReducedTransaction {
    /// Returns `reduced` transaction, i.e. unsigned transaction where each unsigned input
    /// is augmented with ReducedInput which contains a script reduction result.
    pub fn from_unsigned_tx(
        unsigned_tx: &UnsignedTransaction,
        boxes_to_spend: &ErgoBoxes,
        data_boxes: &ErgoBoxes,
        state_context: &ErgoStateContext,
    ) -> Result<ReducedTransaction, JsValue> {
        let boxes_to_spend: Vec<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox> =
            boxes_to_spend.clone().into();
        let data_boxes: Vec<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox> =
            data_boxes.clone().into();
        let tx_context = ergo_lib::wallet::signing::TransactionContext {
            spending_tx: unsigned_tx.clone().into(),
            boxes_to_spend,
            data_boxes,
        };
        reduce_tx(tx_context, &state_context.clone().into())
            .map_err(to_js)
            .map(ReducedTransaction::from)
    }

    /// Returns serialized bytes or fails with error if cannot be serialized
    pub fn sigma_serialize_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0.sigma_serialize_bytes().map_err(to_js)
    }

    /// Parses ReducedTransaction or fails with error
    pub fn sigma_parse_bytes(data: Vec<u8>) -> Result<ReducedTransaction, JsValue> {
        ergo_lib::chain::transaction::reduced::ReducedTransaction::sigma_parse_bytes(&data)
            .map(ReducedTransaction)
            .map_err(to_js)
    }
}

impl From<ergo_lib::chain::transaction::reduced::ReducedTransaction> for ReducedTransaction {
    fn from(t: ergo_lib::chain::transaction::reduced::ReducedTransaction) -> Self {
        ReducedTransaction(t)
    }
}

impl From<ReducedTransaction> for ergo_lib::chain::transaction::reduced::ReducedTransaction {
    fn from(t: ReducedTransaction) -> Self {
        t.0
    }
}
