//! Interpreter with enhanced functionality to prove statements.

mod context_extension;
mod prover_result;

pub mod hint;

use crate::eval::reduce_to_crypto;
use crate::sigma_protocol::crypto_utils::secure_random_bytes;
use crate::sigma_protocol::dht_protocol;
use crate::sigma_protocol::fiat_shamir::fiat_shamir_hash_fn;
use crate::sigma_protocol::fiat_shamir::fiat_shamir_tree_to_bytes;
use crate::sigma_protocol::gf2_192::gf2_192poly_from_byte_array;
use crate::sigma_protocol::proof_tree::ProofTree;
use crate::sigma_protocol::unchecked_tree::UncheckedDhTuple;
use crate::sigma_protocol::unproven_tree::CandUnproven;
use crate::sigma_protocol::unproven_tree::CorUnproven;
use crate::sigma_protocol::unproven_tree::NodePosition;
use crate::sigma_protocol::unproven_tree::UnprovenDhTuple;
use crate::sigma_protocol::Challenge;
use crate::sigma_protocol::UnprovenLeaf;
use crate::sigma_protocol::SOUNDNESS_BYTES;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use gf2_192::gf2_192poly::Gf2_192Poly;
use gf2_192::gf2_192poly::Gf2_192PolyError;
use gf2_192::Gf2_192Error;
use std::convert::TryInto;
use std::rc::Rc;

pub use context_extension::*;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::ergo_tree::ErgoTreeError;
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
use super::unproven_tree::CthresholdUnproven;
use super::unproven_tree::UnprovenConjecture;
use super::unproven_tree::UnprovenSchnorr;
use super::unproven_tree::UnprovenTree;
use super::FirstProverMessage::FirstDhtProverMessage;
use super::FirstProverMessage::FirstDlogProverMessage;

use crate::eval::context::Context;
use crate::eval::env::Env;
use crate::eval::EvalError;

use thiserror::Error;

/// Prover errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ProverError {
    /// Failed to parse ErgoTree
    #[error("Ergo tree error: {0}")]
    ErgoTreeError(ErgoTreeError),
    /// Failed to evaluate ErgoTree
    #[error("Evaluation error: {0}")]
    EvalError(EvalError),
    /// `gf2_192` error
    #[error("gf2_192 error: {0}")]
    Gf2_192Error(Gf2_192Error),
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
    /// Not yet implemented
    #[error("not yet implemented: {0}")]
    NotYetImplemented(String),
}

impl From<ErgoTreeError> for ProverError {
    fn from(e: ErgoTreeError) -> Self {
        ProverError::ErgoTreeError(e)
    }
}

impl From<FiatShamirTreeSerializationError> for ProverError {
    fn from(e: FiatShamirTreeSerializationError) -> Self {
        ProverError::FiatShamirTreeSerializationError(e)
    }
}

impl From<Gf2_192Error> for ProverError {
    fn from(e: Gf2_192Error) -> Self {
        ProverError::Gf2_192Error(e)
    }
}

impl From<Gf2_192PolyError> for ProverError {
    fn from(e: Gf2_192PolyError) -> Self {
        ProverError::Gf2_192Error(Gf2_192Error::Gf2_192PolyError(e))
    }
}

/// Prover
pub trait Prover {
    /// Secrets of the prover
    fn secrets(&self) -> &[PrivateInput];

