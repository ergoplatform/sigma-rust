use std::convert::TryFrom;
use thiserror::Error;

use crate::chain::transaction::unsigned::UnsignedTransaction;
use crate::chain::transaction::{DataInput, Input, Transaction, TransactionError, UnsignedInput};
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::tx_id::TxId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TransactionJson {
    #[cfg_attr(feature = "json", serde(rename = "id"))]
    pub tx_id: TxId,
    /// inputs, that will be spent by this transaction.
    #[cfg_attr(feature = "json", serde(rename = "inputs"))]
    pub inputs: Vec<Input>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    #[cfg_attr(feature = "json", serde(rename = "dataInputs"))]
    pub data_inputs: Vec<DataInput>,
    #[cfg_attr(feature = "json", serde(rename = "outputs"))]
    pub outputs: Vec<ErgoBox>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UnsignedTransactionJson {
    /// unsigned inputs, that will be spent by this transaction.
    #[cfg_attr(feature = "json", serde(rename = "inputs"))]
    pub inputs: Vec<UnsignedInput>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    #[cfg_attr(feature = "json", serde(rename = "dataInputs"))]
    pub data_inputs: Vec<DataInput>,
    /// box candidates to be created by this transaction
    #[cfg_attr(feature = "json", serde(rename = "outputs"))]
    pub outputs: Vec<ErgoBoxCandidate>,
}

impl From<UnsignedTransaction> for UnsignedTransactionJson {
    fn from(v: UnsignedTransaction) -> Self {
        UnsignedTransactionJson {
            inputs: v.inputs.as_vec().clone(),
            data_inputs: v
                .data_inputs
                .map(|di| di.as_vec().clone())
                .unwrap_or_default(),
            outputs: v.output_candidates.as_vec().clone(),
        }
    }
}

impl TryFrom<UnsignedTransactionJson> for UnsignedTransaction {
    // We never return this type but () fails to compile (can't format) and ! is experimental
    type Error = String;
    fn try_from(tx_json: UnsignedTransactionJson) -> Result<Self, Self::Error> {
        UnsignedTransaction::new_from_vec(tx_json.inputs, tx_json.data_inputs, tx_json.outputs)
            .map_err(|e| format!("TryFrom<UnsignedTransactionJson> error: {0}", e))
    }
}

impl From<Transaction> for TransactionJson {
    fn from(v: Transaction) -> Self {
        TransactionJson {
            tx_id: v.id(),
            inputs: v.inputs.as_vec().clone(),
            data_inputs: v
                .data_inputs
                .map(|di| di.as_vec().clone())
                .unwrap_or_default(),
            outputs: v.outputs,
        }
    }
}

/// Errors on parsing Transaction from JSON
#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[allow(missing_docs)]
pub enum TransactionFromJsonError {
    #[error("Tx id parsed from JSON differs from calculated from serialized bytes")]
    InvalidTxId,
    #[error("Tx error: {0}")]
    TransactionError(#[from] TransactionError),
}

impl TryFrom<TransactionJson> for Transaction {
    type Error = TransactionFromJsonError;
    fn try_from(tx_json: TransactionJson) -> Result<Self, Self::Error> {
        let output_candidates: Vec<ErgoBoxCandidate> =
            tx_json.outputs.iter().map(|o| o.clone().into()).collect();
        let tx = Transaction::new_from_vec(tx_json.inputs, tx_json.data_inputs, output_candidates)?;
        if tx.tx_id == tx_json.tx_id {
            Ok(tx)
        } else {
            Err(TransactionFromJsonError::InvalidTxId)
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use crate::chain::transaction::unsigned::UnsignedTransaction;
    use crate::chain::transaction::Transaction;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn tx_roundtrip(t in any::<Transaction>()) {
            let j = serde_json::to_string(&t)?;
            eprintln!("{}", j);
            let t_parsed: Transaction = serde_json::from_str(&j)?;
            prop_assert_eq![t, t_parsed];
        }

        #[test]
        fn unsigned_tx_roundtrip(t in any::<UnsignedTransaction>()) {
            let j = serde_json::to_string(&t)?;
            eprintln!("{}", j);
            let t_parsed: UnsignedTransaction = serde_json::from_str(&j)?;
            prop_assert_eq![t, t_parsed];
        }

    }
}
