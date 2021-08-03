//! Ergo transaction

use crate::box_coll::ErgoBoxCandidates;
use crate::box_coll::ErgoBoxes;
use crate::data_input::DataInputs;
use crate::input::{Inputs, UnsignedInputs};
use ergo_lib::chain;
use std::convert::TryFrom;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// Transaction id
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct TxId(chain::transaction::TxId);

#[wasm_bindgen]
impl TxId {
    /// Zero (empty) transaction id (to use as dummy value in tests)
    pub fn zero() -> TxId {
        chain::transaction::TxId::zero().into()
    }

    /// get the tx id as bytes
    pub fn to_str(&self) -> String {
        let base16_bytes = ergo_lib::chain::Base16EncodedBytes::new(self.0 .0 .0.as_ref());
        base16_bytes.into()
    }

    /// convert a hex string into a TxId
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<TxId, JsValue> {
        let bytes = ergo_lib::chain::Base16DecodedBytes::try_from(s.to_string())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

        bytes
            .try_into()
            .map(|digest| chain::transaction::TxId(digest).into())
            .map_err(|_e| {
                JsValue::from_str(&format!(
                    "Expected a Vec of length {} but it was {}",
                    chain::Digest32::SIZE,
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
    /// Get id for transaction
    pub fn id(&self) -> TxId {
        self.0.id().into()
    }

    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0.clone()).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// JSON representation
    pub fn from_json(json: &str) -> Result<Transaction, JsValue> {
        serde_json::from_str(json)
            .map(Self)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Inputs for transaction
    pub fn inputs(&self) -> Inputs {
        self.0.inputs.as_vec().clone().into()
    }

    /// Data inputs for transaction
    pub fn data_inputs(&self) -> DataInputs {
        self.0.data_inputs.clone().into()
    }

    /// Output candidates for transaction
    pub fn output_candidates(&self) -> ErgoBoxCandidates {
        self.0.output_candidates.as_vec().clone().into()
    }

    /// Returns ErgoBox's created from ErgoBoxCandidate's with tx id and indices
    pub fn outputs(&self) -> ErgoBoxes {
        self.0.outputs.clone().into()
    }
}

impl From<chain::transaction::Transaction> for Transaction {
    fn from(t: chain::transaction::Transaction) -> Self {
        Transaction(t)
    }
}

/// Unsigned (inputs without proofs) transaction
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedTransaction(chain::transaction::unsigned::UnsignedTransaction);

#[wasm_bindgen]
impl UnsignedTransaction {
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
        self.0.data_inputs.clone().into()
    }

    /// Output candidates for transaction
    pub fn output_candidates(&self) -> ErgoBoxCandidates {
        self.0.output_candidates.as_vec().clone().into()
    }

    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0.clone()).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// JSON representation
    pub fn from_json(json: &str) -> Result<UnsignedTransaction, JsValue> {
        serde_json::from_str(json)
            .map(Self)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

impl From<chain::transaction::unsigned::UnsignedTransaction> for UnsignedTransaction {
    fn from(t: chain::transaction::unsigned::UnsignedTransaction) -> Self {
        UnsignedTransaction(t)
    }
}

impl From<UnsignedTransaction> for chain::transaction::unsigned::UnsignedTransaction {
    fn from(t: UnsignedTransaction) -> Self {
        t.0
    }
}
