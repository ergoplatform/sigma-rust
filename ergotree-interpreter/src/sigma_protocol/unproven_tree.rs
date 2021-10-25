//! Unproven tree types

use super::dht_protocol::FirstDhTupleProverMessage;
use super::proof_tree::ConjectureType;
use super::proof_tree::ProofTree;
use super::proof_tree::ProofTreeConjecture;
use super::proof_tree::ProofTreeKind;
use super::{dlog_protocol::FirstDlogProverMessage, Challenge, FirstProverMessage};
use crate::sigma_protocol::proof_tree::ProofTreeLeaf;
use ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
use ergotree_ir::sigma_protocol::sigma_boolean::cor::Cor;
use ergotree_ir::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
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
            UnprovenTree::UnprovenLeaf(ul) => ul.simulated(),
            UnprovenTree::UnprovenConjecture(uc) => uc.simulated(),
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

    pub(crate) fn with_simulated(self, simulated: bool) -> Self {
        match self {
            UnprovenTree::UnprovenLeaf(ul) => ul.with_simulated(simulated).into(),
            UnprovenTree::UnprovenConjecture(uc) => uc.with_simulated(simulated).into(),
        }
    }

    pub(crate) fn as_tree_kind(&self) -> ProofTreeKind {
        match self {
            UnprovenTree::UnprovenLeaf(ul) => ProofTreeKind::Leaf(ul),
            UnprovenTree::UnprovenConjecture(uc) => ProofTreeKind::Conjecture(uc),
        }
    }

    pub(crate) fn challenge(&self) -> Option<Challenge> {
        match self {
            UnprovenTree::UnprovenLeaf(ul) => ul.challenge(),
            UnprovenTree::UnprovenConjecture(uc) => uc.challenge(),
        }
    }

    pub(crate) fn position(&self) -> &NodePosition {
        match self {
            UnprovenTree::UnprovenLeaf(ul) => ul.position(),
            UnprovenTree::UnprovenConjecture(uc) => uc.position(),
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

impl From<CorUnproven> for UnprovenTree {
    fn from(v: CorUnproven) -> Self {
        UnprovenTree::UnprovenConjecture(v.into())
    }
}

impl From<UnprovenDhTuple> for UnprovenTree {
    fn from(v: UnprovenDhTuple) -> Self {
        UnprovenTree::UnprovenLeaf(v.into())
    }
}

/// Unproven leaf types
#[derive(PartialEq, Debug, Clone, From)]
pub(crate) enum UnprovenLeaf {
    /// Unproven Schnorr
    UnprovenSchnorr(UnprovenSchnorr),
    UnprovenDhTuple(UnprovenDhTuple),
}

impl UnprovenLeaf {
    pub(crate) fn with_position(self, updated: NodePosition) -> Self {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.with_position(updated).into(),
            UnprovenLeaf::UnprovenDhTuple(ut) => ut.with_position(updated).into(),
        }
    }

    pub(crate) fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.with_challenge(challenge).into(),
            UnprovenLeaf::UnprovenDhTuple(ut) => ut.with_challenge(challenge).into(),
        }
    }

    pub(crate) fn with_simulated(self, simulated: bool) -> Self {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.with_simulated(simulated).into(),
            UnprovenLeaf::UnprovenDhTuple(ut) => ut.with_simulated(simulated).into(),
        }
    }

    pub(crate) fn is_real(&self) -> bool {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.is_real(),
            UnprovenLeaf::UnprovenDhTuple(ut) => ut.is_real(),
        }
    }

    pub(crate) fn challenge(&self) -> Option<Challenge> {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.challenge_opt.clone(),
            UnprovenLeaf::UnprovenDhTuple(ut) => ut.challenge_opt.clone(),
        }
    }

    pub(crate) fn position(&self) -> &NodePosition {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => &us.position,
            UnprovenLeaf::UnprovenDhTuple(ut) => &ut.position,
        }
    }

    pub(crate) fn simulated(&self) -> bool {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.simulated,
            UnprovenLeaf::UnprovenDhTuple(udht) => udht.simulated,
        }
    }
}

impl ProofTreeLeaf for UnprovenLeaf {
    fn proposition(&self) -> SigmaBoolean {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
            UnprovenLeaf::UnprovenDhTuple(udht) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDhTuple(udht.proposition.clone()),
            ),
        }
    }

    fn commitment_opt(&self) -> Option<FirstProverMessage> {
        match self {
            UnprovenLeaf::UnprovenSchnorr(us) => us.commitment_opt.clone().map(Into::into),
            UnprovenLeaf::UnprovenDhTuple(udht) => udht.commitment_opt.clone().map(Into::into),
        }
    }
}

#[derive(PartialEq, Debug, Clone, From)]
pub(crate) enum UnprovenConjecture {
    CandUnproven(CandUnproven),
    CorUnproven(CorUnproven),
    CthresholdUnproven(CthresholdUnproven),
}

