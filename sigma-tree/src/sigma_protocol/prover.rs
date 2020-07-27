//! Interpreter with enhanced functionality to prove statements.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(missing_docs)]

use super::{serialize_sig, SigmaBoolean, UncheckedSigmaTree, UncheckedTree, UnprovenTree};
use crate::{
    chain::{ContextExtension, ProverResult},
    eval::{Env, EvalError, Evaluator},
    ErgoTree, ErgoTreeParsingError,
};

pub struct TestProver {}

impl Evaluator for TestProver {}
impl Prover for TestProver {}

#[derive(PartialEq, Eq, Debug, Clone)]
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
            .and_then(|v| match v.sigma_prop {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{Constant, ConstantVal, Expr},
        types::SType,
    };
    use std::rc::Rc;

    #[test]
    fn test_prove_true_prop() {
        let bool_true_tree = ErgoTree::from(Rc::new(Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: ConstantVal::Boolean(true),
        })));
        let message = vec![0u8; 100];

        let prover = TestProver {};
        let res = prover.prove(&bool_true_tree, &Env::empty(), message.as_slice());
        assert!(res.is_ok());
        assert!(res.unwrap().proof.is_empty());
    }

    #[test]
    fn test_prove_false_prop() {
        let bool_false_tree = ErgoTree::from(Rc::new(Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: ConstantVal::Boolean(false),
        })));
        let message = vec![0u8; 100];

        let prover = TestProver {};
        let res = prover.prove(&bool_false_tree, &Env::empty(), message.as_slice());
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ProverError::ReducedToFalse);
    }
}
