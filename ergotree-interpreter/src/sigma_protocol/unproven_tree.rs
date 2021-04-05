//! Unproven tree types

use super::{dlog_protocol::FirstDlogProverMessage, Challenge, FirstProverMessage, ProofTreeLeaf};
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use k256::Scalar;

extern crate derive_more;
use derive_more::From;

/// Unproven trees
#[derive(PartialEq, Debug, Clone, From)]
pub(crate) enum UnprovenTree {
    UnprovenLeaf(UnprovenLeaf),
    UnprovenConjecture(UnprovenConjecture),
}

impl UnprovenTree {
    /// Is real or simulated
    pub(crate) fn is_real(&self) -> bool {
        !self.simulated()
    }

    pub(crate) fn simulated(&self) -> bool {
        match self {
            UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(us)) => us.simulated,
            UnprovenTree::UnprovenConjecture(UnprovenConjecture::CandUnproven(cand)) => {
                cand.simulated
            }
        }
    }
}

impl From<UnprovenSchnorr> for UnprovenTree {
    fn from(v: UnprovenSchnorr) -> Self {
        UnprovenTree::UnprovenLeaf(v.into())
    }
}

impl From<CandUnproven> for UnprovenTree {
    fn from(v: CandUnproven) -> Self {
        UnprovenTree::UnprovenConjecture(v.into())
    }
}

/// Unproven leaf types
#[derive(PartialEq, Debug, Clone, From)]
pub(crate) enum UnprovenLeaf {
    /// Unproven Schnorr
    UnprovenSchnorr(UnprovenSchnorr),
}

impl ProofTreeLeaf for UnprovenLeaf {
    fn proposition(&self) -> SigmaBoolean {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
        }
    }

    fn commitment_opt(&self) -> Option<FirstProverMessage> {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.commitment_opt.clone().map(Into::into),
        }
    }
}

#[derive(PartialEq, Debug, Clone, From)]
pub(crate) enum UnprovenConjecture {
    CandUnproven(CandUnproven),
}

#[allow(missing_docs)]
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct UnprovenSchnorr {
    pub(crate) proposition: ProveDlog,
    pub(crate) commitment_opt: Option<FirstDlogProverMessage>,
    pub(crate) randomness_opt: Option<Scalar>,
    pub(crate) challenge_opt: Option<Challenge>,
    pub(crate) simulated: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct NodePosition {
    positions: Vec<u32>,
}

impl NodePosition {
    pub(crate) fn crypto_tree_prefix() -> Self {
        NodePosition { positions: vec![0] }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct CandUnproven {
    pub(crate) proposition: Vec<SigmaBoolean>,
    pub(crate) challenge_opt: Option<Challenge>,
    pub(crate) simulated: bool,
    pub(crate) children: Vec<UnprovenTree>,
    pub(crate) position: NodePosition,
}

pub(crate) fn rewrite<E, F: Fn(&UnprovenTree) -> Result<Option<UnprovenTree>, E>>(
    tree: UnprovenTree,
    f: F,
) -> Result<UnprovenTree, E> {
    let rewritten_tree = f(&tree)?.unwrap_or(tree);
    Ok(match &rewritten_tree {
        UnprovenTree::UnprovenLeaf(_) => rewritten_tree,
        UnprovenTree::UnprovenConjecture(conj) => match conj {
            UnprovenConjecture::CandUnproven(cand) => {
                let maybe_rewritten_children = cand
                    .children
                    .clone()
                    .into_iter()
                    .map(|c| f(&c))
                    .collect::<Result<Vec<Option<UnprovenTree>>, _>>()?;
                let rewritten_children = maybe_rewritten_children
                    .into_iter()
                    .zip(cand.children.clone())
                    .map(|(rc, c)| rc.unwrap_or(c))
                    .collect::<Vec<UnprovenTree>>();
                UnprovenTree::UnprovenConjecture(UnprovenConjecture::CandUnproven(CandUnproven {
                    children: rewritten_children,
                    ..cand.clone()
                }))
            }
        },
    })
}
