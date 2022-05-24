//! Sigma protocols

#![deny(clippy::unwrap_used)]

pub mod private_input;
pub mod prover;
pub mod verifier;

pub(crate) mod challenge;
mod crypto_utils;
pub mod dht_protocol;
pub mod dlog_protocol;
mod fiat_shamir;
mod gf2_192;
pub mod proof_tree;
pub mod sig_serializer;
pub mod unchecked_tree;
pub mod unproven_tree;
pub mod wscalar;

use std::array::TryFromSliceError;
use std::convert::TryFrom;
use std::convert::TryInto;

use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

use dlog_protocol::FirstDlogProverMessage;
use unchecked_tree::UncheckedTree;
use unproven_tree::{UnprovenLeaf, UnprovenSchnorr};

use self::challenge::Challenge;
use self::dht_protocol::FirstDhTupleProverMessage;
use self::unchecked_tree::UncheckedSchnorr;

use derive_more::From;
use derive_more::TryInto;

/** The message sent by a prover to its associated verifier as part of a sigma protocol interaction. */
pub(crate) trait ProverMessage {
    /// serialized message
    fn bytes(&self) -> Vec<u8>;
}

/** First message from the prover (message `a` of `SigmaProtocol`)*/
#[derive(PartialEq, Debug, Clone, From, TryInto)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(tag = "type"))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum FirstProverMessage {
    /// Discrete log
    #[cfg_attr(feature = "json", serde(rename = "dlog"))]
    FirstDlogProverMessage(FirstDlogProverMessage),
    /// DH tupl
    #[cfg_attr(feature = "json", serde(rename = "dht"))]
    FirstDhtProverMessage(FirstDhTupleProverMessage),
}

impl ProverMessage for FirstProverMessage {
    fn bytes(&self) -> Vec<u8> {
        match self {
            FirstProverMessage::FirstDlogProverMessage(fdpm) => fdpm.bytes(),
            FirstProverMessage::FirstDhtProverMessage(fdhtpm) => fdhtpm.bytes(),
        }
    }
}

/** Size of the binary representation of any group element (2 ^ groupSizeBits == <number of elements in a group>) */
pub(crate) const GROUP_SIZE_BITS: usize = 256;
/** Number of bytes to represent any group element as byte array */
pub(crate) const GROUP_SIZE: usize = GROUP_SIZE_BITS / 8;

/// Byte array of Group size (32 bytes)
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct GroupSizedBytes(pub(crate) Box<[u8; GROUP_SIZE]>);

impl From<&[u8; GROUP_SIZE]> for GroupSizedBytes {
    fn from(b: &[u8; GROUP_SIZE]) -> Self {
        GroupSizedBytes(Box::new(*b))
    }
}

impl TryFrom<Vec<u8>> for GroupSizedBytes {
    type Error = TryFromSliceError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes: [u8; GROUP_SIZE] = value.as_slice().try_into()?;
        Ok(GroupSizedBytes(bytes.into()))
    }
}

impl TryFrom<&[u8]> for GroupSizedBytes {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; GROUP_SIZE] = value.try_into()?;
        Ok(GroupSizedBytes(bytes.into()))
    }
}

/** A size of challenge in Sigma protocols, in bits.
 * If this anything but 192, threshold won't work, because we have polynomials over GF(2^192) and no others.
 * We get the challenge by reducing hash function output to proper value.
 */
pub const SOUNDNESS_BITS: usize = 192;
/// A size of challenge in Sigma protocols, in bytes
pub const SOUNDNESS_BYTES: usize = SOUNDNESS_BITS / 8;

#[cfg(test)]
#[cfg(feature = "arbitrary")]
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
