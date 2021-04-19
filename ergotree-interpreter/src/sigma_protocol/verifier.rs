//! Verifier

use std::rc::Rc;

use super::proof_tree::rewrite;
use super::proof_tree::ProofTree;
use super::prover::ProofBytes;
use super::sig_serializer::SigParsingError;
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
use ergotree_ir::ergo_tree::ErgoTreeParsingError;

use derive_more::From;

/// Errors on proof verification
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum VerifierError {
    /// Failed to parse ErgoTree from bytes
    ErgoTreeError(ErgoTreeParsingError),
    /// Failed to evaluate ErgoTree
    EvalError(EvalError),
    /// Signature parsing error
    SigParsingError(SigParsingError),
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
                        check_commitments(sp, message)
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
fn check_commitments(sp: UncheckedSigmaTree, message: &[u8]) -> bool {
    // Perform Verifier Step 4
    let new_root = compute_commitments(sp);
    let mut s = fiat_shamir_tree_to_bytes(&new_root.clone().into());
    s.append(&mut message.to_vec());
    // Verifier Steps 5-6: Convert the tree to a string `s` for input to the Fiat-Shamir hash function,
    // using the same conversion as the prover in 7
    // Accept the proof if the challenge at the root of the tree is equal to the Fiat-Shamir hash of `s`
    // (and, if applicable,  the associated data). Reject otherwise.
    let expected_challenge = fiat_shamir_hash_fn(s.as_slice());
    new_root.challenge() == expected_challenge.into()
}

/// Verifier Step 4: For every leaf node, compute the commitment a from the challenge e and response $z$,
/// per the verifier algorithm of the leaf's Sigma-protocol.
/// If the verifier algorithm of the Sigma-protocol for any of the leaves rejects, then reject the entire proof.
fn compute_commitments(sp: UncheckedSigmaTree) -> UncheckedSigmaTree {
    let proof_tree = rewrite(sp.into(), &|tree| match tree {
        ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(ust)) => match ust {
            UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(sn)) => {
                let a = dlog_protocol::interactive_prover::compute_commitment(
                    &sn.proposition,
                    &sn.challenge,
                    &sn.second_message,
                );
                Ok(Some(
                    UncheckedSchnorr {
                        commitment_opt: Some(FirstDlogProverMessage(a)),
                        ..sn.clone()
                    }
                    .into(),
                ))
            }
            UncheckedSigmaTree::UncheckedConjecture(_) => Ok(None),
        },
        _ => Ok(None),
    })
    .unwrap();

    if let ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(ust)) = proof_tree {
        ust
    } else {
        panic!(":(")
    }
}

/// Test Verifier implementation
pub struct TestVerifier;

