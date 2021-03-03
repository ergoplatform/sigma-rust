//! Unproven tree types

use super::{dlog_protocol::FirstDlogProverMessage, Challenge, FirstProverMessage, ProofTreeLeaf};
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use k256::Scalar;

/// Unproven trees
pub enum UnprovenTree {
    /// Unproven leaf
    UnprovenLeaf(UnprovenLeaf),
    // UnprovenConjecture,
}

impl UnprovenTree {
    /// Is real or simulated
    pub fn is_real(&self) -> bool {
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

/// Unproven leaf types
pub enum UnprovenLeaf {
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

impl From<UnprovenSchnorr> for UnprovenLeaf {
    fn from(us: UnprovenSchnorr) -> Self {
        UnprovenLeaf::UnprovenSchnorr(us)
    }
}

#[allow(missing_docs)]
#[derive(PartialEq, Debug, Clone)]
pub struct UnprovenSchnorr {
    pub proposition: ProveDlog,
    pub commitment_opt: Option<FirstDlogProverMessage>,
    pub randomness_opt: Option<Scalar>,
    pub challenge_opt: Option<Challenge>,
    pub simulated: bool,
}
