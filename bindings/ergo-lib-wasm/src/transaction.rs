//! Ergo transaction

use crate::box_coll::ErgoBoxCandidates;
use crate::box_coll::ErgoBoxes;
use crate::context_extension::ContextExtension;
use crate::data_input::DataInputs;
use crate::ergo_box::BoxId;
use crate::ergo_box::ErgoBox;
use crate::error_conversion::to_js;
use crate::input::{Inputs, UnsignedInputs};
use crate::json::TransactionJsonEip12;
use crate::json::UnsignedTransactionJsonEip12;
use ergo_lib::chain;
use ergo_lib::chain::transaction::{distinct_token_ids, TxIoVec};
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Uint8Array;
use std::convert::{TryFrom, TryInto};
use wasm_bindgen::prelude::*;

extern crate derive_more;

use crate::ergo_state_ctx::ErgoStateContext;
use crate::transaction::reduced::Propositions;
use derive_more::{From, Into};
use ergo_lib::ergo_chain_types::{Base16DecodedBytes, Base16EncodedBytes, Digest32};

pub mod reduced;

/// CommitmentHint
#[wasm_bindgen]
pub struct CommitmentHint(
    ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::CommitmentHint,
);

/// HintsBag
#[wasm_bindgen]
pub struct HintsBag(
    pub(crate) ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::HintsBag,
);

#[wasm_bindgen]
impl HintsBag {
    /// Empty HintsBag
    pub fn empty() -> HintsBag {
        HintsBag(ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::HintsBag::empty())
    }

    /// Add commitment hint to the bag
    pub fn add_commitment(&mut self, hint: CommitmentHint) {
        self.0.add_hint(
            ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::Hint::CommitmentHint(
                hint.0,
            ),
        );
    }

    /// Length of HintsBag
    pub fn len(&self) -> usize {
        self.0.hints.len()
    }

    /// Get commitment
    pub fn get(&self, index: usize) -> Result<CommitmentHint, JsValue> {
        let commitment = self.0.commitments()[index].clone();
        Ok(CommitmentHint(commitment))
    }
}

impl From<ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::HintsBag> for HintsBag {
    fn from(t: ergo_lib::ergotree_interpreter::sigma_protocol::prover::hint::HintsBag) -> Self {
        HintsBag(t)
    }
}

/// TransactionHintsBag
#[wasm_bindgen]
pub struct TransactionHintsBag(pub(crate) ergo_lib::wallet::multi_sig::TransactionHintsBag);

#[wasm_bindgen]
impl TransactionHintsBag {
    /// Empty TransactionHintsBag
    pub fn empty() -> TransactionHintsBag {
        TransactionHintsBag(ergo_lib::wallet::multi_sig::TransactionHintsBag::empty())
    }

    /// Adding hints for input
    pub fn add_hints_for_input(&mut self, index: usize, hints_bag: &HintsBag) {
        self.0.add_hints_for_input(index, hints_bag.0.clone());
    }

    /// Outputting HintsBag corresponding for an input index
    pub fn all_hints_for_input(&self, index: usize) -> HintsBag {
        HintsBag::from(self.0.all_hints_for_input(index))
    }

    /// Return JSON object (node format)
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        <JsValue as JsValueSerdeExt>::from_serde(&self.0).map_err(to_js)
    }

    /// Parse from JSON object (node format)
    pub fn from_json(json: &str) -> Result<TransactionHintsBag, JsValue> {
        serde_json::from_str(json).map(Self).map_err(to_js)
    }
}

impl From<ergo_lib::wallet::multi_sig::TransactionHintsBag> for TransactionHintsBag {
    fn from(t: ergo_lib::wallet::multi_sig::TransactionHintsBag) -> Self {
        TransactionHintsBag(t)
    }
}

/// Extracting hints form singed(invalid) Transaction
#[wasm_bindgen]
pub fn extract_hints(
    signed_transaction: Transaction,
    state_context: &ErgoStateContext,
    boxes_to_spend: &ErgoBoxes,
    data_boxes: &ErgoBoxes,
    real_propositions: Propositions,
    simulated_propositions: Propositions,
) -> Result<TransactionHintsBag, JsValue> {
    let boxes_to_spend = boxes_to_spend.clone().into();

    let data_boxes = data_boxes.clone().into();
    let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
        signed_transaction.0,
        boxes_to_spend,
        data_boxes,
    )
    .map_err(to_js)?;
    Ok(TransactionHintsBag::from(
        ergo_lib::wallet::multi_sig::extract_hints(
            &tx_context,
            &state_context.0.clone(),
            real_propositions.0,
            simulated_propositions.0,
        )
        .map_err(to_js)?,
    ))
}

/// Transaction id
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct TxId(pub(crate) chain::transaction::TxId);

#[wasm_bindgen]
impl TxId {
    /// Zero (empty) transaction id (to use as dummy value in tests)
    pub fn zero() -> TxId {
        chain::transaction::TxId::zero().into()
    }

