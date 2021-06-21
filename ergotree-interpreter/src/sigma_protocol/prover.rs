//! Interpreter with enhanced functionality to prove statements.

mod context_extension;
mod prover_result;

pub mod hint;

use crate::sigma_protocol::fiat_shamir::fiat_shamir_hash_fn;
use crate::sigma_protocol::fiat_shamir::fiat_shamir_tree_to_bytes;
use crate::sigma_protocol::proof_tree::ProofTree;
use crate::sigma_protocol::unchecked_tree::UncheckedLeaf;
use crate::sigma_protocol::unproven_tree::CandUnproven;
use crate::sigma_protocol::unproven_tree::CorUnproven;
use crate::sigma_protocol::unproven_tree::NodePosition;
use crate::sigma_protocol::Challenge;
use crate::sigma_protocol::UncheckedSigmaTree;
use crate::sigma_protocol::UnprovenLeaf;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use std::convert::TryInto;
use std::rc::Rc;

pub use context_extension::*;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::ergo_tree::ErgoTreeParsingError;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjecture;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
pub use prover_result::*;

use self::hint::HintsBag;

use super::dlog_protocol;
use super::fiat_shamir::FiatShamirTreeSerializationError;
use super::private_input::PrivateInput;
use super::proof_tree;
use super::proof_tree::ProofTreeLeaf;
use super::sig_serializer::serialize_sig;
use super::unchecked_tree::UncheckedConjecture;
use super::unchecked_tree::UncheckedSchnorr;
use super::unchecked_tree::UncheckedTree;
use super::unproven_tree::UnprovenConjecture;
use super::unproven_tree::UnprovenSchnorr;
use super::unproven_tree::UnprovenTree;

use crate::eval::context::Context;
use crate::eval::env::Env;
use crate::eval::{EvalError, Evaluator};

use derive_more::From;
use thiserror::Error;

/// Prover errors
#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
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
    /// Unexpected value encountered
    #[error("Unexpected: {0}")]
    Unexpected(String),
    /// Error while tree serialization for Fiat-Shamir hash
    #[error("Fiat-Shamir tree serialization error: {0}")]
    FiatShamirTreeSerializationError(FiatShamirTreeSerializationError),
}

/// Prover
pub trait Prover: Evaluator {
    /// Secrets of the prover
    fn secrets(&self) -> &[PrivateInput];

