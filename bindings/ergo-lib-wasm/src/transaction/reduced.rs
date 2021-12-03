//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use super::UnsignedTransaction;
use crate::box_coll::ErgoBoxes;
use crate::ergo_state_ctx::ErgoStateContext;
use crate::error_conversion::to_js;

use crate::transaction::{HintsBag, Transaction, TransactionHintsBag};
use ergo_lib::chain::transaction::reduced::reduce_tx;
use ergo_lib::chain::transaction::TxIoVec;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergo_lib::wallet::multi_sig::{
    extract_hints_from_reduced_transaction, generate_commitments_for,
};
use wasm_bindgen::prelude::*;

/// Propositions list(public keys)
#[wasm_bindgen]
pub struct Propositions(pub(crate) Vec<SigmaBoolean>);

#[wasm_bindgen]
impl Propositions {
    /// Create empty SecretKeys
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Propositions(vec![])
    }

    /// Adding new proposition
    pub fn add_proposition_from_byte(&mut self, proposition: Vec<u8>) {
        self.0.push(
            SigmaBoolean::sigma_parse_bytes(&proposition)
                .map_err(to_js)
                .unwrap(),
        );
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
        let boxes_to_spend = TxIoVec::from_vec(boxes_to_spend.clone().into()).map_err(to_js)?;
        let data_boxes = {
            let d: Vec<_> = data_boxes.clone().into();
            if d.is_empty() {
                None
            } else {
                Some(TxIoVec::from_vec(d).map_err(to_js)?)
            }
        };
        let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
            unsigned_tx.clone().into(),
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

    /// Generate commitment for reduced transaction for a public key
    pub fn generate_commitment(&self, public_key: Vec<u8>) -> Result<TransactionHintsBag, JsValue> {
        let mut tx_hints = TransactionHintsBag::empty();
        let pk = SigmaBoolean::sigma_parse_bytes(&public_key).map_err(to_js);
        let generate_for: Vec<SigmaBoolean> = vec![pk.unwrap()];
        for (index, input) in self.0.reduced_inputs().iter().enumerate() {
            let sigma_prop = input.clone().reduction_result.sigma_prop;
            let hints = HintsBag(generate_commitments_for(sigma_prop, &generate_for));
            tx_hints.add_hints_for_input(index, &hints);
        }
        Ok(tx_hints)
    }

    /// Extracting hints from transaction
    pub fn extract_hints(
        &self,
        real_propositions: Propositions,
        simulated_propositions: Propositions,
        signed_transaction: Transaction,
    ) -> TransactionHintsBag {
        TransactionHintsBag::from(extract_hints_from_reduced_transaction(
            self.0.clone(),
            signed_transaction.0,
            real_propositions.0,
            simulated_propositions.0,
        ))
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
