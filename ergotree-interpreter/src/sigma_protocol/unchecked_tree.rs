//! Unchecked proof tree types

use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaConjectureItems;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;

use super::dht_protocol::FirstDhTupleProverMessage;
use super::dht_protocol::SecondDhTupleProverMessage;
use super::proof_tree::ConjectureType;
use super::proof_tree::ProofTree;
use super::proof_tree::ProofTreeConjecture;
use super::proof_tree::ProofTreeKind;
use super::proof_tree::ProofTreeLeaf;
use super::{
    dlog_protocol::{FirstDlogProverMessage, SecondDlogProverMessage},
    Challenge, FirstProverMessage,
};

use derive_more::From;

/// Unchecked tree
#[derive(PartialEq, Debug, Clone, From)]
pub enum UncheckedTree {
    /// No proof needed
    NoProof,
    /// Unchecked sigma tree
    UncheckedSigmaTree(UncheckedSigmaTree),
}

impl UncheckedTree {
    pub(crate) fn as_tree_kind(&self) -> ProofTreeKind {
        match self {
            UncheckedTree::NoProof => panic!("NoProof has not ProofTreeKind representation"),
            UncheckedTree::UncheckedSigmaTree(ust) => ust.as_tree_kind(),
        }
    }

    pub(crate) fn challenge(&self) -> Option<Challenge> {
        match self {
            UncheckedTree::NoProof => None,
            UncheckedTree::UncheckedSigmaTree(ust) => Some(ust.challenge()),
        }
    }
}

/// Unchecked sigma tree
#[derive(PartialEq, Debug, Clone, From)]
pub enum UncheckedSigmaTree {
    /// Unchecked leaf
    UncheckedLeaf(UncheckedLeaf),
    /// Unchecked conjecture (OR, AND, ...)
    UncheckedConjecture(UncheckedConjecture),
}

impl UncheckedSigmaTree {
    /// Get challenge
    pub(crate) fn challenge(&self) -> Challenge {
        match self {
            UncheckedSigmaTree::UncheckedLeaf(ul) => ul.challenge(),
            UncheckedSigmaTree::UncheckedConjecture(uc) => uc.challenge(),
        }
    }

    pub(crate) fn as_tree_kind(&self) -> ProofTreeKind {
        match self {
            UncheckedSigmaTree::UncheckedLeaf(ul) => ProofTreeKind::Leaf(ul),
            UncheckedSigmaTree::UncheckedConjecture(uc) => ProofTreeKind::Conjecture(uc),
        }
    }
}

impl From<UncheckedSchnorr> for UncheckedSigmaTree {
    fn from(v: UncheckedSchnorr) -> Self {
        UncheckedSigmaTree::UncheckedLeaf(v.into())
    }
}

/// Unchecked leaf
#[derive(PartialEq, Debug, Clone, From)]
pub enum UncheckedLeaf {
    /// Unchecked Schnorr
    UncheckedSchnorr(UncheckedSchnorr),
    UncheckedDhTuple(UncheckedDhTuple),
}

impl UncheckedLeaf {
    pub fn challenge(&self) -> Challenge {
        todo!()
    }
}

impl ProofTreeLeaf for UncheckedLeaf {
    fn proposition(&self) -> SigmaBoolean {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
            UncheckedLeaf::UncheckedDhTuple(dhu) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDhTuple(dhu.proposition.clone()),
            ),
        }
    }
    fn commitment_opt(&self) -> Option<FirstProverMessage> {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.commitment_opt.clone().map(Into::into),
            UncheckedLeaf::UncheckedDhTuple(udh) => udh.commitment_opt.clone().map(Into::into),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct UncheckedSchnorr {
    pub proposition: ProveDlog,
    pub commitment_opt: Option<FirstDlogProverMessage>,
    pub challenge: Challenge,
    pub second_message: SecondDlogProverMessage,
}

impl From<UncheckedSchnorr> for UncheckedTree {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedTree::UncheckedSigmaTree(us.into())
    }
}

impl From<UncheckedDhTuple> for UncheckedSigmaTree {
    fn from(dh: UncheckedDhTuple) -> Self {
        UncheckedSigmaTree::UncheckedLeaf(dh.into())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct UncheckedDhTuple {
    pub proposition: ProveDhTuple,
    pub commitment_opt: Option<FirstDhTupleProverMessage>,
    pub challenge: Challenge,
    pub second_message: SecondDhTupleProverMessage,
}

impl From<UncheckedDhTuple> for UncheckedTree {
    fn from(dh: UncheckedDhTuple) -> Self {
        UncheckedTree::UncheckedSigmaTree(dh.into())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum UncheckedConjecture {
    CandUnchecked {
        challenge: Challenge,
        children: SigmaConjectureItems<UncheckedSigmaTree>,
    },
    CorUnchecked {
        challenge: Challenge,
        children: SigmaConjectureItems<UncheckedSigmaTree>,
    },
}

impl UncheckedConjecture {
    pub fn with_children(self, new_children: SigmaConjectureItems<UncheckedSigmaTree>) -> Self {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge,
                children: _,
            } => UncheckedConjecture::CandUnchecked {
                challenge,
                children: new_children,
            },
            UncheckedConjecture::CorUnchecked {
                challenge,
                children: _,
            } => UncheckedConjecture::CorUnchecked {
                challenge,
                children: new_children,
            },
        }
    }

    pub fn children_ust(self) -> SigmaConjectureItems<UncheckedSigmaTree> {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge: _,
                children,
            } => children,
            UncheckedConjecture::CorUnchecked {
                challenge: _,
                children,
            } => children,
        }
    }

    pub fn challenge(&self) -> Challenge {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge,
                children: _,
            } => challenge.clone(),
            UncheckedConjecture::CorUnchecked {
                challenge,
                children: _,
            } => challenge.clone(),
        }
    }
}

impl ProofTreeConjecture for UncheckedConjecture {
    fn conjecture_type(&self) -> ConjectureType {
        match self {
            UncheckedConjecture::CandUnchecked { .. } => ConjectureType::And,
            UncheckedConjecture::CorUnchecked { .. } => ConjectureType::Or,
        }
    }

    fn children(&self) -> SigmaConjectureItems<ProofTree> {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge: _,
                children,
            } => children.mapped_ref(|ust| ust.clone().into()),
            UncheckedConjecture::CorUnchecked {
                challenge: _,
                children,
            } => children.mapped_ref(|ust| ust.clone().into()),
        }
    }
}