    /// Add an extra secret to the prover
    fn append_secret(&mut self, input: PrivateInput);

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
        let ctx_ext = ctx.extension.clone();
        let reduction_result =
            reduce_to_crypto(expr.as_ref(), env, ctx).map_err(ProverError::EvalError)?;
        self.generate_proof(reduction_result.sigma_prop, message, hints_bag, ctx_ext)
    }

    /// Generate proofs for the given message for the given Sigma boolean expression
    fn generate_proof(
        &self,
        sigmabool: SigmaBoolean,
        message: &[u8],
        hints_bag: &HintsBag,
        ctx_ext: ContextExtension,
    ) -> Result<ProverResult, ProverError> {
        let unchecked_tree_opt = match sigmabool {
            SigmaBoolean::TrivialProp(true) => Ok(None),
            SigmaBoolean::TrivialProp(false) => Err(ProverError::ReducedToFalse),
            sb => {
                let tree = convert_to_unproven(sb)?;
                let unchecked_tree = prove_to_unchecked(self, tree, message, hints_bag)?;
                Ok(Some(unchecked_tree))
            }
        }?;
        let proof = match unchecked_tree_opt {
            Some(tree) => serialize_sig(tree),
            None => ProofBytes::Empty,
        };
        Ok(ProverResult {
            proof,
            extension: ctx_ext,
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
) -> Result<UncheckedTree, ProverError> {
    // Prover Step 1: Mark as real everything the prover can prove
    let step1 = mark_real(prover, unproven_tree, hints_bag)?;
    // dbg!(&step1);

    // Prover Step 2: If the root of the tree is marked "simulated" then the prover does not have enough witnesses
    // to perform the proof. Abort.
    if !step1.is_real() {
        return Err(ProverError::TreeRootIsNotReal);
    }

    // Prover Step 3: Change some "real" nodes to "simulated" to make sure each node
    // has the right number of simulated children.

    let step3 = polish_simulated(prover, step1)?;
    // dbg!(&step3);

    // Prover Steps 4, 5, and 6 together: find challenges for simulated nodes; simulate simulated leaves;
    // compute commitments for real leaves
    let step6 = simulate_and_commit(step3, hints_bag)?;
    // dbg!(&step6);

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
    // dbg!(&step8);

    // Prover Step 9: complete the proof by computing challenges at real nodes and additionally responses at real leaves
    let step9 = proving(prover, step8.into(), hints_bag)?;
    // dbg!(&step9);
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
                UnprovenTree::UnprovenLeaf(unp_leaf) => {
                    // If the node is a leaf, mark it "real'' if either the witness for it is
                    // available or a hint shows the secret is known to an external participant in multi-signing;
                    // else mark it "simulated"
                    let secret_known = hints_bag.real_images().contains(&unp_leaf.proposition())
                        || prover
                            .secrets()
                            .iter()
                            .any(|s| s.public_image() == unp_leaf.proposition());
                    Some(unp_leaf.clone().with_simulated(!secret_known).into())
                }
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
                    UnprovenConjecture::CthresholdUnproven(ct) => {
                        // If the node is THRESHOLD(k), mark it "real" if at least k of its children are marked real; else mark it "simulated"
                        let simulated = cast_to_unp(ct.children.clone())?
                            .iter()
                            .filter(|c| c.simulated())
                            .count()
                            >= ct.k as usize;
                        Some(
                            CthresholdUnproven {
                                simulated,
                                ..ct.clone()
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
fn set_positions(uc: UnprovenConjecture) -> Result<UnprovenConjecture, ProverError> {
    let upd_children = uc
        .children()
        .try_mapped(|c| match c {
            ProofTree::UncheckedTree(unch) => Err(ProverError::Unexpected(format!(
                "set_positions: unexpected UncheckedTree: {:?}",
                unch
            ))),
            ProofTree::UnprovenTree(unp) => Ok(unp),
        })?
        .enumerated()
        .mapped(|(idx, utree)| utree.with_position(uc.position().child(idx)).into());
    Ok(match uc {
        UnprovenConjecture::CandUnproven(cand) => cand.with_children(upd_children).into(),
        UnprovenConjecture::CorUnproven(cor) => cor.with_children(upd_children).into(),
        UnprovenConjecture::CthresholdUnproven(ct) => ct.with_children(upd_children).into(),
    })
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
                    Ok(Some(set_positions(a.into())?.into()))
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
                    Ok(Some(set_positions(o.into())?.into()))
                }
                UnprovenConjecture::CthresholdUnproven(ct) => {
                    // If the node is marked "simulated", mark all of its children "simulated"
                    let t: CthresholdUnproven = if ct.simulated {
                        CthresholdUnproven {
                            children: cast_to_unp(ct.children.clone())?
                                .mapped(|c| c.with_simulated(true).into()),
                            ..ct.clone()
                        }
                    } else {
                        // If the node is THRESHOLD(k) marked "real", mark all but k of its children "simulated"
                        // (the node is guaranteed, by the previous step, to have at least k "real" children).
                        // Which particular ones are left "real" is not important for security;
                        // the choice can be guided by efficiency or convenience considerations.
                        //
                        // We'll mark the first k real ones real
                        let mut count_of_real = 0;
                        let mut children_indices_to_be_marked_simulated = Vec::new();
                        let unproven_children = cast_to_unp(ct.children.clone())?;
                        for (idx, kid) in unproven_children.clone().enumerated() {
                            if kid.is_real() {
                                count_of_real += 1;
                                if count_of_real >= ct.k {
                                    children_indices_to_be_marked_simulated.push(idx);
                                };
                            };
                        }
                        CthresholdUnproven {
                            children: unproven_children.enumerated().mapped(|(idx, c)| {
                                if children_indices_to_be_marked_simulated.contains(&idx) {
                                    c.with_simulated(true)
                                } else {
                                    c
                                }
                                .into()
                            }),
                            ..ct.clone()
                        }
                    };
                    Ok(Some(set_positions(t.into())?.into()))
                }
            },
        },
        ProofTree::UncheckedTree(_) => Ok(None),
    })?
    .try_into()
    .map_err(|e: &str| ProverError::Unexpected(e.to_string()))
}

fn step4_real_conj(
    uc: UnprovenConjecture,
    hints_bag: &HintsBag,
) -> Result<Option<ProofTree>, ProverError> {
    assert!(uc.is_real());
    match uc {
        // A real AND node has no simulated children
        UnprovenConjecture::CandUnproven(_) => Ok(None),
        //real OR Threshold case
        UnprovenConjecture::CorUnproven(_) | UnprovenConjecture::CthresholdUnproven(_) => {
            let new_children = cast_to_unp(uc.children())?
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
                    uc.with_children(new_children).into()
                    // CorUnproven {
                    //     children: new_children,
                    //     ..cor.clone()
                    // }
                    // .into(),
                ))
        }
    }
}

