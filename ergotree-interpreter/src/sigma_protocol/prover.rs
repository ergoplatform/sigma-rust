//! Interpreter with enhanced functionality to prove statements.

mod context_extension;
mod prover_result;

pub mod hint;

use crate::sigma_protocol::unproven_tree::CandUnproven;
use crate::sigma_protocol::unproven_tree::NodePosition;
use ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
use std::rc::Rc;

pub use context_extension::*;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::ergo_tree::ErgoTreeParsingError;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
pub use prover_result::*;

use self::hint::HintsBag;

use super::unproven_tree;
use super::unproven_tree::UnprovenConjecture;
use super::{
    dlog_protocol,
    fiat_shamir::{fiat_shamir_hash_fn, fiat_shamir_tree_to_bytes},
    private_input::PrivateInput,
    sig_serializer::serialize_sig,
    unchecked_tree::UncheckedSchnorr,
    Challenge, ProofTree, SigmaBoolean, UncheckedSigmaTree, UncheckedTree, UnprovenLeaf,
    UnprovenSchnorr, UnprovenTree,
};
use crate::eval::context::Context;
use crate::eval::env::Env;
use crate::eval::{EvalError, Evaluator};
use thiserror::Error;

/// Prover errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ProverError {
    /// Failed to parse ErgoTree
    #[error("Ergo tree error: {0}")]
    ErgoTreeError(ErgoTreeParsingError),
    /// Failed to evaluate ErgoTree
    #[error("Evaluation error: {0}")]
    EvalError(EvalError),
    /// Script reduced to false
    #[error("Script reduced to false")]
    ReducedToFalse,
    /// Failed on step2(prover does not have enough witnesses to perform the proof)
    #[error("Failed on step2(prover does not have enough witnesses to perform the proof)")]
    TreeRootIsNotReal,
    /// Simulated leaf does not have challenge
    #[error("Simulated leaf does not have challenge")]
    SimulatedLeafWithoutChallenge,
    /// Lacking challenge on step 9 for "real" unproven tree
    #[error("Lacking challenge on step 9 for \"real\" unproven tree")]
    RealUnprovenTreeWithoutChallenge,
    /// Cannot find a secret for "real" unproven leaf
    #[error("Cannot find a secret for \"real\" unproven leaf")]
    SecretNotFound,
}

impl From<ErgoTreeParsingError> for ProverError {
    fn from(err: ErgoTreeParsingError) -> Self {
        ProverError::ErgoTreeError(err)
    }
}

/// Prover
pub trait Prover: Evaluator {
    /// Secrets of the prover
    fn secrets(&self) -> &[PrivateInput];

    /**
     * The comments in this section are taken from the algorithm for the
     * Sigma-protocol prover as described in the ErgoScript white-paper
     * https://ergoplatform.org/docs/ErgoScript.pdf , Appendix A
     *
     * Generate proofs for the given message for ErgoTree reduced to Sigma boolean expression
     */
    fn prove(
        &self,
        tree: &ErgoTree,
        env: &Env,
        ctx: Rc<Context>,
        message: &[u8],
        hints_bag: HintsBag,
    ) -> Result<ProverResult, ProverError> {
        let expr = tree.proposition()?;
        let proof = self
            .reduce_to_crypto(expr.as_ref(), env, ctx)
            .map_err(ProverError::EvalError)
            .and_then(|v| match v.sigma_prop {
                SigmaBoolean::TrivialProp(true) => Ok(UncheckedTree::NoProof),
                SigmaBoolean::TrivialProp(false) => Err(ProverError::ReducedToFalse),
                sb => {
                    let tree = convert_to_unproven(sb);
                    let unchecked_tree = prove_to_unchecked(self, tree, message, hints_bag)?;
                    Ok(UncheckedTree::UncheckedSigmaTree(unchecked_tree))
                }
            });
        proof.map(|v| ProverResult {
            proof: serialize_sig(v),
            extension: ContextExtension::empty(),
        })
    }
}

