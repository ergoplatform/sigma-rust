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
    use ergotree_ir::serialization::SigmaSerializable;
    use num_bigint::BigUint;
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
            let secrets = vec![PrivateInput::DlogProverInput(secret1), PrivateInput::DlogProverInput(secret2)];
            // any secret (out of 2) known to prover should be enough
            for secret in secrets {
                let prover = TestProver {
                    secrets: vec![secret.clone()],
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
                prop_assert_eq!(ver_res.unwrap().result, true, "verify failed on secret: {:?}", &secret);
            }
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
            let secrets = vec![
                PrivateInput::DlogProverInput(secret1),
                PrivateInput::DlogProverInput(secret2),
                PrivateInput::DlogProverInput(secret3)
            ];
            // any secret (out of 3) known to prover should be enough
            for secret in secrets {
                let prover = TestProver {
                    secrets: vec![secret.clone()],
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
                prop_assert_eq!(ver_res.unwrap().result, true, "verify failed on secret: {:?}", &secret);
            }
        }
    }

    #[test]
    fn sig_test_vector_provedlog() {
        // test vector data from
        // https://github.com/ScorexFoundation/sigmastate-interpreter/blob/6c51c13f7a494a191a7ea5645e56b04fb46a418d/sigmastate/src/test/scala/sigmastate/crypto/SigningSpecification.scala#L14-L30
        let msg =
            base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
                .unwrap();
        let sk = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
                10,
            )
            .unwrap(),
        )
        .unwrap();
        let signature = base16::decode(b"bcb866ba434d5c77869ddcbc3f09ddd62dd2d2539bf99076674d1ae0c32338ea95581fdc18a3b66789904938ac641eba1a66d234070207a2").unwrap();

        // check expected public key
        assert_eq!(
            base16::encode_lower(&sk.public_image().sigma_serialize_bytes()),
            "03cb0d49e4eae7e57059a3da8ac52626d26fc11330af8fb093fa597d8b93deb7b1"
        );

        let expr: Expr = sk.public_image().into();
        let verifier = TestVerifier;
        let ver_res = verifier.verify(
            &expr.into(),
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            signature.into(),
            msg.as_slice(),
        );
        assert_eq!(ver_res.unwrap().result, true);
    }

    #[test]
    fn sig_test_vector_conj_and() {
        // corresponding sigmastate test
        // in SigningSpecification.property("AND signature test vector")
        let msg =
            base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
                .unwrap();
        let sk1 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
                10,
            )
            .unwrap(),
        )
        .unwrap();
        let sk2 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
                10,
            )
            .unwrap(),
        )
        .unwrap();

        let signature = base16::decode(b"9b2ebb226be42df67817e9c56541de061997c3ea84e7e72dbb69edb7318d7bb525f9c16ccb1adc0ede4700a046d0a4ab1e239245460c1ba45e5637f7a2d4cc4cc460e5895125be73a2ca16091db2dcf51d3028043c2b9340").unwrap();

        let expr: Expr = SigmaAnd::new(vec![
            Expr::Const(sk1.public_image().into()),
            Expr::Const(sk2.public_image().into()),
        ])
        .unwrap()
        .into();
        let tree: ErgoTree = expr.into();

        // let prover = TestProver {
        //     secrets: vec![sk1.into(), sk2.into()],
        // };
        // let res = prover.prove(
        //     &tree,
        //     &Env::empty(),
        //     Rc::new(force_any_val::<Context>()),
        //     msg.as_slice(),
        //     &HintsBag::empty(),
        // );
        // let proof: Vec<u8> = res.unwrap().proof.into();
        // dbg!(base16::encode_lower(&proof));

        let verifier = TestVerifier;
        let ver_res = verifier.verify(
            &tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            signature.into(),
            msg.as_slice(),
        );
        assert_eq!(ver_res.unwrap().result, true);
    }

    #[test]
    fn sig_test_vector_conj_or() {
        // corresponding sigmastate test
        // in SigningSpecification.property("OR signature test vector")
        let msg =
            base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
                .unwrap();
        let sk1 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
                10,
            )
            .unwrap(),
        )
        .unwrap();
        let sk2 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
                10,
            )
            .unwrap(),
        )
        .unwrap();

        let signature = base16::decode(b"ec94d2d5ef0e1e638237f53fd883c339f9771941f70020742a7dc85130aaee535c61321aa1e1367befb500256567b3e6f9c7a3720baa75ba6056305d7595748a93f23f9fc0eb9c1aaabc24acc4197030834d76d3c95ede60c5b59b4b306cd787d010e8217f34677d046646778877c669").unwrap();

        let expr: Expr = SigmaOr::new(vec![
            Expr::Const(sk1.public_image().into()),
            Expr::Const(sk2.public_image().into()),
        ])
        .unwrap()
        .into();
        let tree: ErgoTree = expr.into();

        // let prover = TestProver {
        //     secrets: vec![sk1.into()],
        // };
        // let res = prover.prove(
        //     &tree,
        //     &Env::empty(),
        //     Rc::new(force_any_val::<Context>()),
        //     msg.as_slice(),
        //     &HintsBag::empty(),
        // );
        // let proof: Vec<u8> = res.unwrap().proof.into();
        // dbg!(base16::encode_lower(&proof));

        let verifier = TestVerifier;
        let ver_res = verifier.verify(
            &tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            signature.into(),
            msg.as_slice(),
        );
        assert_eq!(ver_res.unwrap().result, true);
    }

    #[test]
    fn sig_test_vector_conj_and_or() {
        // corresponding sigmastate test
        // in SigningSpecification.property("AND with OR signature test vector")
        let msg =
            base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
                .unwrap();
        let sk1 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
                10,
            )
            .unwrap(),
        )
        .unwrap();
        let sk2 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
                10,
            )
            .unwrap(),
        )
        .unwrap();

        let sk3 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"34648336872573478681093104997365775365807654884817677358848426648354905397359",
                10,
            )
            .unwrap(),
        )
        .unwrap();

        let signature = base16::decode(b"397e005d85c161990d0e44853fbf14951ff76e393fe1939bb48f68e852cd5af028f6c7eaaed587f6d5435891a564d8f9a77288773ce5b526a670ab0278aa4278891db53a9842df6fba69f95f6d55cfe77dd7b4bdccc1a3378ac4524b51598cb813258f64c94e98c3ef891a6eb8cbfd2e527a9038ca50b5bb50058de55a859a169628e6ae5ba4cb0332c694e450782d6f").unwrap();

        let expr: Expr = SigmaAnd::new(vec![
            Expr::Const(sk1.public_image().into()),
            SigmaOr::new(vec![
                Expr::Const(sk2.public_image().into()),
                Expr::Const(sk3.public_image().into()),
            ])
            .unwrap()
            .into(),
        ])
        .unwrap()
        .into();
        let tree: ErgoTree = expr.into();

        // let prover = TestProver {
        //     secrets: vec![sk1.into(), sk2.into()],
        // };
        // let res = prover.prove(
        //     &tree,
        //     &Env::empty(),
        //     Rc::new(force_any_val::<Context>()),
        //     msg.as_slice(),
        //     &HintsBag::empty(),
        // );
        // let proof: Vec<u8> = res.unwrap().proof.into();
        // dbg!(base16::encode_lower(&proof));

        let verifier = TestVerifier;
        let ver_res = verifier.verify(
            &tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            signature.into(),
            msg.as_slice(),
        );
        assert_eq!(ver_res.unwrap().result, true);
    }

    #[test]
    fn sig_test_vector_conj_or_and() {
        // corresponding sigmastate test
        // in SigningSpecification.property("OR with AND signature test vector")
        let msg =
            base16::decode(b"1dc01772ee0171f5f614c673e3c7fa1107a8cf727bdf5a6dadb379e93c0d1d00")
                .unwrap();
        let sk1 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"109749205800194830127901595352600384558037183218698112947062497909408298157746",
                10,
            )
            .unwrap(),
        )
        .unwrap();
        let sk2 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"50415569076448343263191022044468203756975150511337537963383000142821297891310",
                10,
            )
            .unwrap(),
        )
        .unwrap();

        let sk3 = DlogProverInput::from_biguint(
            BigUint::parse_bytes(
                b"34648336872573478681093104997365775365807654884817677358848426648354905397359",
                10,
            )
            .unwrap(),
        )
        .unwrap();

        let signature = base16::decode(b"a58b251be319a9656c21876b1136a59f42b18835dec6076c92f7a925ba28d2030218c177ab07563003eff5250cfafeb631ef610f4d710ab8e821bf632203adf23f4376580eaa17ddb36c0138f73a88551f45d92cde2b66dfbb5906c02e4d48106ff08be4a2fc29ec242f495468692f9ddeeb029dc5d8f38e2649cf09c44b67cbcfb3de4202026fb84d23ce2b4ff0f69b").unwrap();

        let expr: Expr = SigmaOr::new(vec![
            Expr::Const(sk1.public_image().into()),
            SigmaAnd::new(vec![
                Expr::Const(sk2.public_image().into()),
                Expr::Const(sk3.public_image().into()),
            ])
            .unwrap()
            .into(),
        ])
        .unwrap()
        .into();
        let tree: ErgoTree = expr.into();

        // let prover = TestProver {
        //     secrets: vec![sk1.into(), sk2.into()],
        // };
        // let res = prover.prove(
        //     &tree,
        //     &Env::empty(),
        //     Rc::new(force_any_val::<Context>()),
        //     msg.as_slice(),
        //     &HintsBag::empty(),
        // );
        // let proof: Vec<u8> = res.unwrap().proof.into();
        // dbg!(base16::encode_lower(&proof));

        let verifier = TestVerifier;
        let ver_res = verifier.verify(
            &tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            signature.into(),
            msg.as_slice(),
        );
        assert_eq!(ver_res.unwrap().result, true);
    }
}