fn step4_simulated_and_conj(cand: CandUnproven) -> Result<Option<ProofTree>, ProverError> {
    assert!(cand.simulated);
    // If the node is AND, then all of its children get e_0 as the challenge
    if let Some(challenge) = cand.challenge_opt.clone() {
        let new_children = cand
            .children
            .clone()
            .mapped(|it| it.with_challenge(challenge.clone()));
        Ok(Some(
            CandUnproven {
                children: new_children,
                ..cand
            }
            .into(),
        ))
    } else {
        Err(ProverError::Unexpected(
            "simulate_and_commit: missing CandUnproven(simulated).challenge".to_string(),
        ))
    }
}

fn step4_simulated_or_conj(cor: CorUnproven) -> Result<Option<ProofTree>, ProverError> {
    // If the node is OR, then each of its children except one gets a fresh uniformly random
    // challenge in {0,1}^t. The remaining child gets a challenge computed as an XOR of the challenges of all
    // the other children and e_0.
    assert!(cor.simulated);
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
            xored_challenge = xored_challenge.xor(
                it.challenge()
                    .ok_or_else(|| ProverError::Unexpected(format!("no challenge in {:?}", it)))?,
            );
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
                ..cor
            }
            .into(),
        ))
    } else {
        Err(ProverError::Unexpected(
            "simulate_and_commit: missing CandUnproven(simulated).challenge".to_string(),
        ))
    }
}

fn step4_simulated_threshold_conj(
    ct: CthresholdUnproven,
) -> Result<Option<ProofTree>, ProverError> {
    // The faster algorithm is as follows. Pick n-k fresh uniformly random values
    // q_1, ..., q_{n-k} from {0,1}^t and let q_0=e_0.
    // Viewing 1, 2, ..., n and q_0, ..., q_{n-k} as elements of GF(2^t),
    // evaluate the polynomial Q(x) = sum {q_i x^i} over GF(2^t) at points 1, 2, ..., n
    // to get challenges for child 1, 2, ..., n, respectively.
    assert!(ct.simulated);
    if let Some(challenge) = ct.challenge_opt.clone() {
        let unproven_children = cast_to_unp(ct.children.clone())?;
        let n = ct.children.len();
        let q = gf2_192poly_from_byte_array(
            challenge,
            secure_random_bytes(SOUNDNESS_BYTES * (n - ct.k as usize)),
        )?;
        let new_children = unproven_children
            .enumerated()
            .mapped(|(idx, c)| {
                // Note the cast to `u8` is safe since `unproven_children` is of type
                // `SigmaConjectureItems<_>` which is a `BoundedVec<_, 2, 255>`.
                let one_based_idx = (idx + 1) as u8;
                let new_challenge = q.evaluate(one_based_idx).into();
                c.with_challenge(new_challenge)
            })
            .mapped(|c| c.into());
        Ok(Some(
            ct.with_polynomial(q).with_children(new_children).into(),
        ))
    } else {
        Err(ProverError::Unexpected(
            "simulate_and_commit: missing CthresholdUnproven(simulated).challenge".to_string(),
        ))
    }
}

