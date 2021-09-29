//! Block header
use num_bigint::BigInt;

use ergotree_ir::chain::{
    block_id::BlockId,
    digest::{ADDigest, Digest32},
    header::Header,
    votes::Votes,
};
use ergotree_ir::sigma_protocol::dlog_group;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// New-type wrapper for deserializing the remote `Header` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderJsonHelper(#[serde(with = "BlockHeaderRef")] pub Header);

/// Block header reference to `Header` type in ergotree-ir.
///
/// We do not implement serde traits in ergotree-ir, but they are requested
/// by crate users. So we split definition into the main one in ergotree-ir and
/// the one which implements `Serialize` and `Deserialize` (which is here).
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(remote = "Header"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub(in crate::chain) struct BlockHeaderRef {
    /// Block version, to be increased on every soft and hardfork
    version: u8,
    /// Bytes representation of ModifierId of this Header
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    id: BlockId,
    /// Id of a parent block
    #[cfg_attr(
        feature = "json",
        serde(rename = "parentId", with = "block_id::BlockIdRef")
    )]
    parent_id: BlockId,
    /// Hash of ADProofs for transactions in a block
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    ad_proofs_root: Digest32,
    /// AvlTree of a state after block application
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    state_root: ADDigest,
    /// Root hash (for a Merkle tree) of transactions in a block.
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    transaction_root: Digest32,
    /// Timestamp of a block in ms from UNIX epoch
    timestamp: u64,
    /// Current difficulty in a compressed view.
    #[cfg_attr(feature = "json", serde(rename = "nBits"))]
    n_bits: u64,
    /// Block height
    height: u32,
    /// Root hash of extension section
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    extension_root: Digest32,
    /// Public key of miner. Part of Autolykos solution.
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    miner_pk: Box<dlog_group::EcPoint>,
    /// One-time public key. Prevents revealing of miners secret.
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    pow_onetime_pk: Box<dlog_group::EcPoint>,
    /// nonce
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    nonce: Vec<u8>,
    /// Distance between pseudo-random number, corresponding to nonce `nonce` and a secret,
    /// corresponding to `miner_pk`. The lower `pow_distance` is, the harder it was to find this solution.
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    pow_distance: BigInt,
    /// Votes
    #[cfg_attr(feature = "json", serde(with = "votes::VotesRef"))]
    votes: Votes,
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
