//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use ergotree_interpreter::eval::reduce_to_crypto;
use ergotree_interpreter::sigma_protocol::prover::ProverError;
use ergotree_ir::chain::context_extension::ContextExtension;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::serialization::SigmaSerializeResult;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

use super::unsigned::UnsignedTransaction;
use super::TxIoVec;
use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::Transaction;
use crate::chain::transaction::UnsignedInput;
use crate::wallet::signing::make_context;
use crate::wallet::signing::update_context;
use crate::wallet::signing::TransactionContext;
use crate::wallet::signing::TxSigningError;
use crate::wallet::tx_context::TransactionContextError;

/// Input box script reduced to SigmaBoolean
/// see EIP-19 for more details -
/// <https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md>
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct ReducedInput {
    /// value of SigmaProp type which represents a statement verifiable via sigma protocol.
    #[cfg_attr(
        feature = "json",
        serde(
            rename = "sigmaProp",
            with = "ergotree_ir::chain::json::sigma_protocol"
        )
    )]
    pub sigma_prop: SigmaBoolean,
    /// estimated cost of expression evaluation
    pub cost: u64,
    /// ContextExtension for the input
    pub extension: ContextExtension,
}

/// Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
/// After an unsigned transaction is reduced it can be signed without context.
/// Thus, it can be serialized and transferred for example to Cold Wallet and signed
/// in an environment where secrets are known.
/// see EIP-19 for more details -
/// <https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md>
/// Reference Scala implementation -
/// <https://github.com/ergoplatform/ergo-appkit/blob/1b7347caa863ecb0b9ba49ae57b090d1f386c906/common/src/main/java/org/ergoplatform/appkit/AppkitProvingInterpreter.scala#L261-L266>
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(try_from = "ReducedTransactionUnchecked"))]
pub struct ReducedTransaction {
    /// Unsigned transation
    #[cfg_attr(feature = "json", serde(rename = "unsignedTx"))]
    pub unsigned_tx: UnsignedTransaction,
    /// Transaction cost according to the prover
    #[cfg_attr(feature = "json", serde(rename = "txCost"))]
    tx_cost: u32,
    /// Reduction result for each unsigned tx input
    #[cfg_attr(feature = "json", serde(rename = "reducedInputs"))]
    reduced_inputs: TxIoVec<ReducedInput>,
}

#[cfg(feature = "json")]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReducedTransactionUnchecked {
    unsigned_tx: UnsignedTransaction,
    tx_cost: u32,
    reduced_inputs: TxIoVec<ReducedInput>,
}

#[cfg(feature = "json")]
impl TryFrom<ReducedTransactionUnchecked> for ReducedTransaction {
    type Error = SigmaSerializationError;

    fn try_from(value: ReducedTransactionUnchecked) -> Result<Self, Self::Error> {
        let tx = Self {
            unsigned_tx: value.unsigned_tx,
            tx_cost: value.tx_cost,
            reduced_inputs: value.reduced_inputs,
        };
        tx.validate()?;
        Ok(tx)
    }
}

impl ReducedTransaction {
    // Recheck at consumers because unsigned_tx is public and can be replaced.
    pub(crate) fn validate(&self) -> Result<(), SigmaSerializationError> {
        if self.unsigned_tx.inputs.len() != self.reduced_inputs.len() {
            return Err(SigmaSerializationError::NotSupported(
                "reduced input count must match unsigned input count".into(),
            ));
        }
        for (idx, (input, reduced)) in self
            .unsigned_tx
            .inputs
            .iter()
            .zip(self.reduced_inputs.iter())
            .enumerate()
        {
            // Map equality ignores insertion order, which affects bytes_to_sign.
            if input.extension.sigma_serialize_bytes()?
                != reduced.extension.sigma_serialize_bytes()?
            {
                return Err(SigmaSerializationError::NotSupported(format!(
                    "reduced input extension must match unsigned input extension at index {idx}"
                )));
            }
        }
        Ok(())
    }

    /// Returns reduction results for each unsigned tx input
    pub fn reduced_inputs(&self) -> TxIoVec<ReducedInput> {
        self.reduced_inputs.clone()
    }
}

/// Reduce each input of unsigned transaction to sigma proposition
pub fn reduce_tx(
    tx_context: TransactionContext<UnsignedTransaction>,
    state_context: &ErgoStateContext,
) -> Result<ReducedTransaction, TxSigningError> {
    let tx = &tx_context.spending_tx;
    let mut ctx = make_context(state_context, &tx_context, 0)?;
    let reduced_inputs = tx
        .inputs
        .clone()
        .enumerated()
        .try_mapped::<_, _, TxSigningError>(|(idx, input)| {
            update_context(&mut ctx, &tx_context, idx)?;
            let input_box = tx_context
                .get_input_box(&input.box_id)
                .ok_or(TransactionContextError::InputBoxNotFound(idx))?;
            let reduction_result = reduce_to_crypto(&input_box.ergo_tree, &ctx)
                .map_err(ProverError::EvalError)
                .map_err(|e| TxSigningError::ProverError(e, idx))?;
            Ok(ReducedInput {
                extension: input.extension,
                sigma_prop: reduction_result.sigma_prop,
                cost: reduction_result.cost,
            })
        })?;
    Ok(ReducedTransaction {
        unsigned_tx: tx.clone(),
        reduced_inputs,
        tx_cost: 0,
    })
}