fn step5_schnorr(
    us: UnprovenSchnorr,
    hints_bag: &HintsBag,
) -> Result<Option<ProofTree>, ProverError> {
    // Steps 5 & 6: first try pulling out commitment from the hints bag. If it exists proceed with it,
    // otherwise, compute the commitment (if the node is real) or simulate it (if the node is simulated)

    // Step 6 (real leaf -- compute the commitment a or take it from the hints bag)
    let res: ProofTree = match hints_bag
        .commitments()
        .into_iter()
        .find(|c| c.position() == &us.position)
    {
        Some(cmt_hint) => {
            let pt: ProofTree = UnprovenSchnorr {
                commitment_opt: Some(
                    cmt_hint
                        .commitment()
                        .clone()
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
                    let (fm, sm) =
                        dlog_protocol::interactive_prover::simulate(&us.proposition, &challenge);
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
                let (r, commitment) = dlog_protocol::interactive_prover::first_message();
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

fn step5_diffie_hellman_tuple(
    dhu: UnprovenDhTuple,
    hints_bag: &HintsBag,
) -> Result<Option<ProofTree>, ProverError> {
    //Steps 5 & 6: pull out commitment from the hints bag, otherwise, compute the commitment(if the node is real),
    // or simulate it (if the node is simulated)

    // Step 6 (real leaf -- compute the commitment a or take it from the hints bag)
    let res: Result<ProofTree, _> = hints_bag
        .commitments()
        .iter()
        .find(|c| c.position() == &dhu.position)
        .map(|cmt_hint| {
            Ok(dhu
                .clone()
                .with_commitment(match cmt_hint.commitment() {
                    FirstDlogProverMessage(_) => {
                        return Err(ProverError::Unexpected(
                            "Step 5 & 6 for UnprovenDhTuple: FirstDlogProverMessage is not expected here".to_string(),
                        ))
                    }
                    FirstDhtProverMessage(dhtm) => dhtm.clone(),
                })
                .into())
        })
        .unwrap_or_else(|| {
            if dhu.simulated {
                // Step 5 (simulated leaf -- complete the simulation)
                if let Some(dhu_challenge) = dhu.challenge_opt.clone() {
                    let (fm, sm) = dht_protocol::interactive_prover::simulate(
                        &dhu.proposition,
                        &dhu_challenge,
                    );
                    Ok(UncheckedDhTuple {
                        proposition: dhu.proposition.clone(),
                        commitment_opt: Some(fm),
                        challenge: dhu_challenge,
                        second_message: sm,
                    }
                        .into())
                } else {
                        Err(ProverError::SimulatedLeafWithoutChallenge)
                    }
            } else {
                // Step 6 -- compute the commitment
                let (r, fm) =
                dht_protocol::interactive_prover::first_message(&dhu.proposition);
                Ok(UnprovenDhTuple {
                    commitment_opt: Some(fm),
                    randomness_opt: Some(r),
                    ..dhu.clone()
                }
                    .into())
            }
        });
    Ok(Some(res?))
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
            // Step 4 part 1: If the node is marked "real", jhen each of its simulated children gets a fresh uniformly
            // random challenge in {0,1}^t.
            ProofTree::UnprovenTree(UnprovenTree::UnprovenConjecture(uc)) => {
                if uc.is_real() {
                    step4_real_conj(uc.clone(), hints_bag)
                } else {
                    match uc {
                        // Step 4 part 2: If the node is marked "simulated", let e_0 be the challenge computed for it.
                        // All of its children are simulated, and thus we compute challenges for all
                        // of them, as follows:
                        UnprovenConjecture::CandUnproven(cand) => {
                            step4_simulated_and_conj(cand.clone())
                        }
                        UnprovenConjecture::CorUnproven(cor) => {
                            step4_simulated_or_conj(cor.clone())
                        }
                        UnprovenConjecture::CthresholdUnproven(ct) => {
                            step4_simulated_threshold_conj(ct.clone())
                        }
                    }
                }
            }

            ProofTree::UnprovenTree(UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(
                us,
            ))) => step5_schnorr(us.clone(), hints_bag),

            ProofTree::UnprovenTree(UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenDhTuple(
                dhu,
            ))) => step5_diffie_hellman_tuple(dhu.clone(), hints_bag),
            ProofTree::UncheckedTree(_) => Ok(None),
        }
    })?
    .try_into()
    .map_err(|e: &str| ProverError::Unexpected(e.to_string()))
}

