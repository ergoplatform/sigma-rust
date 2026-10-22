//! Verifier

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
    SigmaBoolean, UncheckedTree,
};
use crate::eval::EvalError;
use crate::eval::{reduce_to_crypto, ReductionDiagnosticInfo};
use dlog_protocol::FirstDlogProverMessage;
use ergotree_ir::chain::context::Context;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::ergo_tree::ErgoTreeError;

use derive_more::From;
use thiserror::Error;

/// Errors on proof verification
#[derive(Error, Debug, From)]
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
    /// Error while tree serialization for Fiat-Shamir hash
    #[error("Fiat-Shamir tree serialization error: {0}")]
    FiatShamirTreeSerializationError(FiatShamirTreeSerializationError),
}

/// Result of Box.ergoTree verification procedure (see `verify` method).
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// result of SigmaProp condition verification via sigma protocol
    pub result: bool,
    /// estimated cost of contract execution
    pub cost: u64,
    /// Diagnostic information about the reduction (pretty printed expr and/or env)
    pub diag: ReductionDiagnosticInfo,
}

/// Verifier for the proofs generater by [`super::prover::Prover`]
pub trait Verifier {
    /// Executes the script in a given context.
    /// Step 1: Deserialize context variables
    /// Step 2: Evaluate expression and produce SigmaProp value, which is zero-knowledge statement (see also `SigmaBoolean`).
    /// Step 3: Verify that the proof is presented to satisfy SigmaProp conditions.
    fn verify<'ctx>(
        &self,
        tree: &ErgoTree,
        ctx: &Context<'ctx>,
        proof: ProofBytes,
        message: &[u8],
    ) -> Result<VerificationResult, VerifierError> {
        let reduction_result = reduce_to_crypto(tree, ctx)?;
        let res: bool = match reduction_result.sigma_prop {
            SigmaBoolean::TrivialProp(b) => b,
            sb => {
                match proof {
                    ProofBytes::Empty => false,
                    ProofBytes::Some(proof_bytes) => {
                        // Perform Verifier Steps 1-3
                        let unchecked_tree = parse_sig_compute_challenges(&sb, proof_bytes)?;
                        // Perform Verifier Steps 4-6
                        check_commitments(unchecked_tree, message)?
                    }
                }
            }
        };
        Ok(VerificationResult {
            result: res,
            cost: 0,
            diag: reduction_result.diag,
        })
    }
}

/// Verify that the signature is presented to satisfy SigmaProp conditions.
pub fn verify_signature(
    sigma_tree: SigmaBoolean,
    message: &[u8],
    signature: &[u8],
) -> Result<bool, VerifierError> {
    let res: bool = match sigma_tree {
        SigmaBoolean::TrivialProp(b) => b,
        sb => {
            match signature {
                [] => false,
                _ => {
                    // Perform Verifier Steps 1-3
                    let unchecked_tree = parse_sig_compute_challenges(&sb, signature.to_vec())?;
                    // Perform Verifier Steps 4-6
                    check_commitments(unchecked_tree, message)?
                }
            }
        }
    };
    Ok(res)
}

