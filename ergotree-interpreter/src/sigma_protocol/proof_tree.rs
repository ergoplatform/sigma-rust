extern crate derive_more;
use derive_more::From;
use derive_more::TryInto;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

use crate::sigma_protocol::unproven_tree::CandUnproven;
use crate::sigma_protocol::unproven_tree::UnprovenConjecture;
use crate::sigma_protocol::UncheckedSchnorr;
use crate::sigma_protocol::UnprovenSchnorr;

use super::challenge::Challenge;
use super::prover::ProverError;
use super::unchecked_tree::UncheckedConjecture;
use super::unchecked_tree::UncheckedSigmaTree;
use super::unchecked_tree::UncheckedTree;
use super::unproven_tree::CorUnproven;
use super::unproven_tree::NodePosition;
use super::unproven_tree::UnprovenLeaf;
use super::unproven_tree::UnprovenTree;
use super::FirstProverMessage;

/// Proof tree
#[derive(PartialEq, Debug, Clone, From, TryInto)]
pub(crate) enum ProofTree {
    /// Unchecked tree
    UncheckedTree(UncheckedTree),
    /// Unproven tree
    UnprovenTree(UnprovenTree),
}

impl ProofTree {
    /// Create a new proof tree with a new challenge
    pub(crate) fn with_challenge(&self, challenge: Challenge) -> ProofTree {
        match self {
            ProofTree::UncheckedTree(_) => todo!(),
            ProofTree::UnprovenTree(ut) => ut.clone().with_challenge(challenge).into(),
        }
    }

    pub(crate) fn with_position(&self, updated: NodePosition) -> Self {
        match self {
            ProofTree::UncheckedTree(_) => todo!(),
            ProofTree::UnprovenTree(ut) => ut.clone().with_position(updated).into(),
        }
    }

    pub(crate) fn as_tree_kind(&self) -> ProofTreeKind {
        match self {
            ProofTree::UncheckedTree(unch) => unch.as_tree_kind(),
            ProofTree::UnprovenTree(unp) => unp.as_tree_kind(),
        }
    }

    pub(crate) fn position(&self) -> &NodePosition {
        todo!()
    }

    pub(crate) fn challenge(&self) -> Option<Challenge> {
        match self {
            ProofTree::UncheckedTree(unch) => unch.challenge(),
            ProofTree::UnprovenTree(unp) => unp.challenge(),
        }
    }
}

impl From<UncheckedSchnorr> for ProofTree {
    fn from(v: UncheckedSchnorr) -> Self {
        UncheckedTree::UncheckedSigmaTree(v.into()).into()
    }
}

impl From<UnprovenSchnorr> for ProofTree {
    fn from(v: UnprovenSchnorr) -> Self {
        UnprovenTree::UnprovenLeaf(v.into()).into()
    }
}

impl From<CandUnproven> for ProofTree {
    fn from(v: CandUnproven) -> Self {
        UnprovenTree::UnprovenConjecture(v.into()).into()
    }
}

impl From<CorUnproven> for ProofTree {
    fn from(v: CorUnproven) -> Self {
        UnprovenTree::UnprovenConjecture(v.into()).into()
    }
}

impl From<UnprovenConjecture> for ProofTree {
    fn from(v: UnprovenConjecture) -> Self {
        UnprovenTree::UnprovenConjecture(v).into()
    }
}

impl From<UnprovenLeaf> for ProofTree {
    fn from(v: UnprovenLeaf) -> Self {
        UnprovenTree::UnprovenLeaf(v).into()
    }
}

impl From<UncheckedSigmaTree> for ProofTree {
    fn from(ust: UncheckedSigmaTree) -> Self {
        ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(ust))
    }
}

impl From<UncheckedConjecture> for ProofTree {
    fn from(v: UncheckedConjecture) -> Self {
        UncheckedTree::UncheckedSigmaTree(v.into()).into()
    }
}

/// Proof tree leaf
pub(crate) trait ProofTreeLeaf {
    /// Get proposition
    fn proposition(&self) -> SigmaBoolean;

