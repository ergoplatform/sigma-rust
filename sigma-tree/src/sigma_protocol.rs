//! Sigma protocols

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(missing_docs)]

mod challenge;
mod private_input;

pub use challenge::*;
pub use private_input::*;

pub mod dlog_group;
pub mod dlog_protocol;
pub mod fiat_shamir;
pub mod prover;
pub mod sig_serializer;
pub mod sigma_boolean;
pub mod unchecked_tree;
pub mod unproven_tree;
pub mod verifier;

use k256::Scalar;

use dlog_protocol::FirstDlogProverMessage;
use sigma_boolean::{ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree};
use std::convert::TryInto;
use unchecked_tree::{UncheckedSigmaTree, UncheckedTree};
use unproven_tree::{UnprovenLeaf, UnprovenSchnorr, UnprovenTree};

pub trait ProverMessage {
    fn bytes(&self) -> Vec<u8>;
}

pub enum FirstProverMessage {
    FirstDlogProverMessage(FirstDlogProverMessage),
    FirstDHTProverMessage,
}

impl ProverMessage for FirstProverMessage {
    fn bytes(&self) -> Vec<u8> {
        match self {
            FirstProverMessage::FirstDlogProverMessage(fdpm) => fdpm.bytes(),
            FirstProverMessage::FirstDHTProverMessage => todo!(),
        }
    }
}

/// Proof tree
pub enum ProofTree {
    /// Unchecked tree
    UncheckedTree(UncheckedTree),
    /// Unproven tree
    UnprovenTree(UnprovenTree),
}

impl ProofTree {
    pub fn with_challenge(&self, challenge: Challenge) -> ProofTree {
        match self {
            ProofTree::UncheckedTree(_) => todo!(),
            ProofTree::UnprovenTree(ut) => match ut {
                UnprovenTree::UnprovenLeaf(ul) => match ul {
                    UnprovenLeaf::UnprovenSchnorr(us) => ProofTree::UnprovenTree(
                        UnprovenSchnorr {
                            challenge_opt: Some(challenge),
                            ..us.clone()
                        }
                        .into(),
                    ),
                },
            },
        }
    }
}

impl<T: Into<UncheckedTree>> From<T> for ProofTree {
    fn from(t: T) -> Self {
        ProofTree::UncheckedTree(t.into())
    }
}

pub trait ProofTreeLeaf {
    fn proposition(&self) -> SigmaBoolean;

    fn commitment_opt(&self) -> Option<FirstProverMessage>;
}

/** Size of the binary representation of any group element (2 ^ groupSizeBits == <number of elements in a group>) */
pub const GROUP_SIZE_BITS: usize = 256;
/** Number of bytes to represent any group element as byte array */
pub const GROUP_SIZE: usize = GROUP_SIZE_BITS / 8;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GroupSizedBytes(pub Box<[u8; GROUP_SIZE]>);

impl From<GroupSizedBytes> for Scalar {
    fn from(b: GroupSizedBytes) -> Self {
        let sl: &[u8] = b.0.as_ref();
        Scalar::from_bytes_reduced(sl.try_into().expect(""))
    }
}

impl From<&[u8; GROUP_SIZE]> for GroupSizedBytes {
    fn from(b: &[u8; GROUP_SIZE]) -> Self {
        GroupSizedBytes(Box::new(*b))
    }
}

/** A size of challenge in Sigma protocols, in bits.
 * If this anything but 192, threshold won't work, because we have polynomials over GF(2^192) and no others.
 * So DO NOT change the value without implementing polynomials over GF(2^soundnessBits) first
 * and changing code that calls on GF2_192 and GF2_192_Poly classes!!!
 * We get the challenge by reducing hash function output to proper value.
 */
pub const SOUNDNESS_BITS: usize = 192;
pub const SOUNDNESS_BYTES: usize = SOUNDNESS_BITS / 8;

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn ensure_soundness_bits() {
        // see SOUNDNESS_BITS doc comment
        assert!(SOUNDNESS_BITS < GROUP_SIZE_BITS);
        // blake2b hash function requirements
        assert!(SOUNDNESS_BYTES * 8 <= 512);
        assert!(SOUNDNESS_BYTES % 8 == 0);
    }
}