impl SigmaSerializable for ReducedTransaction {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.validate()?;
        let msg = self.unsigned_tx.bytes_to_sign()?;
        w.put_usize_as_u32_unwrapped(msg.len())?;
        w.write_all(&msg)?;
        self.reduced_inputs.as_vec().iter().try_for_each(|red_in| {
            red_in.sigma_prop.sigma_serialize(w)?;
            w.put_u64(red_in.cost)?;
            SigmaSerializeResult::Ok(())
        })?;
        w.put_u32(self.tx_cost)?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let bytes_len = r.get_u32()?;
        let mut buf = vec![0u8; bytes_len as usize];
        r.read_exact(buf.as_mut_slice())?;
        let tx = Transaction::sigma_parse_bytes(&buf)?;
        let input_pairs: TxIoVec<(ReducedInput, UnsignedInput)> =
            tx.inputs.try_mapped::<_, _, SigmaParsingError>(|input| {
                let sigma_prop = SigmaBoolean::sigma_parse(r)?;
                let cost = r.get_u64()?;
                let extension = input.spending_proof.extension;
                let reduced_input = ReducedInput {
                    sigma_prop,
                    cost,
                    extension: extension.clone(),
                };
                let unsigned_input = UnsignedInput {
                    box_id: input.box_id,
                    extension,
                };
                Ok((reduced_input, unsigned_input))
            })?;
        let reduced_inputs = input_pairs.clone().mapped(|p| p.0);
        let unsigned_inputs = input_pairs.mapped(|p| p.1);
        let unsigned_tx =
            UnsignedTransaction::new(unsigned_inputs, tx.data_inputs, tx.output_candidates)?;
        let tx_cost = r.get_u32()?;
        Ok(ReducedTransaction {
            unsigned_tx,
            reduced_inputs,
            tx_cost,
        })
    }
}

