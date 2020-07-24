//! Verifier

#![allow(dead_code)]
#![allow(unused_variables)]

use super::SigmaBoolean;
use crate::{
    eval::{Env, EvalError, Evaluator},
    ErgoTree, ErgoTreeParsingError,
};

pub enum VerifierError {
    ErgoTreeError(ErgoTreeParsingError),
    EvalError(EvalError),
}

impl From<ErgoTreeParsingError> for VerifierError {
    fn from(err: ErgoTreeParsingError) -> Self {
        VerifierError::ErgoTreeError(err)
    }
}

impl From<EvalError> for VerifierError {
    fn from(err: EvalError) -> Self {
        VerifierError::EvalError(err)
    }
}

pub struct VerificationResult {
    result: bool,
    cost: u64,
}

pub trait Verifier: Evaluator {
    fn verify(
        &mut self,
        tree: &ErgoTree,
        env: &Env,
        proof: &[u8],
        message: &[u8],
    ) -> Result<VerificationResult, VerifierError> {
        let expr = tree.proposition()?;
        let cprop = self.reduce_to_crypto(expr.as_ref(), env)?;
        let res: bool = match cprop {
            SigmaBoolean::TrivialProp(b) => b,
            sb => todo!(),
        };
        Ok(VerificationResult {
            result: res,
            cost: 0,
        })
    }
}
