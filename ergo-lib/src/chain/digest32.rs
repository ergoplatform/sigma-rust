use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use ergotree_ir::chain::digest::Digest;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Formatter;
use thiserror::Error;

/// Definition for remote Digest type. Remote Digest wasn't used, because in ergo-lib
/// this type is mostly needed for json serialization and deserialization. Such traits
/// of Digest aren't needed in ergotree-ir.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        into = "Base16EncodedBytes",
        try_from = "Base16DecodedBytes",
        remote = "Digest"
    )
)]
#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) struct DigestRef<const N: usize>(pub(crate) Box<[u8; N]>);

impl<const N: usize> TryFrom<Base16DecodedBytes> for Digest<N> {
    type Error = Digest32Error;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        let arr: [u8; N] = bytes.0.as_slice().try_into()?;
        Ok(Digest(Box::new(arr)))
    }
}

impl<const N: usize> From<Digest<N>> for Base16EncodedBytes {
    fn from(v: Digest<N>) -> Self {
        Base16EncodedBytes::new(v.0.as_ref())
    }
}

impl<const N: usize> std::fmt::Debug for DigestRef<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        base16::encode_lower(&(*self.0)).fmt(f)
    }
}

/// Invalid byte array size
#[derive(Error, Debug)]
pub enum Digest32Error {
    /// error decoding from Base16
    #[error("error decoding from Base16: {0}")]
    Base16DecodingError(#[from] base16::DecodeError),
    /// Invalid byte array size
    #[error("Invalid byte array size ({0})")]
    InvalidSize(#[from] std::array::TryFromSliceError), // todo-sab do we need that?
}