/// Arbitrary impl
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for ReducedTransaction {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<UnsignedTransaction>(),
                any::<SigmaBoolean>(),
                any::<u32>(),
            )
                .prop_map(|(unsigned_tx, sb, tx_cost)| Self {
                    unsigned_tx: unsigned_tx.clone(),
                    reduced_inputs: unsigned_tx.inputs.mapped(|unsigned_input| ReducedInput {
                        sigma_prop: sb.clone(),
                        cost: 0,
                        extension: unsigned_input.extension,
                    }),
                    tx_cost,
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;

    use crate::chain::ergo_box::box_builder::ErgoBoxCandidateBuilder;
    use crate::wallet::Wallet;
    use ergo_chain_types::Digest32;
    use ergotree_ir::chain::ergo_box::{box_value::BoxValue, BoxId};
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::{constant::Constant, expr::Expr};
    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    fn fixture(input_count: usize, reduction_count: usize) -> ReducedTransaction {
        let output = ErgoBoxCandidateBuilder::new(
            BoxValue::SAFE_USER_MIN,
            ErgoTree::try_from(Expr::Const(Constant::from(true))).unwrap(),
            0,
        )
        .build()
        .unwrap();
        let inputs = (0..input_count)
            .map(|i| {
                UnsignedInput::new(
                    BoxId::from(Digest32::from([i as u8; 32])),
                    ContextExtension::empty(),
                )
            })
            .collect();
        ReducedTransaction {
            unsigned_tx: UnsignedTransaction::new_from_vec(inputs, vec![], vec![output]).unwrap(),
            tx_cost: 0,
            reduced_inputs: (0..reduction_count)
                .map(|_| ReducedInput {
                    sigma_prop: true.into(),
                    cost: 0,
                    extension: ContextExtension::empty(),
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    fn changed_extension(reverse: bool) -> ContextExtension {
        let mut extension = ContextExtension::empty();
        for key in if reverse { [2, 1] } else { [1, 2] } {
            extension.values.insert(key, Constant::from(key as i32));
        }
        extension
    }

    fn with_extensions(
        unsigned: ContextExtension,
        reduced: ContextExtension,
    ) -> ReducedTransaction {
        let mut tx = fixture(1, 1);
        tx.unsigned_tx = UnsignedTransaction::new(
            tx.unsigned_tx.inputs.mapped(|mut input| {
                input.extension = unsigned.clone();
                input
            }),
            tx.unsigned_tx.data_inputs,
            tx.unsigned_tx.output_candidates,
        )
        .unwrap();
        tx.reduced_inputs = tx.reduced_inputs.mapped(|mut input| {
            input.extension = reduced.clone();
            input
        });
        tx
    }

    #[test]
    fn reduced_transaction_valid_signing_and_serialization() {
        let tx = fixture(2, 2);
        assert_eq!(sigma_serialize_roundtrip(&tx), tx);
        assert!(Wallet::from_secrets(vec![])
            .sign_reduced_transaction(tx, None)
            .is_ok());
    }

    #[cfg(feature = "json")]
    #[test]
    fn reduced_transaction_valid_extensions_preserve_bytes() {
        let tx = with_extensions(changed_extension(true), changed_extension(true));
        let bytes = tx.sigma_serialize_bytes().unwrap();
        let json = serde_json::to_string(&tx).unwrap();
        let parsed: ReducedTransaction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, tx);
        assert_eq!(parsed.sigma_serialize_bytes().unwrap(), bytes);
        assert_eq!(sigma_serialize_roundtrip(&tx), tx);
        let message = tx.unsigned_tx.bytes_to_sign().unwrap();
        let signed = Wallet::from_secrets(vec![])
            .sign_reduced_transaction(tx, None)
            .unwrap();
        assert_eq!(signed.bytes_to_sign().unwrap(), message);
    }

    #[cfg(feature = "json")]
    #[test]
    fn reduced_transaction_json_rejects_missing_reduction() {
        assert!(serde_json::from_value::<ReducedTransaction>(
            serde_json::to_value(fixture(2, 1)).unwrap()
        )
        .is_err());
    }

    #[cfg(feature = "json")]
    #[test]
    fn reduced_transaction_json_rejects_excess_reduction() {
        assert!(serde_json::from_value::<ReducedTransaction>(
            serde_json::to_value(fixture(1, 2)).unwrap()
        )
        .is_err());
    }

    #[cfg(feature = "json")]
    #[test]
    fn reduced_transaction_json_rejects_extension_mismatch() {
        let tx = with_extensions(changed_extension(false), ContextExtension::empty());
        assert!(
            serde_json::from_value::<ReducedTransaction>(serde_json::to_value(tx).unwrap())
                .is_err()
        );
    }

    #[test]
    fn reduced_transaction_signing_rejects_mutated_input_count() {
        let mut tx = fixture(1, 1);
        tx.unsigned_tx = fixture(2, 2).unsigned_tx;
        assert!(Wallet::from_secrets(vec![])
            .generate_deterministic_commitments(&tx, &[])
            .is_err());
        assert!(Wallet::from_secrets(vec![])
            .sign_reduced_transaction(tx, None)
            .is_err());
    }

    #[test]
    fn reduced_transaction_signing_rejects_excess_reduction() {
        assert!(Wallet::from_secrets(vec![])
            .sign_reduced_transaction(fixture(1, 2), None)
            .is_err());
    }

    #[test]
    fn reduced_transaction_signing_rejects_extension_mismatch() {
        let tx = with_extensions(changed_extension(false), ContextExtension::empty());
        assert!(Wallet::from_secrets(vec![])
            .generate_deterministic_commitments(&tx, &[])
            .is_err());
        assert!(Wallet::from_secrets(vec![])
            .sign_reduced_transaction(tx, None)
            .is_err());
    }

    #[test]
    fn reduced_transaction_serialization_rejects_missing_reduction() {
        assert!(fixture(2, 1).sigma_serialize_bytes().is_err());
    }

    #[test]
    fn reduced_transaction_serialization_rejects_excess_reduction() {
        assert!(fixture(1, 2).sigma_serialize_bytes().is_err());
    }

    #[test]
    fn reduced_transaction_serialization_rejects_extension_mismatch() {
        assert!(
            with_extensions(changed_extension(false), ContextExtension::empty())
                .sigma_serialize_bytes()
                .is_err()
        );
    }

    #[test]
    fn reduced_transaction_signing_rejects_extension_order_mismatch() {
        let unsigned = changed_extension(false);
        let reduced = changed_extension(true);
        assert_eq!(unsigned, reduced);
        assert_ne!(
            unsigned.sigma_serialize_bytes().unwrap(),
            reduced.sigma_serialize_bytes().unwrap()
        );
        let tx = with_extensions(unsigned, reduced);
        assert!(tx.sigma_serialize_bytes().is_err());
        assert!(Wallet::from_secrets(vec![])
            .generate_deterministic_commitments(&tx, &[])
            .is_err());
        #[cfg(feature = "json")]
        assert!(
            serde_json::from_str::<ReducedTransaction>(&serde_json::to_string(&tx).unwrap())
                .is_err()
        );
        assert!(Wallet::from_secrets(vec![])
            .sign_reduced_transaction(tx, None)
            .is_err());
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(32))]

        #[test]
        fn ser_roundtrip(v in any::<ReducedTransaction>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
