//! Hints for a prover which helps the prover to prove a statement.

use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use k256::Scalar;

use crate::sigma_protocol::challenge::Challenge;
use crate::sigma_protocol::unchecked_tree::UncheckedTree;
use crate::sigma_protocol::unproven_tree::NodePosition;
use crate::sigma_protocol::FirstProverMessage;

/// A hint for a prover which helps the prover to prove a statement. For example, if the statement is "pk1 && pk2",
/// and the prover knows only a secret for the public key pk1, the prover fails on proving without a hint. But if the
/// prover knows that pk2 is known to another party, the prover may prove the statement (with an empty proof for "pk2").
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Clone)]
pub enum Hint {
    /// A hint which is indicating that a secret associated with its public image "image" is already proven.
    SecretProven(SecretProven),
    /// A family of hints which are about a correspondence between a public image of a secret image and prover's commitment
    /// to randomness ("a" in a sigma protocol).
    CommitmentHint(CommitmentHint),
}

/// A hint which contains a proof-of-knowledge for a secret associated with its public image "image",
/// with also the mark that the proof is real.
#[cfg_attr(
    feature = "json",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "crate::json::hint::RealSecretProofJson")
)]
#[derive(PartialEq, Debug, Clone)]
pub struct RealSecretProof {
    /// Public image of a secret which is proven
    #[cfg_attr(feature = "json", serde(rename = "pubkey"))]
    pub image: SigmaBoolean,
    /// Challenge used for a proof
    #[cfg_attr(feature = "json", serde(rename = "challenge"))]
    pub challenge: Challenge,
    /// Proof in a tree form
    #[cfg_attr(feature = "json", serde(rename = "proof"))]
    pub unchecked_tree: UncheckedTree,
    /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
    #[cfg_attr(feature = "json", serde(rename = "position"))]
    pub position: NodePosition,
}

/// A hint which contains a proof-of-knowledge for a secret associated with its public image "image",
/// with also the mark that the proof is real.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(
    feature = "json",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "crate::json::hint::SimulatedSecretProofJson")
)]
pub struct SimulatedSecretProof {
    /// Public image of a secret which is proven
    #[cfg_attr(feature = "json", serde(rename = "pubkey"))]
    pub image: SigmaBoolean,
    /// Challenge used for a proof
    #[cfg_attr(feature = "json", serde(rename = "challenge"))]
    pub challenge: Challenge,
    /// Proof in a tree form
    #[cfg_attr(feature = "json", serde(rename = "proof"))]
    pub unchecked_tree: UncheckedTree,
    /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
    #[cfg_attr(feature = "json", serde(rename = "position"))]
    pub position: NodePosition,
}

/// A hint which is indicating that a secret associated with its public image "image" is already proven.
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(tag = "hint"))]
#[derive(PartialEq, Debug, Clone)]
pub enum SecretProven {
    /// A hint which contains a proof-of-knowledge for a secret associated with its public image "image",
    /// with also the mark that the proof is real.
    #[cfg_attr(feature = "json", serde(rename = "proofReal"))]
    RealSecretProof(RealSecretProof),
    /// A hint which contains a proof-of-knowledge for a secret associated with its public image "image",
    /// with also the mark that the proof is real.
    #[cfg_attr(feature = "json", serde(rename = "proofSimulated"))]
    SimulatedSecretProof(SimulatedSecretProof),
}

impl SecretProven {
    /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
    pub fn position(&self) -> &NodePosition {
        match self {
            SecretProven::RealSecretProof(proof) => &proof.position,
            SecretProven::SimulatedSecretProof(proof) => &proof.position,
        }
    }

    /// Challenge used for a proof
    pub fn challenge(&self) -> &Challenge {
        match self {
            SecretProven::RealSecretProof(proof) => &proof.challenge,
            SecretProven::SimulatedSecretProof(proof) => &proof.challenge,
        }
    }
}

/// A hint which contains a commitment to randomness associated with a public image of a secret.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct RealCommitment {
    ///  image of a secret
    #[cfg_attr(feature = "json", serde(rename = "pubkey"))]
    pub image: SigmaBoolean,
    /// commitment to randomness used while proving knowledge of the secret
    #[cfg_attr(feature = "json", serde(rename = "commitment"))]
    pub commitment: FirstProverMessage,
    /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
    #[cfg_attr(feature = "json", serde(rename = "position"))]
    pub position: NodePosition,
}

/// A hint which a commitment to randomness associated with a public image of a secret, as well as randomness itself.
/// Please note that this randomness should be kept in secret by the prover.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct OwnCommitment {
    ///  image of a secret
    #[cfg_attr(feature = "json", serde(rename = "pubkey"))]
    pub image: SigmaBoolean,
    /// randomness
    #[cfg_attr(
        feature = "json",
        serde(rename = "secret", with = "crate::json::scalar")
    )]
    pub secret_randomness: Scalar,
    /// commitment to randomness used while proving knowledge of the secret
    #[cfg_attr(feature = "json", serde(rename = "commitment"))]
    pub commitment: FirstProverMessage,
    /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
    #[cfg_attr(feature = "json", serde(rename = "position"))]
    pub position: NodePosition,
}