impl UnprovenConjecture {
    pub(crate) fn children(&self) -> SigmaConjectureItems<ProofTree> {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.children.clone(),
            UnprovenConjecture::CorUnproven(cor) => cor.children.clone(),
        }
    }

    pub(crate) fn position(&self) -> &NodePosition {
        match self {
            UnprovenConjecture::CandUnproven(cand) => &cand.position,
            UnprovenConjecture::CorUnproven(cor) => &cor.position,
        }
    }

    fn challenge(&self) -> Option<Challenge> {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.challenge_opt.clone(),
            UnprovenConjecture::CorUnproven(cor) => cor.challenge_opt.clone(),
        }
    }

    fn with_position(self, updated: NodePosition) -> Self {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.with_position(updated).into(),
            UnprovenConjecture::CorUnproven(cor) => cor.with_position(updated).into(),
        }
    }

    fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.with_challenge(challenge).into(),
            UnprovenConjecture::CorUnproven(cor) => cor.with_challenge(challenge).into(),
        }
    }

    fn with_simulated(self, simulated: bool) -> Self {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.with_simulated(simulated).into(),
            UnprovenConjecture::CorUnproven(cor) => cor.with_simulated(simulated).into(),
        }
    }

    fn simulated(&self) -> bool {
        match self {
            UnprovenConjecture::CandUnproven(au) => au.simulated,
            UnprovenConjecture::CorUnproven(ou) => ou.simulated,
        }
    }
}

impl ProofTreeConjecture for UnprovenConjecture {
    fn conjecture_type(&self) -> ConjectureType {
        match self {
            UnprovenConjecture::CandUnproven(_) => ConjectureType::And,
            UnprovenConjecture::CorUnproven(_) => ConjectureType::Or,
        }
    }

    fn children(&self) -> SigmaConjectureItems<ProofTree> {
        match self {
            UnprovenConjecture::CandUnproven(cand) => cand.children.clone(),
            UnprovenConjecture::CorUnproven(cor) => cor.children.clone(),
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

    fn with_simulated(self, simulated: bool) -> Self {
        UnprovenSchnorr { simulated, ..self }
    }

    pub(crate) fn is_real(&self) -> bool {
        !self.simulated
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnprovenDhTuple {
    pub proposition: ProveDhTuple,
    pub commitment_opt: Option<FirstDhTupleProverMessage>,
    pub randomness_opt: Option<Scalar>,
    pub challenge_opt: Option<Challenge>,
    pub simulated: bool,
    pub position: NodePosition,
}

impl UnprovenDhTuple {
    fn with_position(self, updated: NodePosition) -> Self {
        UnprovenDhTuple {
            position: updated,
            ..self
        }
    }

    fn with_challenge(self, challenge: Challenge) -> Self {
        UnprovenDhTuple {
            challenge_opt: Some(challenge),
            ..self
        }
    }

    fn with_simulated(self, simulated: bool) -> Self {
        UnprovenDhTuple { simulated, ..self }
    }

    pub fn with_commitment(self, commitment: FirstDhTupleProverMessage) -> Self {
        Self {
            commitment_opt: Some(commitment),
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
    pub(crate) proposition: Cand,
    pub(crate) challenge_opt: Option<Challenge>,
    pub(crate) simulated: bool,
    pub(crate) children: SigmaConjectureItems<ProofTree>,
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

    fn with_simulated(self, simulated: bool) -> Self {
        Self { simulated, ..self }
    }

    pub(crate) fn with_children(self, children: SigmaConjectureItems<ProofTree>) -> Self {
        CandUnproven { children, ..self }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct CorUnproven {
    pub(crate) proposition: Cor,
    pub(crate) challenge_opt: Option<Challenge>,
    pub(crate) simulated: bool,
    pub(crate) children: SigmaConjectureItems<ProofTree>,
    pub(crate) position: NodePosition,
}

impl CorUnproven {
    pub(crate) fn is_real(&self) -> bool {
        !self.simulated
    }

    fn with_position(self, updated: NodePosition) -> Self {
        Self {
            position: updated,
            ..self
        }
    }

    fn with_challenge(self, challenge: Challenge) -> Self {
        Self {
            challenge_opt: Some(challenge),
            ..self
        }
    }

    fn with_simulated(self, simulated: bool) -> Self {
        Self { simulated, ..self }
    }

    pub(crate) fn with_children(self, children: SigmaConjectureItems<ProofTree>) -> Self {
        Self { children, ..self }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct CthresholdUnproven {
    pub(crate) proposition: Cthreshold,
    pub(crate) k: u8,
    pub(crate) children: SigmaConjectureItems<ProofTree>,
    pub(crate) polinomial_opt: Option<Gf2_192Poly>,
    pub(crate) challenge_opt: Option<Challenge>,
    pub(crate) simulated: bool,
    pub(crate) position: NodePosition,
}

// TODO: extract
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Gf2_192Poly {}
