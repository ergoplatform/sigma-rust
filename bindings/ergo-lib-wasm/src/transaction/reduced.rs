//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use super::UnsignedTransaction;
use crate::box_coll::ErgoBoxes;
use crate::ergo_state_ctx::ErgoStateContext;
use crate::error_conversion::to_js;

use ergo_lib::chain::transaction::reduced::reduce_tx;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use wasm_bindgen::prelude::*;

/// Propositions list(public keys)
#[wasm_bindgen]
pub struct Propositions(pub(crate) Vec<SigmaBoolean>);

#[wasm_bindgen]
impl Propositions {
    /// Create empty proposition holder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Propositions(vec![])
    }

    /// Adding new proposition
    pub fn add_proposition_from_byte(&mut self, proposition: Vec<u8>) -> Result<(), JsValue> {
        self.0
            .push(SigmaBoolean::sigma_parse_bytes(&proposition).map_err(to_js)?);
        Ok(())
    }
}

/// Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
/// After an unsigned transaction is reduced it can be signed without context.
/// Thus, it can be serialized and transferred for example to Cold Wallet and signed
/// in an environment where secrets are known.
/// see EIP-19 for more details -
/// <https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md>
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
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
        let boxes_to_spend = boxes_to_spend.clone().into();
        let data_boxes = data_boxes.clone().into();
        let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
            unsigned_tx.0.clone(),
            boxes_to_spend,
            data_boxes,
        )
        .map_err(to_js)?;
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

    /// Returns the unsigned transaction
    pub fn unsigned_tx(&self) -> UnsignedTransaction {
        self.0.unsigned_tx.clone().into()
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
