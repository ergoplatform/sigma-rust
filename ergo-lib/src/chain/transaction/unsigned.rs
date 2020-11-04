//! Unsigned (without proofs) transaction

#[cfg(feature = "json")]
use super::json;
use super::{
    super::{
        data_input::DataInput, digest32::blake2b256_hash, ergo_box::ErgoBoxCandidate, input::Input,
    },
    Transaction, TxId,
};
#[cfg(feature = "json")]
use crate::chain::transaction::ErgoBox;
#[cfg(feature = "json")]
use crate::chain::transaction::TransactionFromJsonError;
use crate::{
    chain::{
        input::UnsignedInput,
        prover_result::{ProofBytes, ProverResult},
    },
    serialization::SigmaSerializable,
};
#[cfg(feature = "json")]
use core::convert::TryFrom;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

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
    pub inputs: Vec<UnsignedInput>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    pub data_inputs: Vec<DataInput>,
    /// box candidates to be created by this transaction
    pub output_candidates: Vec<ErgoBoxCandidate>,
}

impl UnsignedTransaction {
    /// Creates new transation
    pub fn new(
        inputs: Vec<UnsignedInput>,
        data_inputs: Vec<DataInput>,
        output_candidates: Vec<ErgoBoxCandidate>,
    ) -> UnsignedTransaction {
        let tx_to_sign = UnsignedTransaction {
            tx_id: TxId::zero(),
            inputs,
            data_inputs,
            output_candidates,
        };
        let tx_id = tx_to_sign.calc_tx_id();
        UnsignedTransaction {
            tx_id,
            ..tx_to_sign
        }
    }

    fn calc_tx_id(&self) -> TxId {
        let bytes = self.bytes_to_sign();
        TxId(blake2b256_hash(&bytes))
    }

    /// message to be signed by the [`Prover`] (serialized tx)
    pub fn bytes_to_sign(&self) -> Vec<u8> {
        let empty_proofs_input = self
            .inputs
            .iter()
            .map(|ui| Input {
                box_id: ui.box_id.clone(),
                spending_proof: ProverResult {
                    proof: ProofBytes::Empty,
                    extension: ui.extension.clone(),
                },
            })
            .collect();
        let tx = Transaction::new(
            empty_proofs_input,
            self.data_inputs.clone(),
            self.output_candidates.clone(),
        );
        tx.sigma_serialize_bytes()
    }
}

#[cfg(feature = "json")]
impl TryFrom<json::transaction::UnsignedTransactionJson> for UnsignedTransaction {
    type Error = TransactionFromJsonError;
    fn try_from(
        tx_json: json::transaction::UnsignedTransactionJson,
    ) -> std::result::Result<Self, Self::Error> {
        let output_candidates = tx_json.outputs.iter().map(|o| o.clone().into()).collect();
        let tx_to_sign = UnsignedTransaction {
            tx_id: TxId::zero(),
            inputs: tx_json.inputs,
            data_inputs: tx_json.data_inputs,
            output_candidates,
        };
        let tx_id = tx_to_sign.calc_tx_id();
        let tx = UnsignedTransaction {
            tx_id,
            ..tx_to_sign
        };
        if tx.tx_id == tx_json.tx_id {
            Ok(tx)
        } else {
            Err(TransactionFromJsonError::InvalidTxId)
        }
    }
}

#[cfg(feature = "json")]
impl From<UnsignedTransaction> for json::transaction::UnsignedTransactionJson {
    fn from(t: UnsignedTransaction) -> Self {
        json::transaction::UnsignedTransactionJson {
            tx_id: t.tx_id.clone(),
            inputs: t.inputs.clone(),
            data_inputs: t.data_inputs.clone(),
            outputs: t
                .output_candidates
                .iter()
                .enumerate()
                .map(|(idx, c)| ErgoBox::from_box_candidate(c, t.tx_id.clone(), idx as u16))
                .collect(),
        }
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
                .prop_map(|(inputs, data_inputs, outputs)| Self::new(inputs, data_inputs, outputs))
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_unsigned_tx_bytes_to_sign(v in any::<UnsignedTransaction>()) {
            prop_assert!(!v.bytes_to_sign().is_empty());
        }

    }
}