///A hint which contains a commitment to randomness associated with a public image of a secret.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct SimulatedCommitment {
    /// image of a secret
    #[cfg_attr(feature = "json", serde(rename = "pubkey"))]
    pub image: SigmaBoolean,
    /// commitment to randomness used while proving knowledge of the secret
    #[cfg_attr(feature = "json", serde(rename = "commitment"))]
    pub commitment: FirstProverMessage,
    /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
    #[cfg_attr(feature = "json", serde(rename = "position"))]
    pub position: NodePosition,
}

/// A family of hints which are about a correspondence between a public image of a secret image and prover's commitment
/// to randomness ("a" in a sigma protocol).
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(tag = "hint"))]
#[derive(PartialEq, Debug, Clone)]
pub enum CommitmentHint {
    /// A hint which a commitment to randomness associated with a public image of a secret, as well as randomness itself.
    /// Please note that this randomness should be kept in secret by the prover.
    #[cfg_attr(feature = "json", serde(rename = "cmtWithSecret"))]
    OwnCommitment(OwnCommitment),
    /// A hint which contains a commitment to randomness associated with a public image of a secret.
    #[cfg_attr(feature = "json", serde(rename = "cmtReal"))]
    RealCommitment(RealCommitment),
    ///A hint which contains a commitment to randomness associated with a public image of a secret.
    #[cfg_attr(feature = "json", serde(rename = "cmtSimulated"))]
    SimulatedCommitment(SimulatedCommitment),
}

impl CommitmentHint {
    /// A hint is related to a subtree (or a leaf) of a tree. Returns position in the tree.
    pub fn position(&self) -> &NodePosition {
        match self {
            CommitmentHint::OwnCommitment(comm) => &comm.position,
            CommitmentHint::RealCommitment(comm) => &comm.position,
            CommitmentHint::SimulatedCommitment(comm) => &comm.position,
        }
    }

    /// commitment to randomness used while proving knowledge of the secret
    pub fn commitment(&self) -> &FirstProverMessage {
        match self {
            CommitmentHint::OwnCommitment(comm) => &comm.commitment,
            CommitmentHint::RealCommitment(comm) => &comm.commitment,
            CommitmentHint::SimulatedCommitment(comm) => &comm.commitment,
        }
    }
}

/// Collection of hints to be used by a prover
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug, Clone)]
pub struct HintsBag {
    /// Hints stored in a bag
    pub hints: Vec<Hint>,
}

impl HintsBag {
    /// Bag without hints
    pub fn empty() -> Self {
        HintsBag { hints: vec![] }
    }

    /// Adding new hint to hints
    pub fn add_hint(&mut self, hint: Hint) {
        self.hints.push(hint);
    }

    /// Commitments from all CommitmentHints in the bag
    pub fn commitments(&self) -> Vec<CommitmentHint> {
        self.hints
            .clone()
            .into_iter()
            .filter_map(|hint| {
                if let Hint::CommitmentHint(v) = hint {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
    }

    /// RealSecretProof hints only
    pub fn real_proofs(&self) -> Vec<RealSecretProof> {
        self.hints
            .clone()
            .into_iter()
            .filter_map(|hint| {
                if let Hint::SecretProven(SecretProven::RealSecretProof(v)) = hint {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
    }

    /// RealCommitment hints only
    pub fn real_commitments(&self) -> Vec<RealCommitment> {
        self.hints
            .clone()
            .into_iter()
            .filter_map(|hint| {
                if let Hint::CommitmentHint(CommitmentHint::RealCommitment(v)) = hint {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
    }

    /// OwnCommitment hints only
    pub fn own_commitments(&self) -> Vec<OwnCommitment> {
        self.hints
            .clone()
            .into_iter()
            .filter_map(|hint| {
                if let Hint::CommitmentHint(CommitmentHint::OwnCommitment(v)) = hint {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Images of real secrets + real commitments in the bag
    pub fn real_images(&self) -> Vec<SigmaBoolean> {
        let mut from_proofs: Vec<SigmaBoolean> =
            self.real_proofs().iter().map(|p| p.image.clone()).collect();
        let mut from_comms: Vec<SigmaBoolean> = self
            .real_commitments()
            .iter()
            .map(|c| c.image.clone())
            .collect();
        from_proofs.append(&mut from_comms);
        from_proofs
    }

    /// SimulatedSecretProof proofs only
    pub fn simulated_proofs(&self) -> Vec<SimulatedSecretProof> {
        self.hints
            .clone()
            .into_iter()
            .filter_map(|hint| {
                if let Hint::SecretProven(SecretProven::SimulatedSecretProof(v)) = hint {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
    }

    /// All proofs from SecretProven variants
    pub fn proofs(&self) -> Vec<SecretProven> {
        self.hints
            .clone()
            .into_iter()
            .filter_map(|hint| {
                if let Hint::SecretProven(sp) = hint {
                    Some(sp)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Arbitrary
#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use proptest::arbitrary::Arbitrary;
    use proptest::strategy::BoxedStrategy;

    use super::HintsBag;

    impl Arbitrary for HintsBag {
        type Parameters = ();

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            todo!()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}
