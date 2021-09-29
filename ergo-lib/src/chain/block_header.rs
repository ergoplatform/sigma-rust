//! Block header
// todo-sab ergotree-ir::chain::Header types is the same as BlockHeader - remove duplicate

use ergotree_ir::chain::{block_id::BlockId, votes::Votes};
use ergotree_ir::mir::header::PreHeader;
use ergotree_ir::sigma_protocol::dlog_group;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

// todo-sab remote Header? not pub?
/// Block header
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockHeader {
    /// Block version, to be increased on every soft and hardfork
    pub version: u8,
    /// Id of a parent block
    #[cfg_attr(
        feature = "json",
        serde(rename = "parentId", with = "block_id::BlockIdRef")
    )]
    pub parent_id: BlockId,
    /// Timestamp of a block in ms from UNIX epoch
    pub timestamp: u64,
    /// Current difficulty in a compressed view.
    #[cfg_attr(feature = "json", serde(rename = "nBits"))]
    pub n_bits: u64,
    /// Block height
    pub height: u32,
    /// Votes
    #[cfg_attr(feature = "json", serde(with = "votes::VotesRef"))]
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

mod votes {
    use std::convert::{TryFrom, TryInto};

    use ergotree_ir::chain::votes::{Votes, VotesError};
    #[cfg(feature = "json")]
    use serde::{Deserialize, Serialize};

    use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};

    /// Reference for remote Votes type. Remote Votes wasn't used, because in ergo-lib
    /// this type is mostly needed for json serialization and deserialization. Such traits
    /// of Votes aren't needed in ergotree-ir.
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
    pub(super) struct VotesRef([u8; 3]);

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
}

mod block_id {
    use ergotree_ir::chain::{block_id::BlockId, digest::Digest32};
    #[cfg(feature = "json")]
    use serde::{Deserialize, Serialize};

    use crate::chain::DigestRef;

    /// Reference for BlockId type. Remote BlockId wasn't used, because in ergo-lib
    /// this type is mostly needed for json serialization and deserialization. Such traits
    /// of BlockId aren't needed in ergotree-ir.
    #[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "json", serde(remote = "BlockId"))]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub(super) struct BlockIdRef(#[serde(with = "DigestRef")] Digest32);
}
