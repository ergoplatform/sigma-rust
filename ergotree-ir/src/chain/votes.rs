//! Main "remote" type for [Vote]()
use thiserror::Error;

use std::convert::{TryFrom, TryInto};

/// Votes for changing system parameters
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Votes(pub [u8; 3]);

impl From<Votes> for Vec<u8> {
    fn from(v: Votes) -> Self {
        v.0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for Votes {
    type Error = VotesError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let arr: [u8; 3] = bytes.as_slice().try_into()?;
        Ok(Self(arr))
    }
}

/// Votes errors
#[derive(Error, Debug)]
pub enum VotesError {
    /// Invalid byte array size
    #[error("Votes: Invalid byte array size ({0})")]
    InvalidSize(#[from] std::array::TryFromSliceError),
}
