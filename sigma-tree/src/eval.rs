#![allow(dead_code)]
#![allow(unused_variables)]

use crate::{
    ast::{ops::BinOp, ops::NumOp, Expr},
    chain::{ContextExtension, ProverResult},
    sigma_protocol::SigmaBoolean,
    ErgoTree, ErgoTreeParsingError,
};

use cost_accum::CostAccumulator;
use value::Value;

mod cost_accum;
mod costs;
mod value;

pub struct Env();

pub enum EvalError {
    InvalidResultType,
}

pub struct VerificationResult {
    result: bool,
    cost: u64,
}

pub struct ReductionResult {
    sigma_prop: SigmaBoolean,
    cost: u64,
}

pub trait Evaluator {
    // TODO: add the cost to the returned result
    fn reduce_to_crypto(&self, expr: &Expr, env: &Env) -> Result<SigmaBoolean, EvalError> {
        let mut ca = CostAccumulator::new(0, None);
        eval(expr, env, &mut ca).and_then(|v| match v {
            Value::Boolean(b) => Ok(SigmaBoolean::TrivialProp(b)),
            Value::SigmaProp(sb) => Ok(*sb),
            _ => Err(EvalError::InvalidResultType),
        })
    }
}

#[allow(unconditional_recursion)]
fn eval(expr: &Expr, env: &Env, ca: &mut CostAccumulator) -> Result<Value, EvalError> {
    match expr {
        Expr::Const(_) => todo!(), //Ok(EvalResult(*v)),
        Expr::Coll { .. } => todo!(),
        Expr::Tup { .. } => todo!(),
        Expr::PredefFunc(_) => todo!(),
        Expr::CollM(_) => todo!(),
        Expr::BoxM(_) => todo!(),
        Expr::CtxM(_) => todo!(),
        Expr::MethodCall { .. } => todo!(),
        Expr::BinOp(bin_op, l, r) => {
            let v_l = eval(l, env, ca)?;
            let v_r = eval(r, env, ca)?;
            ca.add_cost_of(expr);
            Ok(match bin_op {
                BinOp::Num(op) => match op {
                    NumOp::Add => v_l + v_r,
                },
            })
        }
    }
}

// TODO: extract tree types to sigma_protocol

pub enum ProofTree {
    UncheckedTree(UncheckedTree),
    UnprovenTree(UnprovenTree),
}

pub enum UnprovenTree {}

pub enum UncheckedSigmaTree {}

pub enum UncheckedTree {
    NoProof,
    UncheckedSigmaTree(UncheckedSigmaTree),
}

fn serialize_sig(tree: UncheckedTree) -> Vec<u8> {
    todo!()
}

// TODO: extract Prover

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

// TODO: extract Verifier

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
