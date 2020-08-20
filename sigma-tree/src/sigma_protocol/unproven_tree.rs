use super::{
    dlog_protocol::FirstDlogProverMessage,
    sigma_boolean::{ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree},
    Challenge, FirstProverMessage, ProofTreeLeaf,
};
use k256::Scalar;

/// Unproven tree
pub enum UnprovenTree {
    UnprovenLeaf(UnprovenLeaf),
    // UnprovenConjecture,
}

impl UnprovenTree {
    pub fn real(&self) -> bool {
        match self {
            UnprovenTree::UnprovenLeaf(UnprovenLeaf::UnprovenSchnorr(us)) => !us.simulated,
            // UnprovenTree::UnprovenConjecture => todo!(),
        }
    }
}

impl<T: Into<UnprovenLeaf>> From<T> for UnprovenTree {
    fn from(t: T) -> Self {
        UnprovenTree::UnprovenLeaf(t.into())
    }
}

pub enum UnprovenLeaf {
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

impl From<UnprovenSchnorr> for UnprovenLeaf {
    fn from(us: UnprovenSchnorr) -> Self {
        UnprovenLeaf::UnprovenSchnorr(us)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnprovenSchnorr {
    pub proposition: ProveDlog,
    pub commitment_opt: Option<FirstDlogProverMessage>,
    pub randomness_opt: Option<Scalar>,
    pub challenge_opt: Option<Challenge>,
    pub simulated: bool,
}
