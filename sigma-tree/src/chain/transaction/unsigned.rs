//! Unsigned (without proofs) transaction

use super::{
    super::{
        data_input::DataInput, digest32::blake2b256_hash, ergo_box::ErgoBoxCandidate, input::Input,
        ProofBytes::Empty, ProverResult, UnsignedInput,
    },
    Transaction, TxId,
};
use crate::serialization::SigmaSerializable;

/// Unsigned (inputs without proofs) transaction
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
                    proof: Empty,
                    extension: ui.extension.clone(),
                },
            })
            .collect();
        let tx = Transaction::new(
            empty_proofs_input,
            self.data_inputs.clone(),
            self.output_candidates.clone(),
        );
        tx.sigma_serialise_bytes()
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

        #[test]
        fn test_unsigned_tx_bytes_to_sign(v in any::<UnsignedTransaction>()) {
            prop_assert!(!v.bytes_to_sign().is_empty());
        }

    }
}
