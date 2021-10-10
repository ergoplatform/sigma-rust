//! Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
//! is augmented with ReducedInput which contains a script reduction result.

use std::rc::Rc;

use ergotree_interpreter::eval::env::Env;
use ergotree_interpreter::eval::reduce_to_crypto;
use ergotree_interpreter::eval::ReductionResult;
use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use ergotree_interpreter::sigma_protocol::prover::ProverError;

use crate::chain::ergo_state_context::ErgoStateContext;
use crate::wallet::signing::make_context;
use crate::wallet::signing::TransactionContext;
use crate::wallet::signing::TxSigningError;

use super::unsigned::UnsignedTransaction;
use super::TxIoVec;

/// Input box script reduced to SigmaBoolean
/// see EIP-19 for more details - https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md
pub struct ReducedInput {
    // TODO: add box_id?
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
pub struct ReducedTransaction {
    /// Unsigned transation
    pub unsigned_tx: UnsignedTransaction,
    /// Reduction result for each unsigned tx input
    pub reduced_inputs: TxIoVec<ReducedInput>,
}

// TODO: move to signing?
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