    /// The comments in this section are taken from the algorithm for the
    /// Sigma-protocol prover as described in the ErgoScript white-paper
    /// <https://ergoplatform.org/docs/ErgoScript.pdf>, Appendix A
    ///  
    /// Generate proofs for the given message for ErgoTree reduced to Sigma boolean expression
    fn prove(
        &self,
        tree: &ErgoTree,
        env: &Env,
        ctx: Rc<Context>,
        message: &[u8],
        hints_bag: &HintsBag,
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
/// <https://ergoplatform.org/docs/ErgoScript.pdf> (Appendix A)
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
    hints_bag: &HintsBag,
) -> Result<UncheckedSigmaTree, ProverError> {
    // Prover Step 1: Mark as real everything the prover can prove
    let step1 = mark_real(prover, unproven_tree, hints_bag)?;
    dbg!(&step1);

    // Prover Step 2: If the root of the tree is marked "simulated" then the prover does not have enough witnesses
    // to perform the proof. Abort.
    if !step1.is_real() {
        return Err(ProverError::TreeRootIsNotReal);
    }

    // Prover Step 3: Change some "real" nodes to "simulated" to make sure each node
    // has the right number of simulated children.

    let step3 = polish_simulated(prover, step1)?;
    dbg!(&step3);

    // Prover Steps 4, 5, and 6 together: find challenges for simulated nodes; simulate simulated leaves;
    // compute commitments for real leaves
    let step6 = simulate_and_commit(step3, hints_bag)?;
    dbg!(&step6);

    // Prover Steps 7: convert the relevant information in the tree (namely, tree structure, node types,
    // the statements being proven and commitments at the leaves)
    // to a string
    let var_name = fiat_shamir_tree_to_bytes(&step6.clone().into())?;
    let mut s = var_name;

    // Prover Step 8: compute the challenge for the root of the tree as the Fiat-Shamir hash of s
    // and the message being signed.
    s.append(&mut message.to_vec());
    let root_challenge: Challenge = fiat_shamir_hash_fn(s.as_slice()).into();
    let step8 = step6.with_challenge(root_challenge);
    dbg!(&step8);

    // Prover Step 9: complete the proof by computing challenges at real nodes and additionally responses at real leaves
    let step9 = proving(prover, step8.into(), hints_bag)?;
    dbg!(&step9);
    // Prover Step 10: output the right information into the proof
    convert_to_unchecked(step9)
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
    hints_bag: &HintsBag,
) -> Result<UnprovenTree, ProverError> {
    proof_tree::rewrite(unproven_tree.into(), &|tree| {
        Ok(match tree {
            ProofTree::UnprovenTree(unp) => match unp {
                UnprovenTree::UnprovenLeaf(unp_leaf) => match unp_leaf {
                    UnprovenLeaf::UnprovenSchnorr(us) => {
                        // If the node is a leaf, mark it "real'' if either the witness for it is
                        // available or a hint shows the secret is known to an external participant in multi-signing;
                        // else mark it "simulated"
                        let secret_known =
                            hints_bag.real_images().contains(&unp_leaf.proposition())
                                || prover.secrets().iter().any(|s| match s {
                                    PrivateInput::DlogProverInput(dl) => {
                                        dl.public_image() == us.proposition
                                    }
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
                },
                UnprovenTree::UnprovenConjecture(unp_conj) => match unp_conj {
                    UnprovenConjecture::CandUnproven(cand) => {
                        // If the node is AND, mark it "real" if all of its children are marked real; else mark it "simulated"
                        let simulated = cast_to_unp(cand.children.clone())?
                            .iter()
                            .any(|c| c.simulated());
                        Some(
                            CandUnproven {
                                simulated,
                                ..cand.clone()
                            }
                            .into(),
                        )
                    }
                    UnprovenConjecture::CorUnproven(cor) => {
                        // If the node is OR, mark it "real" if at least one child is marked real; else mark it "simulated"
                        let simulated = cast_to_unp(cor.children.clone())?
                            .iter()
                            .all(|c| c.simulated());
                        Some(
                            CorUnproven {
                                simulated,
                                ..cor.clone()
                            }
                            .into(),
                        )
                    }
                },
            },
            ProofTree::UncheckedTree(_) => None,
        })
    })?
    .try_into()
    .map_err(|e: &str| ProverError::Unexpected(e.to_string()))
}

/// Set positions for children of a unproven inner node (conjecture, so AND/OR/THRESHOLD)
fn set_positions(uc: UnprovenConjecture) -> UnprovenConjecture {
    let upd_children = uc
        .children()
        .enumerated()
        .mapped(|(idx, utree)| utree.with_position(uc.position().child(idx)));
    match uc {
        UnprovenConjecture::CandUnproven(cand) => cand.with_children(upd_children).into(),
        UnprovenConjecture::CorUnproven(cor) => cor.with_children(upd_children).into(),
    }
}

/// If the node is OR marked "real",  mark all but one of its children "simulated"
/// (the node is guaranteed by step 1 to have at least one "real" child).
/// Which particular child is left "real" is not important for security;
/// the choice can be guided by efficiency or convenience considerations.
fn make_cor_children_simulated(cor: CorUnproven) -> Result<CorUnproven, ProverError> {
    let casted_children = cast_to_unp(cor.children)?;
    let first_real_child = casted_children
        .iter()
        .find(|it| it.is_real())
        .ok_or_else(|| {
            ProverError::Unexpected(format!(
                "make_cor_children_simulated: no real child is found amoung: {:?}",
                casted_children
            ))
        })?;
    let children = casted_children
        .clone()
        .mapped(|c| {
            if &c == first_real_child || c.simulated() {
                c
            } else {
                c.with_simulated(true)
            }
        })
        .mapped(|c| c.into());
    Ok(CorUnproven { children, ..cor })
}

fn cast_to_unp(
    children: SigmaConjectureItems<ProofTree>,
) -> Result<SigmaConjectureItems<UnprovenTree>, ProverError> {
    children.try_mapped(|c| {
        if let ProofTree::UnprovenTree(ut) = c {
            Ok(ut)
        } else {
            Err(ProverError::Unexpected(format!(
                "make_cor_children_simulated: expected UnprovenTree got: {:?}",
                c
            )))
        }
    })
}

/// Prover Step 3: This step will change some "real" nodes to "simulated" to make sure each node has
/// the right number of simulated children.
/// In a top-down traversal of the tree, do the following for each node:
fn polish_simulated<P: Prover + ?Sized>(
    _prover: &P,
    unproven_tree: UnprovenTree,
) -> Result<UnprovenTree, ProverError> {
    proof_tree::rewrite(unproven_tree.into(), &|tree| match tree {
        ProofTree::UnprovenTree(ut) => match ut {
            UnprovenTree::UnprovenLeaf(_) => Ok(None),
            UnprovenTree::UnprovenConjecture(conj) => match conj {
                UnprovenConjecture::CandUnproven(cand) => {
                    // If the node is marked "simulated", mark all of its children "simulated"
                    let a: CandUnproven = if cand.simulated {
                        cand.clone().with_children(
                            cast_to_unp(cand.children.clone())?
                                .mapped(|c| c.with_simulated(true).into()),
                        )
                    } else {
                        cand.clone()
                    };
                    Ok(Some(set_positions(a.into()).into()))
                }
                UnprovenConjecture::CorUnproven(cor) => {
                    // If the node is marked "simulated", mark all of its children "simulated"
                    let o: CorUnproven = if cor.simulated {
                        CorUnproven {
                            children: cast_to_unp(cor.children.clone())?
                                .mapped(|c| c.with_simulated(true).into()),
                            ..cor.clone()
                        }
                    } else {
                        // If the node is OR marked "real",  mark all but one of its children "simulated"
                        make_cor_children_simulated(cor.clone())?
                    };
                    Ok(Some(set_positions(o.into()).into()))
                }
            },
        },
        ProofTree::UncheckedTree(_) => Ok(None),
    })?
    .try_into()
    .map_err(|e: &str| ProverError::Unexpected(e.to_string()))
}

/**
 Prover Step 4: In a top-down traversal of the tree, compute the challenges e for simulated children of every node
 Prover Step 5: For every leaf marked "simulated", use the simulator of the Sigma-protocol for that leaf
 to compute the commitment $a$ and the response z, given the challenge e that is already stored in the leaf.
 Prover Step 6: For every leaf marked "real", use the first prover step of the Sigma-protocol for that leaf to
 compute the commitment a.
*/
fn simulate_and_commit(
    unproven_tree: UnprovenTree,
    hints_bag: &HintsBag,
) -> Result<UnprovenTree, ProverError> {
    proof_tree::rewrite(unproven_tree.into(), &|tree| {
        match tree {
            // Step 4 part 1: If the node is marked "real", then each of its simulated children gets a fresh uniformly
            // random challenge in {0,1}^t.

            // A real AND node has no simulated children
            ProofTree::UnprovenTree(UnprovenTree::UnprovenConjecture(
                UnprovenConjecture::CandUnproven(cand),
            )) if cand.is_real() => Ok(None),

            //real OR
            ProofTree::UnprovenTree(UnprovenTree::UnprovenConjecture(
                UnprovenConjecture::CorUnproven(cor),
            )) if cor.is_real() => {
                let new_children = cast_to_unp(cor.children.clone())?
                    .mapped(|c| {
                        if c.is_real() {
                            c
                        } else {
                            // take challenge from previously done proof stored in the hints bag,
                            // or generate random challenge for simulated child
                            let new_challenge: Challenge = hints_bag
                                .proofs()
                                .into_iter()
                                .find(|p| p.position() == c.position())
                                .map(|p| p.challenge().clone())
                                .unwrap_or_else(Challenge::secure_random);
                            c.with_challenge(new_challenge)
                        }
                    })
                    .mapped(|c| c.into());
                Ok(Some(
                    CorUnproven {
                        children: new_children,
                        ..cor.clone()
                    }
                    .into(),
                ))
            }

            // Step 4 part 2: If the node is marked "simulated", let e_0 be the challenge computed for it.
            // All of its children are simulated, and thus we compute challenges for all
            // of them, as follows:
            ProofTree::UnprovenTree(UnprovenTree::UnprovenConjecture(
                UnprovenConjecture::CandUnproven(cand),
            )) => {
                // If the node is AND, then all of its children get e_0 as the challenge
                if let Some(challenge) = cand.challenge_opt.clone() {
                    let new_children = cand
                        .children
                        .clone()
                        .mapped(|it| it.with_challenge(challenge.clone()));
                    Ok(Some(
                        CandUnproven {
                            children: new_children,
                            ..cand.clone()
                        }
                        .into(),
                    ))
                } else {
                    Err(ProverError::Unexpected(
                        "simulate_and_commit: missing CandUnproven(simulated).challenge"
                            .to_string(),
                    ))
                }
            }

            ProofTree::UnprovenTree(UnprovenTree::UnprovenConjecture(
                UnprovenConjecture::CorUnproven(cor),
            )) => {
                // If the node is OR, then each of its children except one gets a fresh uniformly random
                // challenge in {0,1}^t. The remaining child gets a challenge computed as an XOR of the challenges of all
                // the other children and e_0.
                if let Some(challenge) = cor.challenge_opt.clone() {
                    let unproven_children = cast_to_unp(cor.children.clone())?;
                    let mut tail: Vec<UnprovenTree> = unproven_children
                        .clone()
                        .into_iter()
                        .skip(1)
                        .map(|it| it.with_challenge(Challenge::secure_random()))
                        .collect();
                    let mut xored_challenge = challenge;
                    for it in &tail {
                        xored_challenge = xored_challenge.xor(it.challenge().ok_or_else(|| {
                            ProverError::Unexpected(format!("no challenge in {:?}", it))
                        })?);
                    }
                    let head = unproven_children
                        .first()
                        .clone()
                        .with_challenge(xored_challenge);
                    let mut new_children = vec![head];
                    new_children.append(&mut tail);
                    #[allow(clippy::unwrap_used)] // since quantity is preserved unwrap is safe here
                    Ok(Some(
                        CorUnproven {
                            children: new_children
                                .into_iter()
                                .map(|c| c.into())
                                .collect::<Vec<ProofTree>>()
                                .try_into()
                                .unwrap(),
                            ..cor.clone()
                        }
                        .into(),
                    ))
                } else {
                    Err(ProverError::Unexpected(
                        "simulate_and_commit: missing CandUnproven(simulated).challenge"
                            .to_string(),
                    ))
                }
            }

            ProofTree::UnprovenTree(UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(
                us,
            ))) => {
                // Steps 5 & 6: first try pulling out commitment from the hints bag. If it exists proceed with it,
                // otherwise, compute the commitment (if the node is real) or simulate it (if the node is simulated)

                // Step 6 (real leaf -- compute the commitment a or take it from the hints bag)
                let res: ProofTree = match hints_bag
                    .commitments()
                    .into_iter()
                    .find(|c| c.position() == us.position)
                {
                    Some(cmt_hint) => {
                        let pt: ProofTree = UnprovenSchnorr {
                            commitment_opt: Some(
                                cmt_hint
                                    .commitment()
                                    .try_into()
                                    .map_err(|e: &str| ProverError::Unexpected(e.to_string()))?,
                            ),
                            ..us.clone()
                        }
                        .into();
                        pt
                    }
                    None => {
                        if us.simulated {
                            // Step 5 (simulated leaf -- complete the simulation)
                            if let Some(challenge) = us.challenge_opt.clone() {
                                let (fm, sm) = dlog_protocol::interactive_prover::simulate(
                                    &us.proposition,
                                    &challenge,
                                );
                                Ok(ProofTree::UncheckedTree(
                                    UncheckedSchnorr {
                                        proposition: us.proposition.clone(),
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
                            let (r, commitment) =
                                dlog_protocol::interactive_prover::first_message();
                            Ok(ProofTree::UnprovenTree(
                                UnprovenSchnorr {
                                    commitment_opt: Some(commitment),
                                    randomness_opt: Some(r),
                                    ..us.clone()
                                }
                                .into(),
                            ))
                        }?
                    }
                };
                Ok(Some(res))
            }
            ProofTree::UncheckedTree(_) => Ok(None),
        }
    })?
    .try_into()
    .map_err(|e: &str| ProverError::Unexpected(e.to_string()))
}

/**
 Prover Step 9: Perform a top-down traversal of only the portion of the tree marked "real" in order to compute
 the challenge e for every node marked "real" below the root and, additionally, the response z for every leaf
 marked "real"
*/
fn proving<P: Prover + ?Sized>(
    prover: &P,
    proof_tree: ProofTree,
    hints_bag: &HintsBag,
) -> Result<ProofTree, ProverError> {
    proof_tree::rewrite(proof_tree, &|tree| {
        match &tree {
            ProofTree::UncheckedTree(unch) => match unch {
                UncheckedTree::NoProof => Err(ProverError::Unexpected(
                    "Unexpected NoProof in proving()".to_string(),
                )),
                UncheckedTree::UncheckedSigmaTree(ust) => match ust {
                    UncheckedSigmaTree::UncheckedLeaf(_) => Ok(None),
                    UncheckedSigmaTree::UncheckedConjecture(_) => Err(ProverError::Unexpected(
                        format!("proving: unexpected {:?}", tree),
                    )),
                },
            },
            ProofTree::UnprovenTree(unproven_tree) => match unproven_tree {
                UnprovenTree::UnprovenConjecture(conj) => match conj {
                    UnprovenConjecture::CandUnproven(cand) => {
                        if cand.is_real() {
                            // If the node is AND, let each of its children have the challenge e_0
                            if let Some(challenge) = cand.challenge_opt.clone() {
                                let updated = cand
                                    .clone()
                                    .children
                                    .mapped(|child| child.with_challenge(challenge.clone()));
                                Ok(Some(cand.clone().with_children(updated).into()))
                            } else {
                                Err(ProverError::Unexpected(
                                    "proving: CandUnproven.challenge_opt is empty".to_string(),
                                ))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    UnprovenConjecture::CorUnproven(cor) => {
                        // If the node is OR, it has only one child marked "real".
                        // Let this child have the challenge equal to the XOR of the challenges of all
                        // the other children and e_0
                        if cor.is_real() {
                            if let Some(root_challenge) = &cor.challenge_opt {
                                let challenge: Challenge = cor
                                    .children
                                    .clone()
                                    .iter()
                                    .flat_map(|c| c.challenge())
                                    .fold(root_challenge.clone(), |acc, c| acc.xor(c));
                                let children = cor.children.clone().mapped(|c| match c {
                                    ProofTree::UnprovenTree(ref ut) if ut.is_real() => {
                                        c.with_challenge(challenge.clone())
                                    }
                                    _ => c,
                                });
                                Ok(Some(
                                    CorUnproven {
                                        children,
                                        ..cor.clone()
                                    }
                                    .into(),
                                ))
                            } else {
                                Err(ProverError::Unexpected(
                                    "proving: CorUnproven.challenge_opt is empty".to_string(),
                                ))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                },

                // If the node is a leaf marked "real", compute its response according to the second prover step
                // of the Sigma-protocol given the commitment, challenge, and witness, or pull response from the hints bag
                UnprovenTree::UnprovenLeaf(unp_leaf) if unp_leaf.is_real() => match unp_leaf {
                    UnprovenLeaf::UnprovenSchnorr(us) => {
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
                                    us.randomness_opt.ok_or_else(|| {
                                        ProverError::Unexpected(format!(
                                            "empty randomness in {:?}",
                                            us
                                        ))
                                    })?,
                                    &challenge,
                                );
                                Ok(Some(
                                    UncheckedSchnorr {
                                        proposition: us.proposition.clone(),
                                        commitment_opt: None,
                                        challenge,
                                        second_message: z,
                                    }
                                    .into(),
                                ))
                            } else {
                                Err(ProverError::SecretNotFound)
                            }
                        } else {
                            Err(ProverError::RealUnprovenTreeWithoutChallenge)
                        }
                    }
                },
                UnprovenTree::UnprovenLeaf(unp_leaf) => {
                    // if the simulated node is proven by someone else, take it from hints bag
                    let res: ProofTree = hints_bag
                        .simulated_proofs()
                        .into_iter()
                        .find(|proof| proof.image == unp_leaf.proposition())
                        .map(|proof| proof.unchecked_tree.into())
                        .unwrap_or_else(|| unp_leaf.clone().into());
                    Ok(Some(res))
                }
            },
        }
    })
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
                position: NodePosition::crypto_tree_prefix(),
            }
            .into(),
        },
        SigmaBoolean::SigmaConjecture(conj) => match conj {
            SigmaConjecture::Cand(cand) => CandUnproven {
                proposition: cand.clone(),
                challenge_opt: None,
                simulated: false,
                children: cand.items.mapped(|it| convert_to_unproven(it).into()),
                position: NodePosition::crypto_tree_prefix(),
            }
            .into(),
            SigmaConjecture::Cor(cor) => CorUnproven {
                proposition: cor.clone(),
                challenge_opt: None,
                simulated: false,
                children: cor.items.mapped(|it| convert_to_unproven(it).into()),
                position: NodePosition::crypto_tree_prefix(),
            }
            .into(),
            SigmaConjecture::Cthreshold(_) => todo!(),
        },
        SigmaBoolean::TrivialProp(_) => panic!("TrivialProp is not expected here"),
    }
}

fn convert_to_unchecked(tree: ProofTree) -> Result<UncheckedSigmaTree, ProverError> {
    match &tree {
        ProofTree::UncheckedTree(unch_tree) => match unch_tree {
            UncheckedTree::NoProof => Err(ProverError::Unexpected(
                "convert_to_unchecked: unexpected NoProof".to_string(),
            )),
            UncheckedTree::UncheckedSigmaTree(ust) => match ust {
                UncheckedSigmaTree::UncheckedLeaf(ul) => match ul {
                    UncheckedLeaf::UncheckedSchnorr(_) => Ok(ust.clone()),
                },
                UncheckedSigmaTree::UncheckedConjecture(_) => Err(ProverError::Unexpected(
                    format!("convert_to_unchecked: unexpected {:?}", tree),
                )),
            },
        },
        ProofTree::UnprovenTree(unp_tree) => match unp_tree {
            UnprovenTree::UnprovenLeaf(_) => Err(ProverError::Unexpected(format!(
                "convert_to_unchecked: unexpected {:?}",
                tree
            ))),
            UnprovenTree::UnprovenConjecture(conj) => match conj {
                UnprovenConjecture::CandUnproven(cand) => Ok(UncheckedConjecture::CandUnchecked {
                    challenge: cand.challenge_opt.clone().ok_or_else(|| {
                        ProverError::Unexpected(format!("no challenge in {:?}", cand))
                    })?,
                    children: cand.children.clone().try_mapped(convert_to_unchecked)?,
                }
                .into()),
                UnprovenConjecture::CorUnproven(cor) => Ok(UncheckedConjecture::CorUnchecked {
                    challenge: cor.challenge_opt.clone().ok_or_else(|| {
                        ProverError::Unexpected(format!("no challenge in {:?}", cor))
                    })?,
                    children: cor.children.clone().try_mapped(convert_to_unchecked)?,
                }
                .into()),
            },
        },
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

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma_protocol::private_input::DlogProverInput;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::sigma_and::SigmaAnd;
    use ergotree_ir::mir::sigma_or::SigmaOr;
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
            &HintsBag::empty(),
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
            &HintsBag::empty(),
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
            &HintsBag::empty(),
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
            &HintsBag::empty(),
        );
        assert_ne!(res.unwrap().proof, ProofBytes::Empty);
    }

    #[test]
    fn test_prove_pk_and_or() {
        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let secret3 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let pk3 = secret3.public_image();
        let expr: Expr = SigmaAnd::new(vec![
            Expr::Const(pk1.into()),
            SigmaOr::new(vec![Expr::Const(pk2.into()), Expr::Const(pk3.into())])
                .unwrap()
                .into(),
        ])
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
            &HintsBag::empty(),
        );
        assert_ne!(res.unwrap().proof, ProofBytes::Empty);
    }

    #[test]
    fn test_prove_pk_or_pk() {
        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let expr: Expr = SigmaOr::new(vec![Expr::Const(pk1.into()), Expr::Const(pk2.into())])
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
            &HintsBag::empty(),
        );
        assert_ne!(res.unwrap().proof, ProofBytes::Empty);
    }

    #[test]
    fn test_prove_pk_or_and() {
        let secret1 = DlogProverInput::random();
        let secret2 = DlogProverInput::random();
        let secret3 = DlogProverInput::random();
        let pk1 = secret1.public_image();
        let pk2 = secret2.public_image();
        let pk3 = secret3.public_image();
        let expr: Expr = SigmaOr::new(vec![
            Expr::Const(pk1.into()),
            SigmaAnd::new(vec![Expr::Const(pk2.into()), Expr::Const(pk3.into())])
                .unwrap()
                .into(),
        ])
        .unwrap()
        .into();
        let tree: ErgoTree = expr.into();
        let message = vec![0u8; 100];

        let prover = TestProver {
            secrets: vec![secret2.into(), secret3.into()],
        };
        let res = prover.prove(
            &tree,
            &Env::empty(),
            Rc::new(force_any_val::<Context>()),
            message.as_slice(),
            &HintsBag::empty(),
        );
        assert_ne!(res.unwrap().proof, ProofBytes::Empty);
    }
}
