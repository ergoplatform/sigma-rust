//! Unchecked proof tree types

use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;

use super::{
    dlog_protocol::{FirstDlogProverMessage, SecondDlogProverMessage},
    Challenge, FirstProverMessage, ProofTree, ProofTreeLeaf,
};

/// Unchecked tree
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum UncheckedTree {
    /// No proof needed
    NoProof,
    /// Unchecked sigma tree
    UncheckedSigmaTree(UncheckedSigmaTree),
}

/// Unchecked sigma tree
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum UncheckedSigmaTree {
    /// Unchecked leaf
    UncheckedLeaf(UncheckedLeaf),
    /// Unchecked conjecture (OR, AND, ...)
    UncheckedConjecture,
}

impl UncheckedSigmaTree {
    /// Get challenge
    pub(crate) fn challenge(&self) -> Challenge {
        match self {
            UncheckedSigmaTree::UncheckedLeaf(UncheckedLeaf::UncheckedSchnorr(us)) => {
                us.challenge.clone()
            }
            UncheckedSigmaTree::UncheckedConjecture => todo!(),
        }
    }
}

impl<T: Into<UncheckedLeaf>> From<T> for UncheckedSigmaTree {
    fn from(t: T) -> Self {
        UncheckedSigmaTree::UncheckedLeaf(t.into())
    }
}

impl From<UncheckedSigmaTree> for ProofTree {
    fn from(ust: UncheckedSigmaTree) -> Self {
        ProofTree::UncheckedTree(UncheckedTree::UncheckedSigmaTree(ust))
    }
}

/// Unchecked leaf
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum UncheckedLeaf {
    /// Unchecked Schnorr
    UncheckedSchnorr(UncheckedSchnorr),
}

impl ProofTreeLeaf for UncheckedLeaf {
    fn proposition(&self) -> SigmaBoolean {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => SigmaBoolean::ProofOfKnowledge(
                SigmaProofOfKnowledgeTree::ProveDlog(us.proposition.clone()),
            ),
        }
    }
    fn commitment_opt(&self) -> Option<FirstProverMessage> {
        match self {
            UncheckedLeaf::UncheckedSchnorr(us) => us.commitment_opt.clone().map(Into::into),
        }
    }
}

impl From<UncheckedSchnorr> for UncheckedLeaf {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedLeaf::UncheckedSchnorr(us)
    }
}

#[allow(missing_docs)]
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct UncheckedSchnorr {
    pub(crate) proposition: ProveDlog,
    pub(crate) commitment_opt: Option<FirstDlogProverMessage>,
    pub(crate) challenge: Challenge,
    pub(crate) second_message: SecondDlogProverMessage,
}

impl From<UncheckedSchnorr> for UncheckedTree {
    fn from(us: UncheckedSchnorr) -> Self {
        UncheckedTree::UncheckedSigmaTree(us.into())
    }
}
