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

/// Unchecked sigma tree
#[derive(PartialEq, Debug, Clone, From)]
pub enum UncheckedTree {
    /// Unchecked leaf
    UncheckedLeaf(UncheckedLeaf),
    /// Unchecked conjecture (OR, AND, ...)
    UncheckedConjecture(UncheckedConjecture),
}

impl UncheckedTree {
    /// Get challenge
    pub(crate) fn challenge(&self) -> Challenge {
        match self {
            UncheckedTree::UncheckedLeaf(ul) => ul.challenge(),
            UncheckedTree::UncheckedConjecture(uc) => uc.challenge(),
        }
    }

    pub(crate) fn as_tree_kind(&self) -> ProofTreeKind {
        match self {
            UncheckedTree::UncheckedLeaf(ul) => ProofTreeKind::Leaf(ul),
            UncheckedTree::UncheckedConjecture(uc) => ProofTreeKind::Conjecture(uc),
        }
    }

    pub(crate) fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UncheckedTree::UncheckedLeaf(ul) => ul.with_challenge(challenge).into(),
            UncheckedTree::UncheckedConjecture(uc) => uc.with_challenge(challenge).into(),
        }
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
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.challenge.clone(),
            UncheckedLeaf::UncheckedDhTuple(udht) => udht.challenge.clone(),
        }
    }

    pub fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.with_challenge(challenge).into(),
            UncheckedLeaf::UncheckedDhTuple(udht) => udht.with_challenge(challenge).into(),
        }
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

impl UncheckedSchnorr {
    pub fn with_challenge(self, challenge: Challenge) -> Self {
        UncheckedSchnorr { challenge, ..self }
    }
}

impl From<UncheckedSchnorr> for UncheckedTree {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedTree::UncheckedLeaf(us.into())
    }
}

impl From<UncheckedDhTuple> for UncheckedTree {
    fn from(dh: UncheckedDhTuple) -> Self {
        UncheckedTree::UncheckedLeaf(dh.into())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct UncheckedDhTuple {
    pub proposition: ProveDhTuple,
    pub commitment_opt: Option<FirstDhTupleProverMessage>,
    pub challenge: Challenge,
    pub second_message: SecondDhTupleProverMessage,
}

impl UncheckedDhTuple {
    pub fn with_challenge(self, challenge: Challenge) -> Self {
        UncheckedDhTuple { challenge, ..self }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum UncheckedConjecture {
    CandUnchecked {
        challenge: Challenge,
        children: SigmaConjectureItems<UncheckedTree>,
    },
    CorUnchecked {
        challenge: Challenge,
        children: SigmaConjectureItems<UncheckedTree>,
    },
}

impl UncheckedConjecture {
    pub fn with_children(self, new_children: SigmaConjectureItems<UncheckedTree>) -> Self {
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

    pub fn children_ust(self) -> SigmaConjectureItems<UncheckedTree> {
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

    pub fn with_challenge(self, challenge: Challenge) -> Self {
        match self {
            UncheckedConjecture::CandUnchecked {
                challenge: _,
                children,
            } => UncheckedConjecture::CandUnchecked {
                challenge,
                children,
            },
            UncheckedConjecture::CorUnchecked {
                challenge: _,
                children,
            } => UncheckedConjecture::CorUnchecked {
                challenge,
                children,
            },
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
