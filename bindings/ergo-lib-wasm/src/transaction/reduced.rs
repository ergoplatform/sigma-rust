//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use super::UnsignedTransaction;
use crate::box_coll::ErgoBoxes;
use crate::ergo_state_ctx::ErgoStateContext;
use crate::error_conversion::to_js;

use ergo_lib::chain::transaction::reduced::reduce_tx;
use ergo_lib::chain::transaction::TxIoVec;
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use wasm_bindgen::prelude::*;
use ergo_lib::chain::json::hints::CommitmentHintJson;
use ergo_lib::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::{Hint};
use ergo_lib::ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use ergo_lib::wallet::multi_sig::{bag_for_multi_sig, generate_commitments_for};
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree};
use crate::transaction::{HintsBag, Transaction};
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog as OtherProveDlog;
// use crate::address::Address;
use crate::transaction::CommitmentHintJson as CommitmentHintJsonWasm;
use ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::HintsBag as OtherHintsBag;

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

    /// Getting first input box serialized sigma prop bytes
    pub fn get_first_input_serialized_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0.reduced_inputs().first().clone().reduction_result.sigma_prop.sigma_serialize_bytes().map_err(to_js)
    }

    /// Generate commitment for first input with input address
    pub fn generate_commitment_for_first_input(&self, secret_base16:&str)->Result<String, JsValue>{
        let sigma_prop=self.0.reduced_inputs().first().clone().reduction_result.sigma_prop;
        let secret=DlogProverInput::from_base16_str(secret_base16.to_string()).unwrap();
        let pk=secret.public_image();
        let generate_for:Vec<SigmaBoolean>=vec![SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pk.clone()))];
        let hints=generate_commitments_for(sigma_prop,&generate_for);
        let mut commitments:Vec<CommitmentHintJson>=Vec::new();
        match hints.hints[0].clone(){
            Hint::SecretProven(_) => {}
            Hint::CommitmentHint(cmt) => {
                let cmt_json:CommitmentHintJson=CommitmentHintJson::from(cmt);
                commitments.push(cmt_json);
            }
        }
        match hints.hints[1].clone(){
            Hint::SecretProven(_) => {}
            Hint::CommitmentHint(cmt) => {
                let cmt_json:CommitmentHintJson=CommitmentHintJson::from(cmt);
                commitments.push(cmt_json);
            }
        }
        serde_json::to_string_pretty(&commitments)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// bag for multi sig
    pub fn bag_for_multi_sig(&self,pk_base16:&str,own_cmt:&str, signed_transaction:Transaction)->HintsBag{
        let sigma_prop=self.0.reduced_inputs().first().clone().reduction_result.sigma_prop;
        let mut real_proposition:Vec<SigmaBoolean>=Vec::new();
        let simulated_proposition:Vec<SigmaBoolean>=Vec::new();
        let pks:Vec<&str>=pk_base16.split(',').collect();
        for pk in pks{
            real_proposition.push(SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(OtherProveDlog::from(EcPoint::from_base16_str(pk.to_string()).unwrap()))))
        }
        let proof=Vec::from(signed_transaction.0.inputs.get(0).unwrap().spending_proof.clone().proof);
        let mut bag:OtherHintsBag=bag_for_multi_sig(sigma_prop,&real_proposition,&simulated_proposition,&proof);
        let own=CommitmentHintJsonWasm::from_json(own_cmt);
        bag.add_hint(Hint::CommitmentHint(own.0));
        crate::transaction::HintsBag{0:bag}

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