/// The comments in this section are taken from the algorithm for the
/// Sigma-protocol prover as described in the white paper
/// https://ergoplatform.org/docs/ErgoScript.pdf (Appendix A)
// if we are concerned about timing attacks against the prover, we should make sure that this code
//  takes the same amount of time regardless of which nodes are real and which nodes are simulated
//  In particular, we should avoid the use of exists and forall, because they short-circuit the evaluation
//  once the right value is (or is not) found. We should also make all loops look similar, the same
//  amount of copying is done regardless of what's real or simulated,
//  real vs. simulated computations take the same time, etc.
fn prove_to_unchecked<P: Prover + ?Sized>(
    prover: &P,
    unproven_tree: UnprovenTree,
    message: &[u8],
    hints_bag: HintsBag,
) -> Result<UncheckedSigmaTree, ProverError> {
    // Prover Step 1: Mark as real everything the prover can prove
    let step1 = mark_real(prover, unproven_tree, hints_bag)?;

    // Prover Step 2: If the root of the tree is marked "simulated" then the prover does not have enough witnesses
    // to perform the proof. Abort.
    if !step1.is_real() {
        return Err(ProverError::TreeRootIsNotReal);
    }

    // Prover Step 3: Change some "real" nodes to "simulated" to make sure each node
    // has the right number of simulated children.

    // skipped, since it leaves UnprovenSchnorr untouched
    // let step3 = self.polish_simulated(step1);

    // Prover Steps 4, 5, and 6 together: find challenges for simulated nodes; simulate simulated leaves;
    // compute commitments for real leaves
    let step6 = simulate_and_commit(prover, step1)?;

    // Prover Steps 7: convert the relevant information in the tree (namely, tree structure, node types,
    // the statements being proven and commitments at the leaves)
    // to a string
    let var_name = fiat_shamir_tree_to_bytes(&step6);
    let mut s = var_name;

    // Prover Step 8: compute the challenge for the root of the tree as the Fiat-Shamir hash of s
    // and the message being signed.
    s.append(&mut message.to_vec());
    let root_challenge: Challenge = fiat_shamir_hash_fn(s.as_slice()).into();
    let step8 = step6.with_challenge(root_challenge);

    // Prover Step 9: complete the proof by computing challenges at real nodes and additionally responses at real leaves
    let step9 = proving(prover, step8)?;

    // Prover Step 10: output the right information into the proof
    match step9 {
        ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(tree)) => Ok(tree),
        _ => todo!(),
    }
}

/**
 Prover Step 1: This step will mark as "real" every node for which the prover can produce a real proof.
 This step may mark as "real" more nodes than necessary if the prover has more than the minimal
 necessary number of witnesses (for example, more than one child of an OR).
 This will be corrected in the next step.
 In a bottom-up traversal of the tree, do the following for each node:
*/
fn mark_real<P: Prover + ?Sized>(
    prover: &P,
    unproven_tree: UnprovenTree,
    hints_bag: HintsBag,
) -> Result<UnprovenTree, ProverError> {
    unproven_tree::rewrite(unproven_tree, |tree| {
        Ok(match tree {
            UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(us)) => {
                // If the node is a leaf, mark it "real'' if either the witness for it is available or a hint shows the secret
                // is known to an external participant in multi-signing;
                // else mark it "simulated"
                // TODO: update with hints_bag
                let secret_known = prover.secrets().iter().any(|s| match s {
                    PrivateInput::DlogProverInput(dl) => dl.public_image() == us.proposition,
                    _ => false,
                });
                Some(
                    UnprovenSchnorr {
                        simulated: !secret_known,
                        ..us.clone()
                    }
                    .into(),
                )
            }
            UnprovenTree::UnprovenConjecture(UnprovenConjecture::CandUnproven(cand)) => {
                // If the node is AND, mark it "real" if all of its children are marked real; else mark it "simulated"
                let simulated = cand.children.iter().any(|c| c.simulated());
                Some(
                    CandUnproven {
                        simulated,
                        ..cand.clone()
                    }
                    .into(),
                )
            }
        })
    })
}

/**
 Prover Step 3: This step will change some "real" nodes to "simulated" to make sure each node has
 the right number of simulated children.
 In a top-down traversal of the tree, do the following for each node:
*/
fn polish_simulated<P: Prover + ?Sized>(_prover: &P, _unproven_tree: UnprovenTree) -> UnprovenTree {
    todo!()
}

/**
 Prover Step 4: In a top-down traversal of the tree, compute the challenges e for simulated children of every node
 Prover Step 5: For every leaf marked "simulated", use the simulator of the Sigma-protocol for that leaf
 to compute the commitment $a$ and the response z, given the challenge e that is already stored in the leaf.
 Prover Step 6: For every leaf marked "real", use the first prover step of the Sigma-protocol for that leaf to
 compute the commitment a.
*/
fn simulate_and_commit<P: Prover + ?Sized>(
    _prover: &P,
    tree: UnprovenTree,
) -> Result<ProofTree, ProverError> {
    match tree {
        UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(us)) => {
            if us.simulated {
                // Step 5 (simulated leaf -- complete the simulation)
                if let Some(challenge) = us.challenge_opt {
                    let (fm, sm) =
                        dlog_protocol::interactive_prover::simulate(&us.proposition, &challenge);
                    Ok(ProofTree::UncheckedTree(
                        UncheckedSchnorr {
                            proposition: us.proposition,
                            commitment_opt: Some(fm),
                            challenge,
                            second_message: sm,
                        }
                        .into(),
                    ))
                } else {
                    Err(ProverError::SimulatedLeafWithoutChallenge)
                }
            } else {
                // Step 6 (real leaf -- compute the commitment a)
                let (r, commitment) = dlog_protocol::interactive_prover::first_message();
                Ok(ProofTree::UnprovenTree(
                    UnprovenSchnorr {
                        commitment_opt: Some(commitment),
                        randomness_opt: Some(r),
                        ..us
                    }
                    .into(),
                ))
            }
        }
        UnprovenTree::UnprovenConjecture(_) => todo!(),
    }
}

