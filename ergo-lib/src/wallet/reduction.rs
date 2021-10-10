//! functions to work with reduced transaction

use crate::wallet::signing::{TransactionContext, make_context, TxSigningError};
use crate::chain::ergo_state_context::ErgoStateContext;
use std::rc::Rc;
use ergotree_interpreter::eval::env::Env;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_interpreter::eval::EvalError;
use ergotree_ir::ergo_tree::ErgoTreeError;

use std::io::Write;
use ergotree_interpreter::sigma_protocol::verifier::TestVerifier;
use thiserror::Error;
use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_interpreter::eval::Evaluator;
/// Wallet errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ReductionError {

    /// Error on Serialization error on reduction
    #[error("Serialization Error: {0}")]
    SigmaSerializationError(SigmaSerializationError),

    /// Error on Evaluation error
    #[error("Evaluation Error: {0}")]
    EvalError(EvalError),

    /// Error on Ergo Tree proposition
    #[error("Proposition Error: {0}")]
    ErgoTreeError(ErgoTreeError),

    /// Error on make signing tx
    #[error("Proposition Error: {0}")]
    TxSigningError(TxSigningError),
}

impl From<SigmaSerializationError> for ReductionError {
    fn from(e: SigmaSerializationError) -> Self {
        ReductionError::SigmaSerializationError(e)
    }
}

impl From<EvalError> for ReductionError {
    fn from(e: EvalError) -> Self {
        ReductionError::EvalError(e)
    }
}

impl From<ErgoTreeError> for ReductionError {
    fn from(e: ErgoTreeError) -> Self {
        ReductionError::ErgoTreeError(e)
    }
}

impl From<TxSigningError> for ReductionError {
    fn from(e: TxSigningError) -> Self {
        ReductionError::TxSigningError(e)
    }
}


/// Signs a transaction (generating proofs for inputs)
pub fn reduce_transaction(
    tx_context: TransactionContext,
    state_context: &ErgoStateContext,
) -> Result<Vec<u8>, ReductionError> {
    let tx = &tx_context.spending_tx;
    let mut data = Vec::new();
    let mut w = SigmaByteWriter::new(&mut data, None);
    let signing_bytes = tx.bytes_to_sign().map_err(ReductionError::from)?;
    let message_to_sign = signing_bytes.as_slice();
    w.put_u32(message_to_sign.len() as u32);
    w.write(message_to_sign);
    for idx in 0..tx_context.boxes_to_spend.len() {
        let input = &(tx_context.boxes_to_spend[idx]);
        let ctx = Rc::new(make_context(state_context, &tx_context, idx)?);
        let expr = input.ergo_tree.proposition()?;
        let verifier = TestVerifier;
        let verifier_result = verifier.reduce_to_crypto(&expr, &Env::empty(), ctx)?;
        verifier_result.sigma_prop.sigma_serialize(&mut w);
        w.put_u64(verifier_result.cost);
    }
    w.put_u64(0);
    Ok(data)
}
