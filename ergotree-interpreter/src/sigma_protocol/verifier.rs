//! Verifier

use std::rc::Rc;

use super::dht_protocol;
use super::dht_protocol::FirstDhTupleProverMessage;
use super::fiat_shamir::FiatShamirTreeSerializationError;
use super::prover::ProofBytes;
use super::sig_serializer::SigParsingError;
use super::unchecked_tree::UncheckedDhTuple;
use super::{
    dlog_protocol,
    fiat_shamir::{fiat_shamir_hash_fn, fiat_shamir_tree_to_bytes},
    sig_serializer::parse_sig_compute_challenges,
    unchecked_tree::{UncheckedLeaf, UncheckedSchnorr},
    SigmaBoolean, UncheckedSigmaTree, UncheckedTree,
};
use crate::eval::context::Context;
use crate::eval::env::Env;
use crate::eval::{EvalError, Evaluator};
use dlog_protocol::FirstDlogProverMessage;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::ergo_tree::ErgoTreeError;

use derive_more::From;
use thiserror::Error;

/// Errors on proof verification
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum VerifierError {
    /// Failed to parse ErgoTree from bytes
    #[error("ErgoTreeError: {0}")]
    ErgoTreeError(ErgoTreeError),
    /// Failed to evaluate ErgoTree
    #[error("EvalError: {0}")]
    EvalError(EvalError),
    /// Signature parsing error
    #[error("SigParsingError: {0}")]
    SigParsingError(SigParsingError),
    /// Unexpected value encountered
    #[error("Unexpected: {0}")]
    Unexpected(String),
    /// Error while tree serialization for Fiat-Shamir hash
    #[error("Fiat-Shamir tree serialization error: {0}")]
    FiatShamirTreeSerializationError(FiatShamirTreeSerializationError),
}

/// Result of Box.ergoTree verification procedure (see `verify` method).
pub struct VerificationResult {
    /// result of SigmaProp condition verification via sigma protocol
    pub result: bool,
    /// estimated cost of contract execution
    pub cost: u64,
}

/// Verifier for the proofs generater by [`super::prover::Prover`]
pub trait Verifier: Evaluator {
    /// Executes the script in a given context.
    /// Step 1: Deserialize context variables
    /// Step 2: Evaluate expression and produce SigmaProp value, which is zero-knowledge statement (see also `SigmaBoolean`).
    /// Step 3: Verify that the proof is presented to satisfy SigmaProp conditions.
    fn verify(
        &self,
        tree: &ErgoTree,
        env: &Env,
        ctx: Rc<Context>,
        proof: ProofBytes,
        message: &[u8],
    ) -> Result<VerificationResult, VerifierError> {
        let expr = tree.proposition()?;
        let cprop = self.reduce_to_crypto(expr.as_ref(), env, ctx)?.sigma_prop;
        let res: bool = match cprop {
            SigmaBoolean::TrivialProp(b) => b,
            sb => {
                // Perform Verifier Steps 1-3
                match parse_sig_compute_challenges(&sb, proof)? {
                    UncheckedTree::UncheckedSigmaTree(sp) => {
                        // Perform Verifier Steps 4-6
                        check_commitments(sp, message)?
                    }
                    UncheckedTree::NoProof => false,
                }
            }
        };
        Ok(VerificationResult {
            result: res,
            cost: 0,
        })
    }
}

/// Perform Verifier Steps 4-6
fn check_commitments(sp: UncheckedSigmaTree, message: &[u8]) -> Result<bool, VerifierError> {
    // Perform Verifier Step 4
    let new_root = compute_commitments(sp);
    let mut s = fiat_shamir_tree_to_bytes(&new_root.clone().into())?;
    s.append(&mut message.to_vec());
    // Verifier Steps 5-6: Convert the tree to a string `s` for input to the Fiat-Shamir hash function,
    // using the same conversion as the prover in 7
    // Accept the proof if the challenge at the root of the tree is equal to the Fiat-Shamir hash of `s`
    // (and, if applicable,  the associated data). Reject otherwise.
    let expected_challenge = fiat_shamir_hash_fn(s.as_slice());
    Ok(new_root.challenge() == expected_challenge.into())
}

/// Verifier Step 4: For every leaf node, compute the commitment a from the challenge e and response $z$,
/// per the verifier algorithm of the leaf's Sigma-protocol.
/// If the verifier algorithm of the Sigma-protocol for any of the leaves rejects, then reject the entire proof.
fn compute_commitments(sp: UncheckedSigmaTree) -> UncheckedSigmaTree {
    match sp {
        UncheckedSigmaTree::UncheckedLeaf(leaf) => match leaf {
            UncheckedLeaf::UncheckedSchnorr(sn) => {
                let a = dlog_protocol::interactive_prover::compute_commitment(
                    &sn.proposition,
                    &sn.challenge,
                    &sn.second_message,
                );
                UncheckedSchnorr {
                    commitment_opt: Some(FirstDlogProverMessage(a.into())),
                    ..sn
                }
                .into()
            }
            UncheckedLeaf::UncheckedDhTuple(dh) => {
                let (a, b) = dht_protocol::interactive_prover::compute_commitment(
                    &dh.proposition,
                    &dh.challenge,
                    &dh.second_message,
                );
                UncheckedDhTuple {
                    commitment_opt: Some(FirstDhTupleProverMessage::new(a, b)),
                    ..dh
                }
                .into()
            }
        },
        UncheckedSigmaTree::UncheckedConjecture(conj) => conj
            .clone()
            .with_children(conj.children_ust().mapped(compute_commitments))
            .into(),
    }
}

/// Test Verifier implementation
pub struct TestVerifier;

impl Evaluator for TestVerifier {}
impl Verifier for TestVerifier {}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests;