fn step9_real_and(cand: CandUnproven) -> Result<Option<ProofTree>, ProverError> {
    assert!(cand.is_real());
    // If the node is AND, let each of its children have the challenge e_0
    if let Some(challenge) = cand.challenge_opt.clone() {
        let updated = cand
            .clone()
            .children
            .mapped(|child| child.with_challenge(challenge.clone()));
        Ok(Some(cand.with_children(updated).into()))
    } else {
        Err(ProverError::Unexpected(
            "proving: CandUnproven.challenge_opt is empty".to_string(),
        ))
    }
}

fn step9_real_or(cor: CorUnproven) -> Result<Option<ProofTree>, ProverError> {
    assert!(cor.is_real());
    // If the node is OR, it has only one child marked "real".
    // Let this child have the challenge equal to the XOR of the challenges of all
    // the other children and e_0
    if let Some(root_challenge) = &cor.challenge_opt {
        let challenge: Challenge = cor
            .children
            .clone()
            .iter()
            .flat_map(|c| c.challenge())
            .fold(root_challenge.clone(), |acc, c| acc.xor(c));
        let children = cor.children.clone().mapped(|c| match c {
            ProofTree::UnprovenTree(ref ut) if ut.is_real() => c.with_challenge(challenge.clone()),
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
}

fn step9_real_threshold(ct: CthresholdUnproven) -> Result<Option<ProofTree>, ProverError> {
    assert!(ct.is_real());
    // If the node is THRESHOLD(k), number its children from 1 to no. Let i_1,..., i_{n-k}
    // be the indices of thechildren marked `"simulated" and e_1, ...,  e_{n-k} be
    // their corresponding challenges.
    // Let i_0 = 0. Viewing 0, 1, 2, ..., n and e_0, ..., e_{n-k} as elements of GF(2^t),
    // find (via polynomial interpolation) the lowest-degree polynomial
    // Q(x)=sum_{i=0}^{n-k} a_i x^i  over GF(2^t) that is equal to e_j at i_j
    // for each f from 0 to n-k
    // (this polynomial will have n-k+1 coefficients, and the lowest coefficient
    // will be e_0). For child number i of the node, if the child is marked "real",
    // compute its challenge as Q(i) (if the child is marked
    // "simulated", its challenge is already Q(i), by construction of Q).
    if let Some(challenge) = ct.challenge_opt.clone() {
        let mut points = Vec::new();
        let mut values = Vec::new();
        for (idx, child) in ct.children.clone().enumerated() {
            let one_based_idx = idx + 1;
            let challenge_opt = match child {
                ProofTree::UncheckedTree(ut) => match ut {
                    UncheckedTree::UncheckedLeaf(ul) => Some(ul.challenge()),
                    UncheckedTree::UncheckedConjecture(_) => None,
                },
                ProofTree::UnprovenTree(unpt) => unpt.challenge(),
            };
            if let Some(challenge) = challenge_opt {
                points.push(one_based_idx as u8);
                values.push(challenge.into());
            };
        }

        let value_at_zero = challenge.into();
        let q = Gf2_192Poly::interpolate(&points, &values, value_at_zero)?;

        let new_children = ct.children.clone().enumerated().mapped(|(idx, child)| {
            // Note the cast to `u8` is safe since `ct.children` is of type
            // `SigmaConjectureItems<_>` which is a `BoundedVec<_, 2, 255>`.
            let one_based_idx = (idx + 1) as u8;
            match &child {
                ProofTree::UnprovenTree(ut) if ut.is_real() => {
                    child.with_challenge(q.evaluate(one_based_idx).into())
                }
                _ => child,
            }
        });
        Ok(Some(
            ct.with_polynomial(q).with_children(new_children).into(),
        ))
    } else {
        Err(ProverError::Unexpected(
            "proving: CthresholdUnproven.challenge_opt is empty".to_string(),
        ))
    }
}

fn step9_real_schnorr<P: Prover + ?Sized>(
    us: UnprovenSchnorr,
    prover: &P,
) -> Result<Option<ProofTree>, ProverError> {
    assert!(us.is_real());
    // If the node is a leaf marked "real", compute its response according to the second prover step
    // of the Sigma-protocol given the commitment, challenge, and witness, or pull response from the hints bag
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
                priv_key,
                us.randomness_opt.ok_or_else(|| {
                    ProverError::Unexpected(format!("empty randomness in {:?}", us))
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

fn step9_real_dh_tuple<P: Prover + ?Sized>(
    dhu: UnprovenDhTuple,
    prover: &P,
    hints_bag: &HintsBag,
) -> Result<Option<ProofTree>, ProverError> {
    assert!(dhu.is_real());
    // If the node is a leaf marked "real", compute its response according to the second prover step
    // of the Sigma-protocol given the commitment, challenge, and witness, or pull response from
    // the hints bag
    if let Some(dhu_challenge) = dhu.challenge_opt.clone() {
        let priv_key_opt = prover
            .secrets()
            .iter()
            .find(|s| s.public_image() == dhu.proposition.clone().into());
        let z = match priv_key_opt {
            Some(PrivateInput::DhTupleProverInput(priv_key)) => match hints_bag
                .own_commitments()
                .iter()
                .find(|c| c.position == dhu.position)
            {
                Some(commitment_from_hints_bag) => {
                    dht_protocol::interactive_prover::second_message(
                        priv_key,
                        &commitment_from_hints_bag.secret_randomness,
                        &dhu_challenge,
                    )
                }
                None => dht_protocol::interactive_prover::second_message(
                    priv_key,
                    &dhu.randomness_opt.ok_or_else(|| {
                        ProverError::Unexpected(format!("empty randomness in {:?}", dhu))
                    })?,
                    &dhu_challenge,
                ),
            },
            Some(pi) => {
                return Err(ProverError::Unexpected(format!(
                    "Expected DH prover input in prover secrets, got {:?}",
                    pi
                )))
            }
            None => {
                return Err(ProverError::NotYetImplemented(
                    "when secret not found".to_string(),
                ))
            }
        };
        Ok(Some(
            UncheckedDhTuple {
                proposition: dhu.proposition.clone(),
                commitment_opt: None,
                challenge: dhu_challenge,
                second_message: z,
            }
            .into(),
        ))
    } else {
        Err(ProverError::RealUnprovenTreeWithoutChallenge)
    }
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
                UncheckedTree::UncheckedLeaf(_) => Ok(None),
                UncheckedTree::UncheckedConjecture(_) => Err(ProverError::Unexpected(format!(
                    "proving: unexpected {:?}",
                    tree
                ))),
            },

            ProofTree::UnprovenTree(unproven_tree) => match unproven_tree {
                UnprovenTree::UnprovenConjecture(conj) => {
                    if conj.is_real() {
                        match conj {
                            UnprovenConjecture::CandUnproven(cand) => step9_real_and(cand.clone()),
                            UnprovenConjecture::CorUnproven(cor) => step9_real_or(cor.clone()),
                            UnprovenConjecture::CthresholdUnproven(ct) => {
                                step9_real_threshold(ct.clone())
                            }
                        }
                    } else {
                        Ok(None)
                    }
                }

                UnprovenTree::UnprovenLeaf(unp_leaf) => {
                    if unp_leaf.is_real() {
                        match unp_leaf {
                            UnprovenLeaf::UnprovenSchnorr(us) => {
                                step9_real_schnorr(us.clone(), prover)
                            }
                            UnprovenLeaf::UnprovenDhTuple(dhu) => {
                                step9_real_dh_tuple(dhu.clone(), prover, hints_bag)
                            }
                        }
                    } else {
                        // if the simulated node is proven by someone else, take it from hints bag
                        let res: ProofTree = hints_bag
                            .simulated_proofs()
                            .into_iter()
                            .find(|proof| proof.image == unp_leaf.proposition())
                            .map(|proof| proof.unchecked_tree.into())
                            .unwrap_or_else(|| unp_leaf.clone().into());
                        Ok(Some(res))
                    }
                }
            },
        }
    })
}

fn convert_to_unproven(sb: SigmaBoolean) -> Result<UnprovenTree, ProverError> {
    Ok(match sb {
        SigmaBoolean::ProofOfKnowledge(pok) => match pok {
            SigmaProofOfKnowledgeTree::ProveDhTuple(pdht) => UnprovenDhTuple {
                proposition: pdht,
                commitment_opt: None,
                randomness_opt: None,
                challenge_opt: None,
                simulated: false,
                position: NodePosition::crypto_tree_prefix(),
            }
            .into(),
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
                children: cand
                    .items
                    .try_mapped(|it| convert_to_unproven(it).map(Into::into))?,
                position: NodePosition::crypto_tree_prefix(),
            }
            .into(),
            SigmaConjecture::Cor(cor) => CorUnproven {
                proposition: cor.clone(),
                challenge_opt: None,
                simulated: false,
                children: cor
                    .items
                    .try_mapped(|it| convert_to_unproven(it).map(Into::into))?,
                position: NodePosition::crypto_tree_prefix(),
            }
            .into(),
            SigmaConjecture::Cthreshold(ct) => CthresholdUnproven {
                proposition: ct.clone(),
                k: ct.k,
                children: ct
                    .children
                    .try_mapped(|it| convert_to_unproven(it).map(Into::into))?,
                polinomial_opt: None,
                challenge_opt: None,
                simulated: false,
                position: NodePosition::crypto_tree_prefix(),
            }
            .into(),
        },
        SigmaBoolean::TrivialProp(_) => {
            return Err(ProverError::Unexpected(
                "TrivialProp is not expected here".to_string(),
            ))
        }
    })
}

fn convert_to_unchecked(tree: ProofTree) -> Result<UncheckedTree, ProverError> {
    match &tree {
        ProofTree::UncheckedTree(unch_tree) => match unch_tree {
            UncheckedTree::UncheckedLeaf(_) => Ok(unch_tree.clone()),
            UncheckedTree::UncheckedConjecture(_) => Err(ProverError::Unexpected(format!(
                "convert_to_unchecked: unexpected {:?}",
                tree
            ))),
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
                UnprovenConjecture::CthresholdUnproven(ct) => {
                    Ok(UncheckedConjecture::CthresholdUnchecked {
                        challenge: ct.challenge_opt.clone().ok_or_else(|| {
                            ProverError::Unexpected(format!("no challenge in {:?}", ct))
                        })?,
                        children: ct.children.clone().try_mapped(convert_to_unchecked)?,
                        k: ct.k,
                        polynomial: ct.polinomial_opt.clone().ok_or_else(|| {
                            ProverError::Unexpected(format!("no polynomial in {:?}", ct))
                        })?,
                    }
                    .into())
                }
            },
        },
    }
}

