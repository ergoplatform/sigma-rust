//! Ergo transaction

use crate::box_coll::ErgoBoxCandidates;
use crate::box_coll::ErgoBoxes;
use crate::data_input::DataInputs;
use crate::error_conversion::to_js;
use crate::input::{Inputs, UnsignedInputs};
use crate::json::TransactionJsonEip12;
use crate::json::UnsignedTransactionJsonEip12;
use ergo_lib::chain;
use ergo_lib::chain::transaction::distinct_token_ids;
use ergo_lib::ergotree_ir::chain::base16_bytes::Base16DecodedBytes;
use ergo_lib::ergotree_ir::chain::base16_bytes::Base16EncodedBytes;
use ergo_lib::ergotree_ir::chain::digest32::Digest32;
use js_sys::Uint8Array;
use std::convert::TryFrom;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

pub mod reduced;

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
        JsValue::from_serde(&tx_dapp).map_err(|e| JsValue::from_str(&format!("{}", e)))
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
        JsValue::from_serde(&tx_dapp).map_err(|e| JsValue::from_str(&format!("{}", e)))
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
