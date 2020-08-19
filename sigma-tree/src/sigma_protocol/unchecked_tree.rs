use super::{
    dlog_protocol::{FirstDlogProverMessage, SecondDlogProverMessage},
    sigma_boolean::{ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree},
    Challenge, FirstProverMessage, ProofTree, ProofTreeLeaf,
};

/// Unchecked tree
pub enum UncheckedTree {
    /// No proof needed
    NoProof,
    /// Unchecked sigma tree
    UncheckedSigmaTree(UncheckedSigmaTree),
}

/// Unchecked sigma tree
#[derive(PartialEq, Debug, Clone)]
pub enum UncheckedSigmaTree {
    UncheckedLeaf(UncheckedLeaf),
    UncheckedConjecture,
}

impl UncheckedSigmaTree {
    pub fn challenge(&self) -> Challenge {
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

#[derive(PartialEq, Debug, Clone)]
pub enum UncheckedLeaf {
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
