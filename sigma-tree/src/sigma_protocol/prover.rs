//! Interpreter with enhanced functionality to prove statements.

#![allow(dead_code)]
#![allow(unused_variables)]

use super::{serialize_sig, SigmaBoolean, UncheckedSigmaTree, UncheckedTree, UnprovenTree};
use crate::{
    chain::{ContextExtension, ProverResult},
    eval::{Env, EvalError, Evaluator},
    ErgoTree, ErgoTreeParsingError,
};

pub struct TestProver {}

impl Evaluator for TestProver {}
impl Prover for TestProver {}

pub enum ProverError {
    ErgoTreeError(ErgoTreeParsingError),
    EvalError(EvalError),
    ReducedToFalse,
}

impl From<ErgoTreeParsingError> for ProverError {
    fn from(err: ErgoTreeParsingError) -> Self {
        ProverError::ErgoTreeError(err)
    }
}

pub struct ReductionResult {
    sigma_prop: SigmaBoolean,
    cost: u64,
}

pub trait Prover: Evaluator {
    fn prove(
        &self,
        tree: &ErgoTree,
        env: &Env,
        message: &[u8],
    ) -> Result<ProverResult, ProverError> {
        let expr = tree.proposition()?;
        let proof = self
            .reduce_to_crypto(expr.as_ref(), env)
            .map_err(ProverError::EvalError)
            .and_then(|v| match v {
                SigmaBoolean::TrivialProp(true) => Ok(UncheckedTree::NoProof),
                SigmaBoolean::TrivialProp(false) => Err(ProverError::ReducedToFalse),
                sb => {
                    let tree = self.convert_to_unproven(sb);
                    let unchecked_tree = self.prove_to_unchecked(tree, message);
                    Ok(UncheckedTree::UncheckedSigmaTree(unchecked_tree))
                }
            });
        proof.map(|v| ProverResult {
            proof: serialize_sig(v),
            extension: ContextExtension::empty(),
        })
    }

    fn convert_to_unproven(&self, sigma_tree: SigmaBoolean) -> UnprovenTree {
        todo!()
    }

    fn prove_to_unchecked(
        &self,
        unproven_tree: UnprovenTree,
        message: &[u8],
    ) -> UncheckedSigmaTree {
        todo!()
    }
}
