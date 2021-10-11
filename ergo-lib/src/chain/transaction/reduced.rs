//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use std::rc::Rc;

use ergotree_interpreter::eval::env::Env;
use ergotree_interpreter::eval::reduce_to_crypto;
use ergotree_interpreter::eval::ReductionResult;
use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use ergotree_interpreter::sigma_protocol::prover::ProverError;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializeResult;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

use crate::chain::ergo_state_context::ErgoStateContext;
use crate::chain::transaction::Transaction;
use crate::chain::transaction::UnsignedInput;
use crate::wallet::signing::make_context;
use crate::wallet::signing::TransactionContext;
use crate::wallet::signing::TxSigningError;

use super::unsigned::UnsignedTransaction;
use super::TxIoVec;

/// Input box script reduced to SigmaBoolean
/// see EIP-19 for more details - https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md
#[derive(PartialEq, Debug, Clone)]
pub struct ReducedInput {
    /// Input box script reduced to SigmaBoolean
    pub reduction_result: ReductionResult,
    /// ContextExtension for the input
    pub extension: ContextExtension,
}

/// Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
/// After an unsigned transaction is reduced it can be signed without context.
/// Thus, it can be serialized and transferred for example to Cold Wallet and signed
/// in an environment where secrets are known.
/// see EIP-19 for more details - https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md
/// Reference Scala implementation - https://github.com/ergoplatform/ergo-appkit/blob/1b7347caa863ecb0b9ba49ae57b090d1f386c906/common/src/main/java/org/ergoplatform/appkit/AppkitProvingInterpreter.scala#L261-L266
#[derive(PartialEq, Debug)]
pub struct ReducedTransaction {
    /// Unsigned transation
    unsigned_tx: UnsignedTransaction,
    /// Reduction result for each unsigned tx input
    reduced_inputs: TxIoVec<ReducedInput>,
}

/// Reduce each input of unsigned transaction to sigma proposition
pub fn reduce_tx(
    tx_context: TransactionContext,
    state_context: &ErgoStateContext,
) -> Result<ReducedTransaction, TxSigningError> {
    let tx = &tx_context.spending_tx;
    let reduced_inputs = tx.inputs.clone().enumerated().try_mapped(|(idx, input)| {
        if let Some(input_box) = tx_context
            .boxes_to_spend
            .iter()
            .find(|b| b.box_id() == input.box_id)
        {
            let ctx = Rc::new(make_context(state_context, &tx_context, idx)?);
            let expr = input_box
                .ergo_tree
                .proposition()
                .map_err(ProverError::ErgoTreeError)
                .map_err(|e| TxSigningError::ProverError(e, idx))?;
            let reduction_result = reduce_to_crypto(&expr, &Env::empty(), ctx)
                .map_err(ProverError::EvalError)
                .map_err(|e| TxSigningError::ProverError(e, idx))?;
            Ok(ReducedInput {
                reduction_result,
                extension: input.extension,
            })
        } else {
            Err(TxSigningError::InputBoxNotFound(idx))
        }
    })?;
    Ok(ReducedTransaction {
        unsigned_tx: tx.clone(),
        reduced_inputs,
    })
}

impl SigmaSerializable for ReducedTransaction {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        let msg = self.unsigned_tx.bytes_to_sign()?;
        w.put_usize_as_u32_unwrapped(msg.len())?;
        w.write_all(&msg)?;
        self.reduced_inputs.as_vec().iter().try_for_each(|red_in| {
            red_in.reduction_result.sigma_prop.sigma_serialize(w)?;
            Ok(w.put_u64(red_in.reduction_result.cost)?)
        })
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
                    reduction_result: ReductionResult { sigma_prop, cost },
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
        Ok(ReducedTransaction {
            unsigned_tx,
            reduced_inputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    impl Arbitrary for ReducedTransaction {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<UnsignedTransaction>(), any::<SigmaBoolean>())
                .prop_map(|(unsigned_tx, sb)| Self {
                    unsigned_tx: unsigned_tx.clone(),
                    reduced_inputs: unsigned_tx.inputs.mapped(|unsigned_input| ReducedInput {
                        reduction_result: ReductionResult {
                            sigma_prop: sb.clone(),
                            cost: 0,
                        },
                        extension: unsigned_input.extension,
                    }),
                })
                .boxed()
        }
        type Strategy = BoxedStrategy<Self>;
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn ser_roundtrip(v in any::<ReducedTransaction>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