    /// get the tx id as bytes
    pub fn to_str(&self) -> String {
        let base16_bytes = Base16EncodedBytes::new(self.0 .0 .0.as_ref());
        base16_bytes.into()
    }

    /// convert a hex string into a TxId
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<TxId, JsValue> {
        let bytes = Base16DecodedBytes::try_from(s.to_string()).map_err(to_js)?;

        bytes
            .try_into()
            .map(|digest| chain::transaction::TxId(digest).into())
            .map_err(|_e| {
                JsValue::from_str(&format!(
                    "Expected a Vec of length {} but it was {}",
                    Digest32::SIZE,
                    s.len()
                ))
            })
    }
}

/**
 * ErgoTransaction is an atomic state transition operation. It destroys Boxes from the state
 * and creates new ones. If transaction is spending boxes protected by some non-trivial scripts,
 * its inputs should also contain proof of spending correctness - context extension (user-defined
 * key-value map) and data inputs (links to existing boxes in the state) that may be used during
 * script reduction to crypto, signatures that satisfies the remaining cryptographic protection
 * of the script.
 * Transactions are not encrypted, so it is possible to browse and view every transaction ever
 * collected into a block.
 */
#[wasm_bindgen]
pub struct Transaction(chain::transaction::Transaction);

#[wasm_bindgen]
impl Transaction {
    /// Create new transaction
    #[wasm_bindgen(constructor)]
    pub fn new(
        inputs: &Inputs,
        data_inputs: &DataInputs,
        outputs: &ErgoBoxCandidates,
    ) -> Result<Transaction, JsValue> {
        let inputs: Vec<chain::transaction::Input> = inputs.into();
        let data_inputs: Vec<chain::transaction::DataInput> = data_inputs.into();
        let outputs: Vec<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBoxCandidate> =
            outputs.clone().into();
        Ok(Transaction(
            chain::transaction::Transaction::new(
                TxIoVec::try_from(inputs).map_err(to_js)?,
                TxIoVec::try_from(data_inputs).map_err(to_js)?.into(),
                TxIoVec::try_from(outputs).map_err(to_js)?,
            )
            .map_err(to_js)?,
        ))
    }

    /// Create Transaction from UnsignedTransaction and an array of proofs in the same order as
    /// UnsignedTransaction.inputs with empty proof indicated with empty byte array
    pub fn from_unsigned_tx(
        unsigned_tx: UnsignedTransaction,
        proofs: Vec<Uint8Array>,
    ) -> Result<Transaction, JsValue> {
        chain::transaction::Transaction::from_unsigned_tx(
            unsigned_tx.0,
            proofs
                .into_iter()
                .map(|bytes| bytes.to_vec().into())
                .collect(),
        )
        .map_err(|e| JsValue::from_str(&format!("{}", e)))
        .map(Into::into)
    }

    /// Get id for transaction
    pub fn id(&self) -> TxId {
        self.0.id().into()
    }

    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string_pretty(&self.0.clone())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
    /// (similar to [`Self::to_json`], but as JS object with box value and token amount encoding as strings)
    pub fn to_js_eip12(&self) -> Result<JsValue, JsValue> {
        let tx_dapp: TransactionJsonEip12 = self.0.clone().into();
        <JsValue as JsValueSerdeExt>::from_serde(&tx_dapp)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// parse from JSON
    /// supports Ergo Node/Explorer API and box values and token amount encoded as strings
    pub fn from_json(json: &str) -> Result<Transaction, JsValue> {
        serde_json::from_str(json).map(Self).map_err(to_js)
    }

    /// Inputs for transaction
    pub fn inputs(&self) -> Inputs {
        self.0.inputs.as_vec().clone().into()
    }

    /// Data inputs for transaction
    pub fn data_inputs(&self) -> DataInputs {
        self.0
            .data_inputs
            .clone()
            .map(|di| di.as_vec().clone())
            .unwrap_or_default()
            .into()
    }

    /// Output candidates for transaction
    pub fn output_candidates(&self) -> ErgoBoxCandidates {
        self.0.output_candidates.as_vec().clone().into()
    }

    /// Returns ErgoBox's created from ErgoBoxCandidate's with tx id and indices
    pub fn outputs(&self) -> ErgoBoxes {
        self.0.outputs.clone().to_vec().into()
    }

    /// Returns serialized bytes or fails with error if cannot be serialized
    pub fn sigma_serialize_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.0.sigma_serialize_bytes().map_err(to_js)
    }

    /// Parses Transaction or fails with error
    pub fn sigma_parse_bytes(data: Vec<u8>) -> Result<Transaction, JsValue> {
        ergo_lib::chain::transaction::Transaction::sigma_parse_bytes(&data)
            .map(Transaction)
            .map_err(to_js)
    }

    /// Check the signature of the transaction's input corresponding
    /// to the given input box, guarded by P2PK script
    pub fn verify_p2pk_input(&self, input_box: ErgoBox) -> Result<bool, JsValue> {
        self.0.verify_p2pk_input(input_box.into()).map_err(to_js)
    }
}

