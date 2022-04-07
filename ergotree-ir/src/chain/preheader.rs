//! Block header with fields that can be predicted by miner

use ergo_chain_types::BlockId;

use crate::sigma_protocol::dlog_group;

use super::votes::Votes;

/// Block header with the current `spendingTransaction`, that can be predicted
/// by a miner before it's formation
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PreHeader {
    /// Block version, to be increased on every soft and hardfork
    pub version: u8,
    /// Hash of parent block
    pub parent_id: BlockId,
    /// Timestamp of a block in ms from UNIX epoch
    pub timestamp: u64,
    /// Current difficulty in a compressed view.
    pub n_bits: u64,
    /// Block height
    pub height: u32,
    /// Public key of miner
    pub miner_pk: Box<dlog_group::EcPoint>,
    /// Votes
    pub votes: Votes,
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use proptest::array::{uniform3, uniform32};
    use proptest::prelude::*;

    use crate::sigma_protocol::dlog_group::EcPoint;

    use super::*;

    impl Arbitrary for PreHeader {
        type Parameters = ();
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                uniform32(1u8..),
                // Timestamps between 2000-2050
                946_674_000_000..2_500_400_300_000u64,
                any::<u64>(),
                1_000_000u32..10_000_000u32,
                any::<Box<EcPoint>>(),
                uniform3(1u8..),
            )
                .prop_map(|(parent_id, timestamp, n_bits, height, miner_pk, votes)| {
                    let parent_id = BlockId(parent_id.into());
                    let votes = Votes(votes);
                    Self {
                        version: 1,
                        parent_id,
                        timestamp,
                        n_bits,
                        height,
                        miner_pk,
                        votes,
                    }
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<PreHeader>;
    }
}