impl Evaluator for TestVerifier {}
impl Verifier for TestVerifier {}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::sigma_protocol::prover::hint::HintsBag;
    use crate::sigma_protocol::{
        private_input::{DlogProverInput, PrivateInput},
        prover::{Prover, TestProver},
    };
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::sigma_and::SigmaAnd;
    use ergotree_ir::mir::sigma_or::SigmaOr;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    fn proof_append_byte(proof: &ProofBytes) -> ProofBytes {
        match proof {
            ProofBytes::Empty => panic!(),
            ProofBytes::Some(bytes) => {
                let mut new_bytes = bytes.clone();
                new_bytes.push(1u8);
                ProofBytes::Some(new_bytes)
            }
        }
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(4))]

        #[test]
        fn test_prover_verifier_p2pk(secret in any::<DlogProverInput>(), message in vec(any::<u8>(), 100..200)) {
            let pk = secret.public_image();
            let tree = ErgoTree::from(Expr::Const(pk.into()));

            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret)],
            };
            let res = prover.prove(&tree,
                &Env::empty(),
                Rc::new(force_any_val::<Context>()),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            prop_assert_eq!(verifier.verify(&tree,
                                            &Env::empty(),
                                            Rc::new(force_any_val::<Context>()),
                                            proof.clone(),
                                            message.as_slice())
                            .unwrap().result,
                            true);

            // possible to append bytes
            prop_assert_eq!(verifier.verify(&tree,
                                            &Env::empty(),
                                            Rc::new(force_any_val::<Context>()),
                                            proof_append_byte(&proof),
                                            message.as_slice())
                            .unwrap().result,
                            true);

            // wrong message
            prop_assert_eq!(verifier.verify(&tree,
                                            &Env::empty(),
                                            Rc::new(force_any_val::<Context>()),
                                            proof,
                                            vec![1u8; 100].as_slice())
                            .unwrap().result,
                            false);
        }

        #[test]
        fn test_prover_verifier_conj_and(secret1 in any::<DlogProverInput>(),
                                         secret2 in any::<DlogProverInput>(),
                                         message in vec(any::<u8>(), 100..200)) {
            let pk1 = secret1.public_image();
            let pk2 = secret2.public_image();
            let expr: Expr = SigmaAnd::new(vec![Expr::Const(pk1.into()), Expr::Const(pk2.into())])
                .unwrap()
                .into();
            let tree = ErgoTree::from(expr);
            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret1), PrivateInput::DlogProverInput(secret2)],
            };
            let res = prover.prove(&tree,
                &Env::empty(),
                Rc::new(force_any_val::<Context>()),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree,
                                          &Env::empty(),
                                          Rc::new(force_any_val::<Context>()),
                                          proof,
                                          message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true);
        }

        #[test]
        fn test_prover_verifier_conj_and_and(secret1 in any::<DlogProverInput>(),
                                             secret2 in any::<DlogProverInput>(),
                                             secret3 in any::<DlogProverInput>(),
                                             message in vec(any::<u8>(), 100..200)) {
            let pk1 = secret1.public_image();
            let pk2 = secret2.public_image();
            let pk3 = secret3.public_image();
            let expr: Expr = SigmaAnd::new(vec![
                Expr::Const(pk1.into()),
                SigmaAnd::new(vec![Expr::Const(pk2.into()), Expr::Const(pk3.into())])
                    .unwrap()
                    .into(),
            ]).unwrap().into();
            let tree = ErgoTree::from(expr);
            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret1),
                    PrivateInput::DlogProverInput(secret2),
                    PrivateInput::DlogProverInput(secret3)
                ],
            };
            let res = prover.prove(&tree,
                &Env::empty(),
                Rc::new(force_any_val::<Context>()),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree,
                                          &Env::empty(),
                                          Rc::new(force_any_val::<Context>()),
                                          proof,
                                          message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true);
        }

        #[test]
        fn test_prover_verifier_conj_or(secret1 in any::<DlogProverInput>(),
                                         secret2 in any::<DlogProverInput>(),
                                         message in vec(any::<u8>(), 100..200)) {
            let pk1 = secret1.public_image();
            let pk2 = secret2.public_image();
            let expr: Expr = SigmaOr::new(vec![Expr::Const(pk1.into()), Expr::Const(pk2.into())])
                .unwrap()
                .into();
            let tree = ErgoTree::from(expr);
            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret1), PrivateInput::DlogProverInput(secret2)],
            };
            let res = prover.prove(&tree,
                &Env::empty(),
                Rc::new(force_any_val::<Context>()),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree,
                                          &Env::empty(),
                                          Rc::new(force_any_val::<Context>()),
                                          proof,
                                          message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true);
        }

        #[test]
        fn test_prover_verifier_conj_or_or(secret1 in any::<DlogProverInput>(),
                                             secret2 in any::<DlogProverInput>(),
                                             secret3 in any::<DlogProverInput>(),
                                             message in vec(any::<u8>(), 100..200)) {
            let pk1 = secret1.public_image();
            let pk2 = secret2.public_image();
            let pk3 = secret3.public_image();
            let expr: Expr = SigmaOr::new(vec![
                Expr::Const(pk1.into()),
                SigmaOr::new(vec![Expr::Const(pk2.into()), Expr::Const(pk3.into())])
                    .unwrap()
                    .into(),
            ]).unwrap().into();
            let tree = ErgoTree::from(expr);
            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret1),
                    PrivateInput::DlogProverInput(secret2),
                    PrivateInput::DlogProverInput(secret3)
                ],
            };
            let res = prover.prove(&tree,
                &Env::empty(),
                Rc::new(force_any_val::<Context>()),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree,
                                          &Env::empty(),
                                          Rc::new(force_any_val::<Context>()),
                                          proof,
                                          message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true);
        }
    }
    // TODO: add custom SigmaBoolean generator for  PK + AND + OR of various depth and test prover/verifier

    // TODO: draft an issue for prover/verifier spec sharing test vectors with sigmastate
    // Test vector should have: SigmaBoolean, secrets, proof
}