    /// Get commitment
    fn commitment_opt(&self) -> Option<FirstProverMessage>;
}

pub(crate) enum ConjectureType {
    And = 0,
    Or = 1,
    Threshold = 2,
}

pub(crate) trait ProofTreeConjecture {
    fn conjecture_type(&self) -> ConjectureType;
    fn children(&self) -> Vec<ProofTree>;
}

pub(crate) enum ProofTreeKind<'a> {
    Leaf(&'a dyn ProofTreeLeaf),
    Conjecture(&'a dyn ProofTreeConjecture),
}

pub(crate) fn rewrite_children<
    T: Into<ProofTree> + Clone,
    F: Fn(&ProofTree) -> Result<Option<ProofTree>, ProverError>,
>(
    children: Vec<T>,
    f: &F,
) -> Result<Vec<ProofTree>, ProverError> {
    children
        .into_iter()
        .map(|c| {
            f(&c.clone().into()).map(|opt| {
                // recursively rewrite the underlying tree
                if let Some(tree) = opt {
                    rewrite(tree, f)
                } else {
                    rewrite(c.into(), f)
                }
            })
        })
        .flatten()
        .collect()
}

pub(crate) fn cast_to_ust(
    children: Vec<ProofTree>,
) -> Result<Vec<UncheckedSigmaTree>, ProverError> {
    children
        .into_iter()
        .map(|c| {
            if let ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(ust)) = c {
                Ok(ust)
            } else {
                Err(ProverError::Unexpected(format!(
                    "rewrite: expected UncheckedSigmaTree got: {:?}",
                    c
                )))
            }
        })
        .collect::<Result<Vec<UncheckedSigmaTree>, _>>()
}

pub(crate) fn rewrite<F: Fn(&ProofTree) -> Result<Option<ProofTree>, ProverError>>(
    tree: ProofTree,
    f: &F,
) -> Result<ProofTree, ProverError> {
    let rewritten_tree = f(&tree)?.unwrap_or(tree);
    Ok(match &rewritten_tree {
        ProofTree::UnprovenTree(unp_tree) => match unp_tree {
            UnprovenTree::UnprovenLeaf(_) => rewritten_tree,
            UnprovenTree::UnprovenConjecture(conj) => match conj {
                UnprovenConjecture::CandUnproven(cand) => UnprovenTree::UnprovenConjecture(
                    UnprovenConjecture::CandUnproven(CandUnproven {
                        children: rewrite_children(cand.children.clone(), f)?,
                        ..cand.clone()
                    }),
                )
                .into(),
                UnprovenConjecture::CorUnproven(cor) => {
                    UnprovenTree::UnprovenConjecture(UnprovenConjecture::CorUnproven(CorUnproven {
                        children: rewrite_children(cor.children.clone(), f)?,
                        ..cor.clone()
                    }))
                    .into()
                }
            },
        },
        ProofTree::UncheckedTree(unch_tree) => {
            match unch_tree {
                UncheckedTree::NoProof => rewritten_tree,
                UncheckedTree::UncheckedSigmaTree(ust) => {
                    match ust {
                        UncheckedSigmaTree::UncheckedLeaf(_) => rewritten_tree,
                        UncheckedSigmaTree::UncheckedConjecture(conj) => {
                            match conj {
                                UncheckedConjecture::CandUnchecked {
                                    challenge,
                                    children,
                                } => {
                                    // TODO: reduce indentation?
                                    let rewritten_children = rewrite_children(children.clone(), f)?;
                                    let casted_children = cast_to_ust(rewritten_children)?;
                                    UncheckedConjecture::CandUnchecked {
                                        children: casted_children,
                                        challenge: challenge.clone(),
                                    }
                                    .into()
                                }
                                UncheckedConjecture::CorUnchecked {
                                    challenge,
                                    children,
                                } => {
                                    let rewritten_children = rewrite_children(children.clone(), f)?;
                                    let casted_children = cast_to_ust(rewritten_children)?;
                                    UncheckedConjecture::CorUnchecked {
                                        children: casted_children,
                                        challenge: challenge.clone(),
                                    }
                                    .into()
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
