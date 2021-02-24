//! Transaction signing

use std::rc::Rc;

use crate::chain::ergo_box::BoxId;
use crate::chain::transaction::Input;
use crate::chain::{
    ergo_box::ErgoBox,
    ergo_state_context::ErgoStateContext,
    transaction::{unsigned::UnsignedTransaction, Transaction},
};

use ergotree_ir::eval::context::Context;
use ergotree_ir::eval::env::Env;
use ergotree_ir::ir_ergo_box::IrBoxId;
use ergotree_ir::ir_ergo_box::IrErgoBox;
use ergotree_ir::ir_ergo_box::IrErgoBoxArena;
use ergotree_ir::ir_ergo_box::IrErgoBoxArenaError;
use ergotree_ir::sigma_protocol::prover::Prover;
use ergotree_ir::sigma_protocol::prover::ProverError;
use thiserror::Error;

/// Errors on transaction signing
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum TxSigningError {
    /// error on proving an input
    #[error("Prover error (tx input index {1}): {0}")]
    ProverError(ProverError, usize),
    /// failed to find an input in boxes_to_spend
    #[error("Input box not found (index {0})")]
    InputBoxNotFound(usize),
    /// Context creation error
    #[error("Context error: {0}")]
    ContextError(String),
}

/// Transaction and an additional info required for signing
#[derive(PartialEq, Debug, Clone)]
pub struct TransactionContext {
    /// Unsigned transaction to sign
    pub spending_tx: UnsignedTransaction,
    /// Boxes corresponding to [`UnsignedTransaction::inputs`]
    pub boxes_to_spend: Vec<ErgoBox>,
    /// Boxes corresponding to [`UnsignedTransaction::data_inputs`]
    pub data_boxes: Vec<ErgoBox>,
}

/// Holding all ErgoBox needed for interpreter [`ergotree_ir::eval::context::Context`]
#[derive(Debug)]
pub struct ErgoBoxArena {}

impl ErgoBoxArena {
    /// Create new arena and store given boxes
    pub fn new(self_box: ErgoBox, outputs: Vec<ErgoBox>, data_inputs: Vec<ErgoBox>) -> Self {
        todo!()
    }
}

impl IrErgoBoxArena for ErgoBoxArena {
    fn get(&self, id: &IrBoxId) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError> {
        todo!()
    }
}

impl From<BoxId> for IrBoxId {
    fn from(id: BoxId) -> Self {
        IrBoxId::new(*id.0 .0)
    }
}

/// `self_index` - index of the SELF box in the tx_ctx.boxes_to_spend
pub fn make_context(
    state_ctx: &ErgoStateContext,
    tx_ctx: &TransactionContext,
    self_index: usize,
) -> Result<Context, TxSigningError> {
    let height = state_ctx.pre_header.height;
    let self_box =
        tx_ctx
            .boxes_to_spend
            .get(self_index)
            .cloned()
            .ok_or(TxSigningError::ContextError(
                "self_index is out of bounds".to_string(),
            ))?;
    let outputs: Vec<ErgoBox> = tx_ctx
        .spending_tx
        .output_candidates
        .iter()
        .enumerate()
        .map(|(idx, b)| ErgoBox::from_box_candidate(b, tx_ctx.spending_tx.id(), idx as u16))
        .collect();
    let data_inputs: Vec<ErgoBox> = tx_ctx.data_boxes.clone();
    let self_box_ir = self_box.box_id().into();
    let outputs_ir = outputs.iter().map(|b| b.box_id().into()).collect();
    let data_inputs_ir = data_inputs.iter().map(|b| b.box_id().into()).collect();
    let box_arena = Rc::new(ErgoBoxArena::new(self_box, outputs, data_inputs));
    Ok(Context {
        box_arena,
        height,
        self_box: self_box_ir,
        outputs: outputs_ir,
        data_inputs: data_inputs_ir,
    })
}

