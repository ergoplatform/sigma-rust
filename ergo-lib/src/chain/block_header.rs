//! Block header
// todo ergotree-ir::chain::Header types is the same as BlockHeader - remove duplicate

use std::convert::TryFrom;
use std::convert::TryInto;

use ergotree_ir::chain::block_id::BlockId;
use ergotree_ir::chain::digest::Digest32;
use ergotree_ir::chain::votes::Votes;
use ergotree_ir::chain::votes::VotesError;
use ergotree_ir::mir::header::PreHeader;
use ergotree_ir::sigma_protocol::dlog_group;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use super::Base16DecodedBytes;
use super::Base16EncodedBytes;
use super::DigestRef;

/// Block id
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(remote = "BlockId"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockIdRef(#[serde(with = "DigestDef")] Digest32);

/// Votes for changing system parameters
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        into = "Base16EncodedBytes",
        try_from = "crate::chain::json::block_header::VotesEncodingVariants"
    )
)]
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "json", serde(remote = "Votes"))]
pub struct VotesRef(pub [u8; 3]);

impl TryFrom<Base16DecodedBytes> for Votes {
    type Error = VotesError;

    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        bytes.0.try_into()
    }
}

impl From<Votes> for Base16EncodedBytes {
    fn from(v: Votes) -> Self {
        Base16EncodedBytes::new(v.0.as_ref())
    }
}

/// Block header
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockHeader {
    /// Block version, to be increased on every soft and hardfork
    pub version: u8,
    /// Id of a parent block
    #[cfg_attr(feature = "json", serde(rename = "parentId", with = "BlockIdRef"))]
    pub parent_id: BlockId,
    /// Timestamp of a block in ms from UNIX epoch
    pub timestamp: u64,
    /// Current difficulty in a compressed view.
    #[cfg_attr(feature = "json", serde(rename = "nBits"))]
    pub n_bits: u64,
    /// Block height
    pub height: u32,
    /// Votes
    #[cfg_attr(feature = "json", serde(with = "VotesRef"))]
    pub votes: Votes,
}

impl From<BlockHeader> for PreHeader {
    fn from(bh: BlockHeader) -> Self {
        PreHeader {
            version: bh.version,
            parent_id: bh.parent_id.0.into(),
            timestamp: bh.timestamp,
            n_bits: bh.n_bits,
            height: bh.height,
            miner_pk: dlog_group::identity().into(), // TODO: get from bh.powSolution when its implemented
            votes: bh.votes.into(),
        }
    }
}