/// Test prover implementation
pub struct TestProver {
    /// secrets to be used in proofs generation
    pub secrets: Vec<PrivateInput>,
}

impl Prover for TestProver {
    fn secrets(&self) -> &[PrivateInput] {
        self.secrets.as_ref()
    }

    fn append_secret(&mut self, input: PrivateInput) {
        self.secrets.push(input)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma_protocol::private_input::DhTupleProverInput;
    use crate::sigma_protocol::private_input::DlogProverInput;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::constant::Literal;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::sigma_and::SigmaAnd;
    use ergotree_ir::mir::sigma_or::SigmaOr;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;
    use std::convert::TryFrom;
    use std::rc::Rc;

    #[test]
    fn test_prove_true_prop() {
        let bool_true_tree = ErgoTree::try_from(Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(true),
        }))
        .unwrap();
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
        let bool_false_tree = ErgoTree::try_from(Expr::Const(Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(false),
        }))
        .unwrap();
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
        let tree = ErgoTree::try_from(Expr::Const(pk.into())).unwrap();
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
        let tree: ErgoTree = expr.try_into().unwrap();
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
        let tree: ErgoTree = expr.try_into().unwrap();
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
        let tree: ErgoTree = expr.try_into().unwrap();
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
        let tree: ErgoTree = expr.try_into().unwrap();
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

    #[test]
    fn test_prove_dht_prop() {
        let secret = DhTupleProverInput::random();
        let pi = secret.public_image();
        let tree = ErgoTree::try_from(Expr::Const(pi.clone().into())).unwrap();
        let message = vec![0u8; 100];

        let prover = TestProver {
            secrets: vec![PrivateInput::DhTupleProverInput(secret)],
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
}
