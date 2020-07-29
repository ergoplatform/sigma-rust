//! Interpreter with enhanced functionality to prove statements.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(missing_docs)]

use super::{
    dlog_protocol, fiat_shamir_hash_fn, fiat_shamir_tree_to_bytes, serialize_sig, Challenge,
    PrivateInput, ProofTree, SigmaBoolean, SigmaProofOfKnowledgeTree, UncheckedLeaf,
    UncheckedSchnorr, UncheckedSigmaTree, UncheckedTree, UnprovenSchnorr, UnprovenTree,
};
use crate::{
    chain::{ContextExtension, ProverResult},
    eval::{Env, EvalError, Evaluator},
    ErgoTree, ErgoTreeParsingError,
};

pub struct TestProver {
    pub secrets: Vec<PrivateInput>,
}

impl Evaluator for TestProver {}
impl Prover for TestProver {
    fn secrets(&self) -> &[PrivateInput] {
        self.secrets.as_ref()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ProverError {
    ErgoTreeError(ErgoTreeParsingError),
    EvalError(EvalError),
    ReducedToFalse,
    TreeRootIsNotReal,
    SimulatedLeafWithoutChallenge,
    RealUnprovenTreeWithoutChallenge,
    SecretNotFound,
}

impl From<ErgoTreeParsingError> for ProverError {
    fn from(err: ErgoTreeParsingError) -> Self {
        ProverError::ErgoTreeError(err)
    }
}

pub trait Prover: Evaluator {
    fn secrets(&self) -> &[PrivateInput];

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
                    let unchecked_tree = self.prove_to_unchecked(tree, message)?;
                    Ok(UncheckedTree::UncheckedSigmaTree(unchecked_tree))
                }
            });
        proof.map(|v| ProverResult {
            proof: serialize_sig(v),
            extension: ContextExtension::empty(),
        })
    }

    fn convert_to_unproven(&self, sigma_tree: SigmaBoolean) -> UnprovenTree {
        match sigma_tree {
            SigmaBoolean::TrivialProp(_) => todo!(), // TODO: why it's even here
            SigmaBoolean::ProofOfKnowledge(pok) => match pok {
                SigmaProofOfKnowledgeTree::ProveDHTuple(_) => todo!(),
                SigmaProofOfKnowledgeTree::ProveDlog(prove_dlog) => {
                    UnprovenTree::UnprovenSchnorr(UnprovenSchnorr {
                        proposition: prove_dlog,
                        commitment_opt: None,
                        randomness_opt: None,
                        challenge_opt: None,
                        simulated: false,
                    })
                }
            },
            SigmaBoolean::CAND(_) => todo!(),
        }
    }

    /// The comments in this section are taken from the algorithm for the
    /// Sigma-protocol prover as described in the white paper
    /// https://ergoplatform.org/docs/ErgoScript.pdf (Appendix A)
    fn prove_to_unchecked(
        &self,
        unproven_tree: UnprovenTree,
        message: &[u8],
    ) -> Result<UncheckedSigmaTree, ProverError> {
        // Prover Step 1: Mark as real everything the prover can prove
        let step1 = self.mark_real(unproven_tree);

        // Prover Step 2: If the root of the tree is marked "simulated" then the prover does not have enough witnesses
        // to perform the proof. Abort.
        if !step1.real() {
            return Err(ProverError::TreeRootIsNotReal);
        }

        // Prover Step 3: Change some "real" nodes to "simulated" to make sure each node
        // has the right number of simulated children.

        // skipped, since it leaves UnprovenSchnorr untouched
        // let step3 = self.polish_simulated(step1);

        // Prover Steps 4, 5, and 6 together: find challenges for simulated nodes; simulate simulated leaves;
        // compute commitments for real leaves
        let step6 = self.simulate_and_commit(step1)?;

        // Prover Steps 7: convert the relevant information in the tree (namely, tree structure, node types,
        // the statements being proven and commitments at the leaves)
        // to a string
        let mut s = fiat_shamir_tree_to_bytes(&step6);

        // Prover Step 8: compute the challenge for the root of the tree as the Fiat-Shamir hash of s
        // and the message being signed.
        s.append(&mut message.to_vec());
        let root_challenge = fiat_shamir_hash_fn(s.as_slice());
        let step8 = step6.with_challenge(Challenge(root_challenge));

        // Prover Step 9: complete the proof by computing challenges at real nodes and additionally responses at real leaves
        let step9 = self.proving(step8);
        todo!();
    }

    /**
     Prover Step 1: This step will mark as "real" every node for which the prover can produce a real proof.
     This step may mark as "real" more nodes than necessary if the prover has more than the minimal
     necessary number of witnesses (for example, more than one child of an OR).
     This will be corrected in the next step.
     In a bottom-up traversal of the tree, do the following for each node:
    */
    fn mark_real(&self, unproven_tree: UnprovenTree) -> UnprovenTree {
        match unproven_tree {
            UnprovenTree::UnprovenSchnorr(us) => {
                let secret_known = self.secrets().iter().any(|s| match s {
                    PrivateInput::DlogProverInput(dl) => dl.public_image() == us.proposition,
                    _ => false,
                });
                UnprovenTree::UnprovenSchnorr(UnprovenSchnorr {
                    simulated: !secret_known,
                    ..us
                })
            }
        }
    }

    /**
     Prover Step 3: This step will change some "real" nodes to "simulated" to make sure each node has
     the right number of simulated children.
     In a top-down traversal of the tree, do the following for each node:
    */
    fn polish_simulated(&self, unproven_tree: UnprovenTree) -> UnprovenTree {
        todo!()
    }

    /**
     Prover Step 4: In a top-down traversal of the tree, compute the challenges e for simulated children of every node
     Prover Step 5: For every leaf marked "simulated", use the simulator of the Sigma-protocol for that leaf
     to compute the commitment $a$ and the response z, given the challenge e that is already stored in the leaf.
     Prover Step 6: For every leaf marked "real", use the first prover step of the Sigma-protocol for that leaf to
     compute the commitment a.
    */
    fn simulate_and_commit(&self, tree: UnprovenTree) -> Result<ProofTree, ProverError> {
        match tree {
            UnprovenTree::UnprovenSchnorr(us) => {
                if us.simulated {
                    // Step 5 (simulated leaf -- complete the simulation)
                    if let Some(challenge) = us.challenge_opt {
                        let (fm, sm) = dlog_protocol::interactive_prover::simulate(
                            &us.proposition,
                            &challenge,
                        );
                        Ok(ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(
                            UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(
                                UncheckedSchnorr {
                                    proposition: us.proposition,
                                    commitment_opt: Some(fm),
                                    challenge,
                                    second_message: sm,
                                },
                            )),
                        )))
                    } else {
                        Err(ProverError::SimulatedLeafWithoutChallenge)
                    }
                } else {
                    // Step 6 (real leaf -- compute the commitment a)
                    let (r, commitment) =
                        dlog_protocol::interactive_prover::first_message(&us.proposition);
                    Ok(ProofTree::UnprovenTree(UnprovenTree::UnprovenSchnorr(
                        UnprovenSchnorr {
                            commitment_opt: Some(commitment),
                            randomness_opt: Some(r),
                            ..us
                        },
                    )))
                }
            }
        }
    }

    /**
     Prover Step 9: Perform a top-down traversal of only the portion of the tree marked "real" in order to compute
     the challenge e for every node marked "real" below the root and, additionally, the response z for every leaf
     marked "real"
    */
    fn proving(&self, tree: ProofTree) -> Result<ProofTree, ProverError> {
        // If the node is a leaf marked "real", compute its response according to the second prover step
        // of the Sigma-protocol given the commitment, challenge, and witness
        match tree {
            ProofTree::UncheckedTree(unchecked_tree) => Ok(tree),
            ProofTree::UnprovenTree(unproven_tree) => match unproven_tree {
                UnprovenTree::UnprovenSchnorr(us) if unproven_tree.real() => {
                    if let Some(challenge) = us.challenge_opt {
                        if let Some(priv_key) = self.secrets().iter().find(|s| match s {
                            PrivateInput::DlogProverInput(dl) => {
                                dl.public_image() == us.proposition
                            }
                            _ => false,
                        }) {
                            let z = dlog_protocol::interactive_prover::second_message(
                                &priv_key,
                                us.randomness_opt.unwrap(),
                                &challenge,
                            );
                            Ok(UncheckedSchnorr {
                                proposition: us.proposition,
                                ommitment_opt: None,
                                challenge,
                                second_message: z,
                            })
                        } else {
                            Err(ProverError::SecretNotFound)
                        }
                    } else {
                        Err(ProverError::RealUnprovenTreeWithoutChallenge)
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{Constant, ConstantVal, Expr},
        sigma_protocol::{DlogProverInput, SigmaProp},
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

        let prover = TestProver { secrets: vec![] };
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

        let prover = TestProver { secrets: vec![] };
        let res = prover.prove(&bool_false_tree, &Env::empty(), message.as_slice());
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ProverError::ReducedToFalse);
    }

    #[test]
    fn test_prove_pk_prop() {
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
        // assert!(res.is_ok());
        assert!(!res.unwrap().proof.is_empty());
    }
}