/**
 Prover Step 9: Perform a top-down traversal of only the portion of the tree marked "real" in order to compute
 the challenge e for every node marked "real" below the root and, additionally, the response z for every leaf
 marked "real"
*/
fn proving<P: Prover + ?Sized>(prover: &P, tree: ProofTree) -> Result<ProofTree, ProverError> {
    // If the node is a leaf marked "real", compute its response according to the second prover step
    // of the Sigma-protocol given the commitment, challenge, and witness
    match tree {
        ProofTree::UncheckedTree(_) => Ok(tree),
        ProofTree::UnprovenTree(unproven_tree) => match unproven_tree {
            UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(us))
                if unproven_tree.is_real() =>
            {
                if let Some(challenge) = us.challenge_opt.clone() {
                    if let Some(priv_key) = prover
                        .secrets()
                        .iter()
                        .flat_map(|s| match s {
                            PrivateInput::DlogProverInput(dl) => vec![dl],
                            _ => vec![],
                        })
                        .find(|prover_input| prover_input.public_image() == us.proposition)
                    {
                        let z = dlog_protocol::interactive_prover::second_message(
                            &priv_key,
                            us.randomness_opt.unwrap(),
                            &challenge,
                        );
                        Ok(UncheckedSchnorr {
                            proposition: us.proposition,
                            commitment_opt: None,
                            challenge,
                            second_message: z,
                        }
                        .into())
                    } else {
                        Err(ProverError::SecretNotFound)
                    }
                } else {
                    Err(ProverError::RealUnprovenTreeWithoutChallenge)
                }
            }
            _ => todo!(),
        },
    }
}

fn convert_to_unproven(sb: SigmaBoolean) -> UnprovenTree {
    match sb {
        SigmaBoolean::ProofOfKnowledge(pok) => match pok {
            SigmaProofOfKnowledgeTree::ProveDhTuple(_) => todo!(),
            SigmaProofOfKnowledgeTree::ProveDlog(prove_dlog) => UnprovenSchnorr {
                proposition: prove_dlog,
                commitment_opt: None,
                randomness_opt: None,
                challenge_opt: None,
                simulated: false,
            }
            .into(),
        },
        SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(Cand { items: sigma_trees })) => {
            CandUnproven {
                proposition: sigma_trees.clone(),
                challenge_opt: None,
                simulated: false,
                children: sigma_trees.into_iter().map(convert_to_unproven).collect(),
                position: NodePosition::crypto_tree_prefix(),
            }
            .into()
        }
        SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(_)) => todo!(),
        SigmaBoolean::TrivialProp(_) => panic!("TrivialProp is not expected here"),
    }
}

/// Test prover implementation
pub struct TestProver {
    /// secrets to be used in proofs generation
    pub secrets: Vec<PrivateInput>,
}

impl Evaluator for TestProver {}
impl Prover for TestProver {
    fn secrets(&self) -> &[PrivateInput] {
        self.secrets.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma_protocol::private_input::DlogProverInput;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::sigma_and::SigmaAnd;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    #[test]
    fn test_prove_true_prop() {
        let bool_true_tree = ErgoTree::from(Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(true),
        }));
        let message = vec![0u8; 100];

        let prover = TestProver { secrets: vec![] };
        let res = prover.prove(
            &bool_true_tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            message.as_slice(),
            HintsBag::empty(),
        );
        assert!(res.is_ok());
        assert_eq!(res.unwrap().proof, ProofBytes::Empty);
    }

    #[test]
    fn test_prove_false_prop() {
        let bool_false_tree = ErgoTree::from(Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Value::Boolean(false),
        }));
        let message = vec![0u8; 100];

        let prover = TestProver { secrets: vec![] };
        let res = prover.prove(
            &bool_false_tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            message.as_slice(),
            HintsBag::empty(),
        );
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ProverError::ReducedToFalse);
    }

    #[test]
    fn test_prove_pk_prop() {
        let secret = DlogProverInput::random();
        let pk = secret.public_image();
        let tree = ErgoTree::from(Expr::Const(pk.into()));
        let message = vec![0u8; 100];

        let prover = TestProver {
            secrets: vec![PrivateInput::DlogProverInput(secret)],
        };
        let res = prover.prove(
            &tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            message.as_slice(),
            HintsBag::empty(),
        );
        assert!(res.is_ok());
        assert_ne!(res.unwrap().proof, ProofBytes::Empty);
    }

    #[test]
    fn test_prove_pk_and_pk() {
        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let expr: Expr = SigmaAnd::new(vec![Expr::Const(pk1.into()), Expr::Const(pk2.into())])
            .unwrap()
            .into();
        let tree: ErgoTree = expr.into();
        let message = vec![0u8; 100];

        let prover = TestProver {
            secrets: vec![secret1.into(), secret2.into()],
        };
        let res = prover.prove(
            &tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            message.as_slice(),
            HintsBag::empty(),
        );
        assert!(res.is_ok());
        assert_ne!(res.unwrap().proof, ProofBytes::Empty);
    }
}
