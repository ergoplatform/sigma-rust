//! Hints for a prover which helps the prover to prove a statement.

use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use num_bigint::BigInt;

use crate::sigma_protocol::challenge::Challenge;
use crate::sigma_protocol::unchecked_tree::UncheckedTree;
use crate::sigma_protocol::unproven_tree::NodePosition;
use crate::sigma_protocol::FirstProverMessage;

/// A hint for a prover which helps the prover to prove a statement. For example, if the statement is "pk1 && pk2",
/// and the prover knows only a secret for the public key pk1, the prover fails on proving without a hint. But if the
/// prover knows that pk2 is known to another party, the prover may prove the statement (with an empty proof for "pk2").
pub enum Hint {
    /// A hint which is indicating that a secret associated with its public image "image" is already proven.
    SecretProven(SecretProven),
    /// A family of hints which are about a correspondence between a public image of a secret image and prover's commitment
    /// to randomness ("a" in a sigma protocol).
    CommitmentHint(CommitmentHint),
}

/// A hint which is indicating that a secret associated with its public image "image" is already proven.
pub enum SecretProven {
    /// A hint which contains a proof-of-knowledge for a secret associated with its public image "image",
    /// with also the mark that the proof is real.
    RealSecretProof {
        /// Public image of a secret which is proven
        image: SigmaBoolean,
        /// Challenge used for a proof
        challenge: Challenge,
        /// Proof in a tree form
        unchecked_tree: UncheckedTree,
        /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
        position: NodePosition,
    },
    /// A hint which contains a proof-of-knowledge for a secret associated with its public image "image",
    /// with also the mark that the proof is real.
    SimulatedSecretProof {
        /// Public image of a secret which is proven
        image: SigmaBoolean,
        /// Challenge used for a proof
        challenge: Challenge,
        /// Proof in a tree form
        unchecked_tree: UncheckedTree,
        /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
        position: NodePosition,
    },
}

/// A family of hints which are about a correspondence between a public image of a secret image and prover's commitment
/// to randomness ("a" in a sigma protocol).
pub enum CommitmentHint {
    /// * A hint which a commitment to randomness associated with a public image of a secret, as well as randomness itself.
    ///  * Please note that this randomness should be kept in secret by the prover.
    ///  *
    OwnCommitment {
        ///  image of a secret
        image: SigmaBoolean,
        /// randomness
        secret_randomness: BigInt,
        /// commitment to randomness used while proving knowledge of the secret
        commitment: FirstProverMessage,
        /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
        position: NodePosition,
    },
    ///A hint which contains a commitment to randomness associated with a public image of a secret.
    RealCommitment {
        /// image of a secret
        image: SigmaBoolean,
        /// commitment to randomness used while proving knowledge of the secret
        commitment: FirstProverMessage,
        /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
        position: NodePosition,
    },
    ///A hint which contains a commitment to randomness associated with a public image of a secret.
    SimulatedCommitment {
        /// image of a secret
        image: SigmaBoolean,
        /// commitment to randomness used while proving knowledge of the secret
        commitment: FirstProverMessage,
        /// A hint is related to a subtree (or a leaf) of a tree. This field encodes a position in the tree.
        position: NodePosition,
    },
}

/// Collection of hints to be used by a prover
pub struct HintsBag {
    /// Hints stored in a bag
    hints: Vec<Hint>,
}

impl HintsBag {
    /// Bag without hints
    pub fn empty() -> Self {
        todo!()
    }
}