/// Perform Verifier Steps 4-6
fn check_commitments(sp: UncheckedTree, message: &[u8]) -> Result<bool, VerifierError> {
    // Perform Verifier Step 4
    let new_root = compute_commitments(sp);
    let mut s = fiat_shamir_tree_to_bytes(&new_root.clone().into())?;
    s.extend_from_slice(message);
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
pub fn compute_commitments(sp: UncheckedTree) -> UncheckedTree {
    match sp {
        UncheckedTree::UncheckedLeaf(leaf) => match leaf {
            UncheckedLeaf::UncheckedSchnorr(sn) => {
                let a = dlog_protocol::interactive_prover::compute_commitment(
                    &sn.proposition,
                    &sn.challenge,
                    &sn.second_message,
                );
                UncheckedSchnorr {
                    commitment_opt: Some(FirstDlogProverMessage { a: a.into() }),
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
        UncheckedTree::UncheckedConjecture(conj) => conj
            .clone()
            .with_children(conj.children_ust().mapped(compute_commitments))
            .into(),
    }
}

/// Test Verifier implementation
pub struct TestVerifier;

impl Verifier for TestVerifier {}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use core::convert::TryFrom;

    use crate::sigma_protocol::private_input::{DhTupleProverInput, DlogProverInput, PrivateInput};
    use crate::sigma_protocol::prover::hint::HintsBag;
    use crate::sigma_protocol::prover::{Prover, TestProver};

    use super::*;
    use ergotree_ir::mir::atleast::Atleast;
    use ergotree_ir::mir::constant::{Constant, Literal};
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::sigma_and::SigmaAnd;
    use ergotree_ir::mir::sigma_or::SigmaOr;
    use ergotree_ir::mir::value::CollKind;
    use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
    use ergotree_ir::types::stype::SType;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    fn proof_append_some_byte(proof: &ProofBytes) -> ProofBytes {
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

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn test_prover_verifier_p2pk(secret in any::<DlogProverInput>(), message in vec(any::<u8>(), 100..200)) {
            let pk = secret.public_image();
            let tree = ErgoTree::try_from(Expr::Const(pk.into())).unwrap();

            let prover = TestProver {
                secrets: vec![PrivateInput::DlogProverInput(secret)],
            };
            let res = prover.prove(&tree,
                &force_any_val::<Context>(),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            prop_assert_eq!(verifier.verify(&tree,
                                            &force_any_val::<Context>(),
                                            proof.clone(),
                                            message.as_slice())
                            .unwrap().result,
                            true);

            // possible to append bytes
            prop_assert_eq!(verifier.verify(&tree,
                                            &force_any_val::<Context>(),
                                            proof_append_some_byte(&proof),
                                            message.as_slice())
                            .unwrap().result,
                            true);

            // wrong message
            prop_assert_eq!(verifier.verify(&tree,
                                            &force_any_val::<Context>(),
                                            proof,
                                            vec![1u8; 100].as_slice())
                            .unwrap().result,
                            false);
        }

        #[test]
        fn test_prover_verifier_dht(secret in any::<DhTupleProverInput>(), message in vec(any::<u8>(), 100..200)) {
            let pk = secret.public_image().clone();
            let tree = ErgoTree::try_from(Expr::Const(pk.into())).unwrap();

            let prover = TestProver {
                secrets: vec![PrivateInput::DhTupleProverInput(secret)],
            };
            let res = prover.prove(&tree,
                &force_any_val::<Context>(),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            prop_assert_eq!(verifier.verify(&tree,
                                            &force_any_val::<Context>(),
                                            proof.clone(),
                                            message.as_slice())
                            .unwrap().result,
                            true);

            // possible to append bytes
            prop_assert_eq!(verifier.verify(&tree,
                                            &force_any_val::<Context>(),
                                            proof_append_some_byte(&proof),
                                            message.as_slice())
                            .unwrap().result,
                            true);

            // wrong message
            prop_assert_eq!(verifier.verify(&tree,
                                            &force_any_val::<Context>(),
                                            proof,
                                            vec![1u8; 100].as_slice())
                            .unwrap().result,
                            false);
        }

        #[test]
        fn test_prover_verifier_conj_and(secret1 in any::<PrivateInput>(),
                                         secret2 in any::<PrivateInput>(),
                                         message in vec(any::<u8>(), 100..200)) {
            let pk1 = secret1.public_image();
            let pk2 = secret2.public_image();
            let expr: Expr = SigmaAnd::new(vec![Expr::Const(pk1.into()), Expr::Const(pk2.into())])
                .unwrap()
                .into();
            let tree = ErgoTree::try_from(expr).unwrap();
            let prover = TestProver {
                secrets: vec![secret1, secret2],
            };
            let res = prover.prove(&tree,
                &force_any_val::<Context>(),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree,
                                          &force_any_val::<Context>(),
                                          proof,
                                          message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true);
        }

        #[test]
        fn test_prover_verifier_conj_and_and(secret1 in any::<PrivateInput>(),
                                             secret2 in any::<PrivateInput>(),
                                             secret3 in any::<PrivateInput>(),
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
            let tree = ErgoTree::try_from(expr).unwrap();
            let prover = TestProver { secrets: vec![secret1, secret2, secret3] };
            let res = prover.prove(&tree,
                &force_any_val::<Context>(),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree,
                                          &force_any_val::<Context>(),
                                          proof,
                                          message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true);
        }

        #[test]
        fn test_prover_verifier_conj_or(secret1 in any::<PrivateInput>(),
                                         secret2 in any::<PrivateInput>(),
                                         message in vec(any::<u8>(), 100..200)) {
            let pk1 = secret1.public_image();
            let pk2 = secret2.public_image();
            let expr: Expr = SigmaOr::new(vec![Expr::Const(pk1.into()), Expr::Const(pk2.into())])
                .unwrap()
                .into();
            let tree = ErgoTree::try_from(expr).unwrap();
            let secrets = vec![secret1, secret2];
            // any secret (out of 2) known to prover should be enough
            for secret in secrets {
                let prover = TestProver {
                    secrets: vec![secret.clone()],
                };
                let res = prover.prove(&tree,
                    &force_any_val::<Context>(),
                    message.as_slice(),
                    &HintsBag::empty());
                let proof = res.unwrap_or_else(|_| panic!("proof failed for secret: {:?}", secret)).proof;
                let verifier = TestVerifier;
                let ver_res = verifier.verify(&tree,
                                              &force_any_val::<Context>(),
                                              proof,
                                              message.as_slice());
                prop_assert_eq!(ver_res.unwrap().result, true, "verify failed on secret: {:?}", &secret);
            }
        }

        #[test]
        fn test_prover_verifier_conj_or_or(secret1 in any::<PrivateInput>(),
                                             secret2 in any::<PrivateInput>(),
                                             secret3 in any::<PrivateInput>(),
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
            let tree = ErgoTree::try_from(expr).unwrap();
            let secrets = vec![secret1, secret2, secret3];
            // any secret (out of 3) known to prover should be enough
            for secret in secrets {
                let prover = TestProver {
                    secrets: vec![secret.clone()],
                };
                let res = prover.prove(&tree,
                    &force_any_val::<Context>(),
                    message.as_slice(),
                    &HintsBag::empty());
                let proof = res.unwrap_or_else(|_| panic!("proof failed for secret: {:?}", secret)).proof;
                let verifier = TestVerifier;
                let ver_res = verifier.verify(&tree,
                                              &force_any_val::<Context>(),
                                              proof,
                                              message.as_slice());
                prop_assert_eq!(ver_res.unwrap().result, true, "verify failed on secret: {:?}", &secret);
            }
        }

        #[test]
        fn test_prover_verifier_atleast(secret1 in any::<DlogProverInput>(),
                                            secret2 in any::<DlogProverInput>(),
                                             secret3 in any::<DlogProverInput>(),
                                             message in vec(any::<u8>(), 100..200)) {
            let bound = Expr::Const(2i32.into());
            let inputs = Literal::Coll(
                CollKind::from_collection(
                    SType::SSigmaProp,
                    [
                        SigmaProp::from(secret1.public_image()).into(),
                        SigmaProp::from(secret2.public_image()).into(),
                        SigmaProp::from(secret3.public_image()).into(),
                    ],
                )
                .unwrap(),
            );
            let input = Constant {
                tpe: SType::SColl(SType::SSigmaProp.into()),
                v: inputs,
            }
            .into();
            let expr: Expr = Atleast::new(bound, input).unwrap().into();
            let tree = ErgoTree::try_from(expr).unwrap();
            let prover = TestProver {
                secrets: vec![secret1.into(), secret3.into()],
            };

            let res = prover.prove(&tree,
                &force_any_val::<Context>(),
                message.as_slice(),
                &HintsBag::empty());
            let proof = res.unwrap().proof;
            let verifier = TestVerifier;
            let ver_res = verifier.verify(&tree,
                &force_any_val::<Context>(),
                proof,
                message.as_slice());
            prop_assert_eq!(ver_res.unwrap().result, true)
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod p2sh_tests {
    use super::*;
    use crate::eval::reduce_to_crypto;
    use crate::sigma_protocol::private_input::{DlogProverInput, PrivateInput};
    use crate::sigma_protocol::prover::hint::HintsBag;
    use crate::sigma_protocol::prover::{Prover, TestProver};
    use ergotree_ir::chain::address::Address;
    use ergotree_ir::chain::context_extension::ContextExtension;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::{Expr, SubstDeserializeError};
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree};
    use sigma_test_util::force_any_val;
    use sigma_util::hash::blake2b256_hash;

    // P2SH box of `proveDlog(pk)` as written by `Address::script()` and read back from
    // its bytes, plus the serialized proposition `08cd ++ pk` the spender supplies.
    fn p2sh_box_tree(secret: &DlogProverInput) -> (ErgoTree, Vec<u8>) {
        let prop_bytes = Expr::Const(secret.public_image().into())
            .sigma_serialize_bytes()
            .unwrap();
        assert_eq!(prop_bytes[..2], [0x08, 0xcd]);
        let hash: [u8; 24] = blake2b256_hash(&prop_bytes)[..24].try_into().unwrap();
        let tree = Address::P2SH(hash).script().unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree.sigma_serialize_bytes().unwrap()).unwrap();
        (tree, prop_bytes)
    }

    fn extension_with(var_id: u8, prop_bytes: &[u8]) -> ContextExtension {
        ContextExtension {
            values: [(var_id, Constant::from(prop_bytes.to_vec()))]
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn p2sh_script_with_var_126_reduces_to_proposition_and_verifies() {
        let secret = DlogProverInput::random();
        let (tree, prop_bytes) = p2sh_box_tree(&secret);
        // Literal ids, so that the test does not move with the constants under test.
        let ext = extension_with(126, &prop_bytes);
        let ctx = force_any_val::<Context>().with_extension(&ext);

        let reduced = reduce_to_crypto(&tree, &ctx).unwrap().sigma_prop;
        assert_eq!(
            reduced,
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(
                secret.public_image()
            ))
        );

        let message = vec![7u8; 100];
        let prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret)],
        };
        let proof = prover
            .prove(&tree, &ctx, &message, &HintsBag::empty())
            .unwrap()
            .proof;
        assert!(
            TestVerifier
                .verify(&tree, &ctx, proof, &message)
                .unwrap()
                .result
        );
    }

    #[test]
    fn p2sh_script_with_only_var_1_fails_to_reduce() {
        let secret = DlogProverInput::random();
        let (tree, prop_bytes) = p2sh_box_tree(&secret);
        let ext = extension_with(1, &prop_bytes);
        let ctx = force_any_val::<Context>().with_extension(&ext);
        // Substituting `DeserializeContext(126)` fails before `OptionGet(GetVar(126))` runs.
        assert_eq!(
            reduce_to_crypto(&tree, &ctx).unwrap_err(),
            EvalError::SubstDeserializeError(SubstDeserializeError::ExtensionKeyNotFound(126))
        );
    }

    #[test]
    fn p2sh_script_with_var_126_of_other_proposition_reduces_to_false() {
        // Variable 126 holds a well-formed proposition whose hash is not the address's:
        // reduction succeeds and the hash comparison makes the result false.
        let secret = DlogProverInput::random();
        let (tree, _) = p2sh_box_tree(&secret);
        let other = DlogProverInput::random();
        let other_prop_bytes = Expr::Const(other.public_image().into())
            .sigma_serialize_bytes()
            .unwrap();
        let ext = extension_with(126, &other_prop_bytes);
        let ctx = force_any_val::<Context>().with_extension(&ext);
        assert_eq!(
            reduce_to_crypto(&tree, &ctx).unwrap().sigma_prop,
            SigmaBoolean::TrivialProp(false)
        );
    }
}
