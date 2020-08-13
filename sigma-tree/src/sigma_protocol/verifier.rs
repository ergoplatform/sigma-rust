//! Verifier

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(missing_docs)]

use super::{
    dlog_protocol, fiat_shamir_hash_fn, fiat_shamir_tree_to_bytes,
    sig_serializer::parse_sig_compute_challenges, SigmaBoolean, UncheckedLeaf, UncheckedSchnorr,
    UncheckedSigmaTree, UncheckedTree,
};
use crate::{
    eval::{Env, EvalError, Evaluator},
    ErgoTree, ErgoTreeParsingError,
};
use dlog_protocol::FirstDlogProverMessage;

#[derive(PartialEq, Eq, Debug, Clone)]
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
    /// Executes the script in a given context.
    /// Step 1: Deserialize context variables
    /// Step 2: Evaluate expression and produce SigmaProp value, which is zero-knowledge statement (see also `SigmaBoolean`).
    /// Step 3: Verify that the proof is presented to satisfy SigmaProp conditions.
    fn verify(
        &self,
        tree: &ErgoTree,
        env: &Env,
        proof: &[u8],
        message: &[u8],
    ) -> Result<VerificationResult, VerifierError> {
        let expr = tree.proposition()?;
        let cprop = self.reduce_to_crypto(expr.as_ref(), env)?.sigma_prop;
        let res: bool = match cprop {
            SigmaBoolean::TrivialProp(b) => b,
            sb => {
                // Perform Verifier Steps 1-3
                match parse_sig_compute_challenges(sb, proof.to_vec()) {
                    UncheckedTree::NoProof => false,
                    UncheckedTree::UncheckedSigmaTree(sp) => {
                        // Perform Verifier Step 4
                        let new_root = compute_commitments(sp);
                        // Verifier Steps 5-6: Convert the tree to a string `s` for input to the Fiat-Shamir hash function,
                        // using the same conversion as the prover in 7
                        // Accept the proof if the challenge at the root of the tree is equal to the Fiat-Shamir hash of `s`
                        // (and, if applicable,  the associated data). Reject otherwise.
                        let mut s = fiat_shamir_tree_to_bytes(&new_root.clone().into());
                        s.append(&mut message.to_vec());
                        let expected_challenge = fiat_shamir_hash_fn(s.as_slice());
                        new_root.challenge() == expected_challenge.into()
                    }
                }
            }
        };
        Ok(VerificationResult {
            result: res,
            cost: 0,
        })
    }
}

/**
 * Verifier Step 4: For every leaf node, compute the commitment a from the challenge e and response $z$,
 * per the verifier algorithm of the leaf's Sigma-protocol.
 * If the verifier algorithm of the Sigma-protocol for any of the leaves rejects, then reject the entire proof.
 */
fn compute_commitments(sp: UncheckedSigmaTree) -> UncheckedSigmaTree {
    match sp {
        UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(sn)) => {
            let a = dlog_protocol::interactive_prover::compute_commitment(
                &sn.proposition,
                &sn.challenge,
                &sn.second_message,
            );
            UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(UncheckedSchnorr {
                commitment_opt: Some(FirstDlogProverMessage(a)),
                ..sn
            }))
        }
        UncheckedSigmaTree::UncheckedConjecture => todo!(),
    }
}

pub struct TestVerifier;

impl Evaluator for TestVerifier {}
impl Verifier for TestVerifier {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{Constant, ConstantVal, Expr},
        sigma_protocol::{
            prover::{Prover, TestProver},
            DlogProverInput, PrivateInput, SigmaProofOfKnowledgeTree, SigmaProp,
        },
        types::SType,
    };
    use std::rc::Rc;

    #[test]
    fn test_prover_verifier_p2pk() {
        let secret = DlogProverInput::random();
        let pk = secret.public_image();
        let tree = ErgoTree::from(Rc::new(Expr::Const(Constant {
            tpe: SType::SSigmaProp,
            v: ConstantVal::SigmaProp(Box::new(SigmaProp(SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(pk),
            )))),
        })));
        let message = vec![0u8; 100];

        let prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret)],
        };
        let res = prover.prove(&tree, &Env::empty(), message.as_slice());
        let proof = res.unwrap().proof;

        let verifier = TestVerifier;
        let ver_res = verifier.verify(&tree, &Env::empty(), proof.as_slice(), message.as_slice());
        assert_eq!(ver_res.unwrap().result, true);
    }
}
