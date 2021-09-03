//! Unsigned (without proofs) transaction

use crate::chain::token::TokenId;

use super::distinct_token_ids;
use super::input::{Input, UnsignedInput};
#[cfg(feature = "json")]
use super::json;
use super::prover_result::ProverResult;
use super::DataInput;
use super::TxIoVec;
use super::{
    super::{digest32::blake2b256_hash, ergo_box::ErgoBoxCandidate},
    Transaction, TxId,
};
use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use ergotree_ir::serialization::SigmaSerializationError;
use indexmap::IndexSet;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use std::convert::TryFrom;
#[cfg(feature = "json")]
use std::convert::TryInto;

/// Unsigned (inputs without proofs) transaction
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        try_from = "json::transaction::UnsignedTransactionJson",
        into = "json::transaction::UnsignedTransactionJson"
    )
)]
#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedTransaction {
    tx_id: TxId,
    /// unsigned inputs, that will be spent by this transaction.
    pub inputs: TxIoVec<UnsignedInput>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    pub data_inputs: Option<TxIoVec<DataInput>>,
    /// box candidates to be created by this transaction
    pub output_candidates: TxIoVec<ErgoBoxCandidate>,
}

impl UnsignedTransaction {
    /// Creates new transaction
    pub fn new(
        inputs: TxIoVec<UnsignedInput>,
        data_inputs: Option<TxIoVec<DataInput>>,
        output_candidates: TxIoVec<ErgoBoxCandidate>,
    ) -> Result<UnsignedTransaction, SigmaSerializationError> {
        let tx_to_sign = UnsignedTransaction {
            tx_id: TxId::zero(),
            inputs,
            data_inputs,
            output_candidates,
        };
        let tx_id = tx_to_sign.calc_tx_id()?;
        Ok(UnsignedTransaction {
            tx_id,
            ..tx_to_sign
        })
    }

    fn calc_tx_id(&self) -> Result<TxId, SigmaSerializationError> {
        let bytes = self.bytes_to_sign()?;
        Ok(TxId(blake2b256_hash(&bytes)))
    }

    fn to_tx_wo_proofs(&self) -> Result<Transaction, SigmaSerializationError> {
        let empty_proofs_input = self.inputs.mapped_ref(|ui| {
            Input::new(
                ui.box_id.clone(),
                ProverResult {
                    proof: ProofBytes::Empty,
                    extension: ui.extension.clone(),
                },
            )
        });
        Transaction::new(
            empty_proofs_input,
            self.data_inputs.clone(),
            self.output_candidates.clone(),
        )
    }

    /// Get transaction id
    pub fn id(&self) -> TxId {
        self.tx_id.clone()
    }

    /// message to be signed by the [`ergotree_interpreter::sigma_protocol::prover::Prover`] (serialized tx)
    pub fn bytes_to_sign(&self) -> Result<Vec<u8>, SigmaSerializationError> {
        let tx = self.to_tx_wo_proofs()?;
        tx.bytes_to_sign()
    }

    /// Returns distinct token ids from all output_candidates
    pub fn distinct_token_ids(&self) -> IndexSet<TokenId> {
        distinct_token_ids(self.output_candidates.clone())
    }
}

#[cfg(feature = "json")]
impl From<UnsignedTransaction> for json::transaction::UnsignedTransactionJson {
    fn from(v: UnsignedTransaction) -> Self {
        json::transaction::UnsignedTransactionJson {
            inputs: v.inputs.as_vec().clone(),
            data_inputs: v
                .data_inputs
                .map(|di| di.as_vec().clone())
                .unwrap_or_default(),
            outputs: v.output_candidates.as_vec().clone(),
        }
    }
}

#[cfg(feature = "json")]
impl TryFrom<json::transaction::UnsignedTransactionJson> for UnsignedTransaction {
    // We never return this type but () fails to compile (can't format) and ! is experimental
    type Error = String;
    fn try_from(tx_json: json::transaction::UnsignedTransactionJson) -> Result<Self, Self::Error> {
        UnsignedTransaction::new(
            tx_json
                .inputs
                .try_into()
                .map_err(|e: bounded_vec::BoundedVecOutOfBounds| e.to_string())?,
            tx_json.data_inputs.try_into().ok(),
            tx_json
                .outputs
                .try_into()
                .map_err(|e: bounded_vec::BoundedVecOutOfBounds| e.to_string())?,
        )
        .map_err(|e| format!("unsigned tx serialization failed: {0}", e))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use proptest::prelude::*;
    use proptest::{arbitrary::Arbitrary, collection::vec};

    impl Arbitrary for UnsignedTransaction {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                vec(any::<UnsignedInput>(), 1..10),
                vec(any::<DataInput>(), 0..10),
                vec(any::<ErgoBoxCandidate>(), 1..10),
            )
                .prop_map(|(inputs, data_inputs, outputs)| {
                    Self::new(
                        inputs.try_into().unwrap(),
                        data_inputs.try_into().ok(),
                        outputs.try_into().unwrap(),
                    )
                    .unwrap()
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_unsigned_tx_bytes_to_sign(v in any::<UnsignedTransaction>()) {
            prop_assert!(!v.bytes_to_sign().unwrap().is_empty());
        }

    }
}