/// Signs a transaction (generating proofs for inputs)
pub fn sign_transaction(
    prover: &dyn Prover,
    tx_context: TransactionContext,
    state_context: &ErgoStateContext,
) -> Result<Transaction, TxSigningError> {
    let tx = tx_context.spending_tx.clone();
    let message_to_sign = tx.bytes_to_sign();
    let mut signed_inputs: Vec<Input> = vec![];
    tx_context
        .boxes_to_spend
        .iter()
        .enumerate()
        .try_for_each(|(idx, input_box)| {
            if let Some(unsigned_input) = tx.inputs.get(idx) {
                let ctx = Rc::new(make_context(state_context, &tx_context, idx)?);
                prover
                    .prove(
                        &input_box.ergo_tree,
                        &Env::empty(),
                        ctx,
                        message_to_sign.as_slice(),
                    )
                    .map(|proof| {
                        let input = Input::new(unsigned_input.box_id.clone(), proof);
                        signed_inputs.push(input);
                    })
                    .map_err(|e| TxSigningError::ProverError(e, idx))
            } else {
                Err(TxSigningError::InputBoxNotFound(idx))
            }
        })?;
    Ok(Transaction::new(
        signed_inputs,
        tx.data_inputs,
        tx.output_candidates,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ergotree_ir::sigma_protocol::private_input::DlogProverInput;
    use ergotree_ir::sigma_protocol::private_input::PrivateInput;
    use ergotree_ir::sigma_protocol::prover::TestProver;
    use ergotree_ir::sigma_protocol::verifier::TestVerifier;
    use ergotree_ir::sigma_protocol::verifier::Verifier;
    use ergotree_ir::sigma_protocol::verifier::VerifierError;
    use ergotree_ir::test_util::force_any_val;
    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::chain::{
        ergo_box::{box_builder::ErgoBoxCandidateBuilder, BoxValue, NonMandatoryRegisters},
        transaction::{TxId, UnsignedInput},
    };
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::expr::Expr;
    use std::rc::Rc;

    fn verify_tx_proofs(
        tx: &Transaction,
        boxes_to_spend: &[ErgoBox],
    ) -> Result<bool, VerifierError> {
        let verifier = TestVerifier;
        let message = tx.bytes_to_sign();
        boxes_to_spend
            .iter()
            .zip(tx.inputs.clone())
            .try_fold(true, |acc, (b, input)| {
                let res = verifier.verify(
                    &b.ergo_tree,
                    &Env::empty(),
                    Rc::new(force_any_val::<Context>()),
                    &input.spending_proof().proof,
                    &message,
                )?;
                Ok(res.result && acc)
            })
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_tx_signing(secrets in vec(any::<DlogProverInput>(), 1..10)) {
            let boxes_to_spend: Vec<ErgoBox> = secrets.iter().map(|secret|{
                let pk = secret.public_image();
                let tree = ErgoTree::from(Expr::Const(pk.into()));
                ErgoBox::new(BoxValue::SAFE_USER_MIN,
                             tree,
                             vec![],
                             NonMandatoryRegisters::empty(),
                             0,
                             TxId::zero(),
                             0)
            }).collect();
            let prover = TestProver {
                secrets: secrets.clone().into_iter().map(PrivateInput::DlogProverInput).collect(),
            };
            let inputs = boxes_to_spend.clone().into_iter().map(UnsignedInput::from).collect();
            let ergo_tree = ErgoTree::from(Expr::Const(secrets.get(0).unwrap().public_image().into()));
            let candidate = ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, ergo_tree, 0)
                .build().unwrap();
            let output_candidates = vec![candidate];
            let tx = UnsignedTransaction::new(inputs, vec![], output_candidates);
            let tx_context = TransactionContext { spending_tx: tx,
                                                  boxes_to_spend: boxes_to_spend.clone(), data_boxes: vec![] };
            let res = sign_transaction(Box::new(prover).as_ref(), tx_context, &ErgoStateContext::dummy());
            let signed_tx = res.unwrap();
            prop_assert!(verify_tx_proofs(&signed_tx, &boxes_to_spend).unwrap());
        }
    }
}