impl From<chain::transaction::Transaction> for Transaction {
    fn from(t: chain::transaction::Transaction) -> Self {
        Transaction(t)
    }
}

/// Unsigned (inputs without proofs) transaction
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct UnsignedTransaction(pub(crate) chain::transaction::unsigned::UnsignedTransaction);

#[wasm_bindgen]
impl UnsignedTransaction {
    /// Create a new unsigned transaction
    #[wasm_bindgen(constructor)]
    pub fn new(
        inputs: &UnsignedInputs,
        data_inputs: &DataInputs,
        output_candidates: &ErgoBoxCandidates,
    ) -> Result<UnsignedTransaction, JsValue> {
        let opt_data_input = if data_inputs.len() > 0 {
            Some(TxIoVec::from_vec(data_inputs.into()).map_err(to_js)?)
        } else {
            None
        };

        Ok(chain::transaction::unsigned::UnsignedTransaction::new(
            TxIoVec::from_vec(inputs.into()).map_err(to_js)?,
            opt_data_input,
            TxIoVec::from_vec(output_candidates.into()).map_err(to_js)?,
        )
        .map_err(to_js)?
        .into())
    }

    /// Consumes the calling UnsignedTransaction and returns a new UnsignedTransaction containing
    /// the ContextExtension in the provided input box id or returns an error if the input box cannot be found.
    /// After the call the calling UnsignedTransaction will be null.
    pub fn with_input_context_ext(
        mut self,
        input_id: &BoxId,
        ext: &ContextExtension,
    ) -> Result<UnsignedTransaction, JsValue> {
        let input = self
            .0
            .inputs
            .iter_mut()
            .find(|input| input.box_id == input_id.clone().into())
            .ok_or_else(|| JsValue::from_str("Box input id not found"))?;
        input.extension = ext.clone().into();

        Ok(self.clone())
    }

    /// Get id for transaction
    pub fn id(&self) -> TxId {
        self.0.id().into()
    }

    /// Inputs for transaction
    pub fn inputs(&self) -> UnsignedInputs {
        self.0.inputs.as_vec().clone().into()
    }

    /// Data inputs for transaction
    pub fn data_inputs(&self) -> DataInputs {
        self.0
            .clone()
            .data_inputs
            .map(|di| di.as_vec().clone())
            .unwrap_or_default()
            .into()
    }

    /// Output candidates for transaction
    pub fn output_candidates(&self) -> ErgoBoxCandidates {
        self.0.output_candidates.as_vec().clone().into()
    }

    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string_pretty(&self.0.clone())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
    /// (similar to [`Self::to_json`], but as JS object with box value and token amount encoding as strings)
    pub fn to_js_eip12(&self) -> Result<JsValue, JsValue> {
        let tx_dapp: UnsignedTransactionJsonEip12 = self.0.clone().into();
        <JsValue as JsValueSerdeExt>::from_serde(&tx_dapp)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// parse from JSON
    /// supports Ergo Node/Explorer API and box values and token amount encoded as strings
    pub fn from_json(json: &str) -> Result<UnsignedTransaction, JsValue> {
        serde_json::from_str(json).map(Self).map_err(to_js)
    }

    /// Returns distinct token id from output_candidates as array of byte arrays
    pub fn distinct_token_ids(&self) -> Vec<Uint8Array> {
        distinct_token_ids(self.0.output_candidates.clone())
            .iter()
            .map(|id| Uint8Array::from(id.as_ref()))
            .collect()
    }
}

/// Verify transaction input's proof
#[wasm_bindgen]
pub fn verify_tx_input_proof(
    input_idx: usize,
    state_context: &ErgoStateContext,
    tx: &Transaction,
    boxes_to_spend: &ErgoBoxes,
    data_boxes: &ErgoBoxes,
) -> Result<bool, JsValue> {
    let boxes_to_spend = boxes_to_spend.clone().into();
    let data_boxes = data_boxes.clone().into();
    let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
        tx.0.clone(),
        boxes_to_spend,
        data_boxes,
    )
    .map_err(to_js)?;
    let state_context_inner = state_context.clone().into();
    Ok(ergo_lib::chain::transaction::verify_tx_input_proof(
        &tx_context,
        &state_context_inner,
        input_idx,
        &tx_context.spending_tx.bytes_to_sign().map_err(to_js)?,
    )
    .map_err(to_js)?
    .result)
}

/// Verify transaction
#[wasm_bindgen]
pub fn validate_tx(
    tx: &Transaction,
    state_context: &ErgoStateContext,
    boxes_to_spend: &ErgoBoxes,
    data_boxes: &ErgoBoxes,
) -> Result<(), JsValue> {
    let boxes_to_spend = boxes_to_spend.clone().into();
    let data_boxes = data_boxes.clone().into();
    let tx_context = ergo_lib::wallet::signing::TransactionContext::new(
        tx.0.clone(),
        boxes_to_spend,
        data_boxes,
    )
    .map_err(to_js)?;
    tx_context.validate(&state_context.0).map_err(to_js)
}
