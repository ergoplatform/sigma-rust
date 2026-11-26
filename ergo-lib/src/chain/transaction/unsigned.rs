//! Unsigned (without proofs) transaction

use super::input::UnsignedInput;

use super::DataInput;
use super::Transaction;
use super::TxIoVec;
use super::{distinct_token_ids, TransactionError};
use alloc::vec::Vec;
use bounded_vec::BoundedVec;
use ergo_chain_types::blake2b256_hash;

use core::convert::TryInto;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::token::TokenId;
use ergotree_ir::chain::tx_id::TxId;
use ergotree_ir::chain::IndexSet;
use ergotree_ir::serialization::SigmaSerializationError;

/// Unsigned (inputs without proofs) transaction
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        try_from = "crate::chain::json::transaction::UnsignedTransactionJson",
        into = "crate::chain::json::transaction::UnsignedTransactionJson"
    )
)]
#[derive(PartialEq, Eq, Debug, Clone)]
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
    pub(crate) outputs: TxIoVec<ErgoBox>,
}

impl UnsignedTransaction {
    /// Creates new transaction from vectors
    pub fn new_from_vec(
        inputs: Vec<UnsignedInput>,
        data_inputs: Vec<DataInput>,
        output_candidates: Vec<ErgoBoxCandidate>,
    ) -> Result<UnsignedTransaction, TransactionError> {
        Ok(UnsignedTransaction::new(
            inputs
                .try_into()
                .map_err(TransactionError::InvalidInputsCount)?,
            BoundedVec::opt_empty_vec(data_inputs)
                .map_err(TransactionError::InvalidDataInputsCount)?,
            output_candidates
                .try_into()
                .map_err(TransactionError::InvalidOutputCandidatesCount)?,
        )?)
    }

    /// Creates new transaction
    pub fn new(
        inputs: TxIoVec<UnsignedInput>,
        data_inputs: Option<TxIoVec<DataInput>>,
        output_candidates: TxIoVec<ErgoBoxCandidate>,
    ) -> Result<UnsignedTransaction, SigmaSerializationError> {
        let outputs = output_candidates
            .clone()
            .enumerated()
            .try_mapped_ref(|(idx, b)| ErgoBox::from_box_candidate(b, TxId::zero(), *idx as u16))?;

        let tx_to_sign = UnsignedTransaction {
            tx_id: TxId::zero(),
            inputs,
            data_inputs,
            output_candidates,
            outputs,
        };
        let tx_id = tx_to_sign.calc_tx_id()?;

        let outputs = tx_to_sign
            .output_candidates
            .clone()
            .enumerated()
            .try_mapped_ref(|(idx, bc)| ErgoBox::from_box_candidate(bc, tx_id, *idx as u16))?;

        Ok(UnsignedTransaction {
            tx_id,
            outputs,
            ..tx_to_sign
        })
    }

    fn calc_tx_id(&self) -> Result<TxId, SigmaSerializationError> {
        let bytes = self.bytes_to_sign()?;
        Ok(TxId(blake2b256_hash(&bytes)))
    }

    fn to_tx_without_proofs(&self) -> Result<Transaction, SigmaSerializationError> {
        let empty_proofs_input = self.inputs.mapped_ref(|ui| ui.input_to_sign());
        Transaction::new(
            empty_proofs_input,
            self.data_inputs.clone(),
            self.output_candidates.clone(),
        )
    }

    /// Get transaction id
    pub fn id(&self) -> TxId {
        self.tx_id
    }

    /// message to be signed by the [`ergotree_interpreter::sigma_protocol::prover::Prover`] (serialized tx)
    pub fn bytes_to_sign(&self) -> Result<Vec<u8>, SigmaSerializationError> {
        let tx = self.to_tx_without_proofs()?;
        tx.bytes_to_sign()
    }

    /// Returns distinct token ids from all output_candidates
    pub fn distinct_token_ids(&self) -> IndexSet<TokenId> {
        distinct_token_ids(&self.output_candidates)
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
pub mod arbitrary {
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
                    Self::new_from_vec(inputs, data_inputs, outputs).unwrap()
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_unsigned_tx_bytes_to_sign(v in any::<UnsignedTransaction>()) {
            prop_assert!(!v.bytes_to_sign().unwrap().is_empty());
        }

    }

    #[test]
    fn test_new_propagates_unserializable_candidate_error() {
        use ergotree_ir::chain::ergo_box::box_value::BoxValue;
        use ergotree_ir::chain::ergo_box::{NonMandatoryRegisters, RegisterValue};
        use ergotree_ir::ergo_tree::ErgoTree;
        use sigma_test_util::force_any_val;

        let invalid_regs = NonMandatoryRegisters::try_from(vec![RegisterValue::Invalid {
            bytes: vec![0xde, 0xad, 0xbe, 0xef],
            error_msg: "invalid register bytes".into(),
        }])
        .unwrap();
        let bad_candidate = ErgoBoxCandidate {
            value: BoxValue::SAFE_USER_MIN,
            ergo_tree: force_any_val::<ErgoTree>(),
            tokens: None,
            additional_registers: invalid_regs,
            creation_height: 1,
        };
        let res = UnsignedTransaction::new_from_vec(
            vec![force_any_val::<UnsignedInput>()],
            vec![],
            vec![bad_candidate],
        );
        assert!(matches!(
            res,
            Err(TransactionError::SigmaSerializationError(
                SigmaSerializationError::NotSupported(_)
            ))
        ));
    }
}
