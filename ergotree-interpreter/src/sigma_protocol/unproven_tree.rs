//! Unproven tree types

use super::proof_tree::ConjectureType;
use super::proof_tree::ProofTree;
use super::proof_tree::ProofTreeConjecture;
use super::proof_tree::ProofTreeKind;
use super::{dlog_protocol::FirstDlogProverMessage, Challenge, FirstProverMessage};
use crate::sigma_protocol::proof_tree::ProofTreeLeaf;
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

    pub(crate) fn with_position(self, updated: NodePosition) -> Self {
        match self {
            UnprovenTree::UnprovenLeaf(ul) => ul.with_position(updated).into(),
            UnprovenTree::UnprovenConjecture(uc) => uc.with_position(updated).into(),
        }
    }

    pub(crate) fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UnprovenTree::UnprovenLeaf(ul) => ul.with_challenge(challenge).into(),
            UnprovenTree::UnprovenConjecture(uc) => uc.with_challenge(challenge).into(),
        }
    }

    pub(crate) fn as_tree_kind(&self) -> ProofTreeKind {
        match self {
            UnprovenTree::UnprovenLeaf(ul) => ProofTreeKind::Leaf(ul),
            UnprovenTree::UnprovenConjecture(uc) => ProofTreeKind::Conjecture(uc),
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

impl UnprovenLeaf {
    fn with_position(self, updated: NodePosition) -> Self {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.with_position(updated).into(),
        }
    }

    fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.with_challenge(challenge).into(),
        }
    }

    pub(crate) fn is_real(&self) -> bool {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.is_real(),
        }
    }
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

impl UnprovenConjecture {
    pub(crate) fn children(&self) -> Vec<ProofTree> {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.children.clone(),
        }
    }

    pub(crate) fn position(&self) -> NodePosition {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.position.clone(),
        }
    }

    fn with_position(self, updated: NodePosition) -> Self {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.with_position(updated).into(),
        }
    }

    fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.with_challenge(challenge).into(),
        }
    }
}

impl ProofTreeConjecture for UnprovenConjecture {
    fn conjecture_type(&self) -> ConjectureType {
        match self {
            UnprovenConjecture::CandUnproven(_) => ConjectureType::And,
        }
    }

    fn children(&self) -> &[ProofTree] {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.children.as_ref(),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct UnprovenSchnorr {
    pub(crate) proposition: ProveDlog,
    pub(crate) commitment_opt: Option<FirstDlogProverMessage>,
    pub(crate) randomness_opt: Option<Scalar>,
    pub(crate) challenge_opt: Option<Challenge>,
    pub(crate) simulated: bool,
    pub(crate) position: NodePosition,
}

impl UnprovenSchnorr {
    fn with_position(self, updated: NodePosition) -> Self {
        UnprovenSchnorr {
            position: updated,
            ..self
        }
    }

    fn with_challenge(self, challenge: Challenge) -> Self {
        UnprovenSchnorr {
            challenge_opt: Some(challenge),
            ..self
        }
    }

    pub(crate) fn is_real(&self) -> bool {
        !self.simulated
    }
}

/// Data type which encodes position of a node in a tree.
///
/// Position is encoded like following (the example provided is for CTHRESHOLD(2, Seq(pk1, pk2, pk3 && pk4)) :
///
/// r#"
///            0
///          / | \
///         /  |  \
///       0-0 0-1 0-2
///               /|
///              / |
///             /  |
///            /   |
///          0-2-0 0-2-1
/// "#;
///
/// So a hint associated with pk1 has a position "0-0", pk4 - "0-2-1" .
///
/// Please note that "0" prefix is for a crypto tree. There are several kinds of trees during evaluation.
/// Initial mixed tree (ergoTree) would have another prefix.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NodePosition {
    /// positions from root (inclusive) in top-down order
    positions: Vec<usize>,
}

impl NodePosition {
    pub fn crypto_tree_prefix() -> Self {
        NodePosition { positions: vec![0] }
    }

    pub fn child(&self, child_idx: usize) -> NodePosition {
        let mut positions = self.positions.clone();
        positions.push(child_idx);
        NodePosition { positions }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct CandUnproven {
    pub(crate) proposition: Vec<SigmaBoolean>,
    pub(crate) challenge_opt: Option<Challenge>,
    pub(crate) simulated: bool,
    pub(crate) children: Vec<ProofTree>,
    pub(crate) position: NodePosition,
}

impl CandUnproven {
    pub(crate) fn is_real(&self) -> bool {
        !self.simulated
    }

    fn with_position(self, updated: NodePosition) -> Self {
        CandUnproven {
            position: updated,
            ..self
        }
    }

    fn with_challenge(self, challenge: Challenge) -> Self {
        CandUnproven {
            challenge_opt: Some(challenge),
            ..self
        }
    }

    pub(crate) fn with_children(self, children: Vec<ProofTree>) -> Self {
        CandUnproven { children, ..self }
    }
}
