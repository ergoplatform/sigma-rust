//! Block header
use num_bigint::BigInt;

use crate::sigma_protocol::dlog_group;

use super::block_id::BlockId;
use super::digest::{ADDigest, Digest32};
use super::modifier_id::ModifierId;
use super::votes::Votes;

/// Represents data of the block header available in Sigma propositions.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Header {
    /// Block version, to be increased on every soft and hardfork.
    pub version: u8,
    /// Bytes representation of ModifierId of this Header
    pub id: BlockId,
    /// Bytes representation of ModifierId of the parent block
    pub parent_id: ModifierId,
    /// Hash of ADProofs for transactions in a block
    pub ad_proofs_root: Digest32,
    /// AvlTree of a state after block application
    pub state_root: ADDigest,
    /// Root hash (for a Merkle tree) of transactions in a block.
    pub transaction_root: Digest32,
    /// Timestamp of a block in ms from UNIX epoch
    pub timestamp: u64,
    /// Current difficulty in a compressed view.
    pub n_bits: u64,
    /// Block height
    pub height: u32,
    /// Root hash of extension section
    pub extension_root: Digest32,
    /// Public key of miner. Part of Autolykos solution.
    pub miner_pk: Box<dlog_group::EcPoint>,
    /// One-time public key. Prevents revealing of miners secret.
    pub pow_onetime_pk: Box<dlog_group::EcPoint>,
    /// nonce
    pub nonce: Vec<u8>,
    /// Distance between pseudo-random number, corresponding to nonce `nonce` and a secret,
    /// corresponding to `miner_pk`. The lower `pow_distance` is, the harder it was to find this solution.
    pub pow_distance: BigInt,
    /// Miner votes for changing system parameters.
    /// 3 bytes in accordance to Scala implementation, but will use `Vec` until further improvements
    pub votes: Votes,
}

impl Header {
    /// Dummy instance intended for tests where actual values are not used
    pub fn dummy() -> Self {
        let empty_digest = Digest32::zero();
        Header {
            version: 1,
            id: BlockId(empty_digest.clone()),
            parent_id: ModifierId(empty_digest.clone()),
            ad_proofs_root: empty_digest.clone(),
            state_root: ADDigest::zero(),
            transaction_root: empty_digest.clone(),
            timestamp: 0,
            n_bits: 0,
            height: 0,
            extension_root: empty_digest,
            miner_pk: dlog_group::generator().into(),
            pow_onetime_pk: dlog_group::generator().into(),
            nonce: Vec::new(),
            pow_distance: BigInt::default(),
            votes: Votes([0u8; 3]),
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use num_bigint::BigInt;
    use proptest::array::{uniform3, uniform32};
    use proptest::prelude::*;

    use crate::chain::digest::{ADDigest, Digest};
    use crate::sigma_protocol::dlog_group::EcPoint;

    use super::{BlockId, Header, ModifierId, Votes};

    impl Arbitrary for Header {
        type Parameters = ();
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                // Timestamps between 2000-2050
                946_674_000_000..2_500_400_300_000u64,
                any::<u64>(),
                0..1_000_000u32,
                any::<Box<EcPoint>>(),
                any::<Box<EcPoint>>(),
                uniform3(1u8..),
            )
                .prop_map(
                    |(
                        id,
                        parent_id,
                        ad_proofs_root,
                        transaction_root,
                        extension_root,
                        timestamp,
                        n_bits,
                        height,
                        miner_pk,
                        pow_onetime_pk,
                        votes,
                    )| {
                        let id = BlockId(Digest(id.into()));
                        let parent_id = ModifierId(Digest(parent_id.into()));
                        let ad_proofs_root = Digest(ad_proofs_root.into());
                        let transaction_root = Digest(transaction_root.into());
                        let extension_root = Digest(extension_root.into());
                        let votes = Votes(votes);
                        Self {
                            version: 1,
                            id,
                            parent_id,
                            ad_proofs_root,
                            state_root: ADDigest::zero(),
                            transaction_root,
                            timestamp,
                            n_bits,
                            height,
                            extension_root,
                            miner_pk,
                            pow_onetime_pk,
                            nonce: Vec::new(),
                            pow_distance: BigInt::default(),
                            votes,
                        }
                    },
                )
                .boxed()
        }

        type Strategy = BoxedStrategy<Header>;
    }
}
