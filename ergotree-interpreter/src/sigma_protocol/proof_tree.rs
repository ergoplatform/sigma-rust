extern crate derive_more;
use derive_more::From;
use derive_more::TryInto;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

use crate::sigma_protocol::unproven_tree::CandUnproven;
use crate::sigma_protocol::unproven_tree::UnprovenConjecture;
use crate::sigma_protocol::UncheckedSchnorr;
use crate::sigma_protocol::UnprovenSchnorr;

use super::challenge::Challenge;
use super::unchecked_tree::UncheckedTree;
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
    fn children(&self) -> &[ProofTree];
}

pub(crate) enum ProofTreeKind<'a> {
    Leaf(&'a dyn ProofTreeLeaf),
    Conjecture(&'a dyn ProofTreeConjecture),
}

pub(crate) fn rewrite<E, F: Fn(&ProofTree) -> Result<Option<ProofTree>, E>>(
    tree: ProofTree,
    f: F,
) -> Result<ProofTree, E> {
    // TODO: recursive call for arbitrary depth?
    let rewritten_tree = f(&tree)?.unwrap_or(tree);
    Ok(match &rewritten_tree {
        ProofTree::UnprovenTree(ut) => match ut {
            UnprovenTree::UnprovenLeaf(_) => rewritten_tree,
            UnprovenTree::UnprovenConjecture(conj) => match conj {
                UnprovenConjecture::CandUnproven(cand) => {
                    let maybe_rewritten_children = cand
                        .children
                        .clone()
                        .into_iter()
                        .map(|c| f(&c))
                        .collect::<Result<Vec<Option<ProofTree>>, _>>()?;
                    let rewritten_children = maybe_rewritten_children
                        .into_iter()
                        .zip(cand.children.clone())
                        .map(|(rc, c)| rc.unwrap_or(c))
                        .collect::<Vec<ProofTree>>();
                    UnprovenTree::UnprovenConjecture(UnprovenConjecture::CandUnproven(
                        CandUnproven {
                            children: rewritten_children,
                            ..cand.clone()
                        },
                    ))
                    .into()
                }
            },
        },
        ProofTree::UncheckedTree(_) => todo!(),
    })
}
