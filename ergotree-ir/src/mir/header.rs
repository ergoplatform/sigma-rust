use num_bigint::BigInt;

use crate::sigma_protocol::dlog_group;

/// Block header with the current `spendingTransaction`, that can be predicted
/// by a miner before it's formation
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PreHeader {
    /// Block version, to be increased on every soft and hardfork
    pub version: u8,
    /// Hash of parent block
    pub parent_id: Vec<u8>,
    /// Timestamp of a block in ms from UNIX epoch
    pub timestamp: u64,
    /// Current difficulty in a compressed view.
    pub n_bits: u64,
    /// Block height
    pub height: u32,
    /// Public key of miner
    pub miner_pk: Box<dlog_group::EcPoint>,
    /// Votes
    pub votes: Vec<u8>,
}

/// Represents data of the block header available in Sigma propositions.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Header {
    /// Block version, to be increased on every soft and hardfork.
    pub version: u8,
    /// Bytes representation of ModifierId of this Header
    pub id: Vec<u8>,
    /// Bytes representation of ModifierId of the parent block
    pub parent_id: Vec<u8>,
    /// Hash of ADProofs for transactions in a block
    pub ad_proofs_root: Vec<u8>,
    /// AvlTree of a state after block application
    pub state_root: AvlTree,
    /// Root hash (for a Merkle tree) of transactions in a block.
    pub transaction_root: Vec<u8>,
    /// Timestamp of a block in ms from UNIX epoch
    pub timestamp: u64,
    /// Current difficulty in a compressed view.
    pub n_bits: u64,
    /// Block height
    pub height: u32,
    /// Root hash of extension section
    pub extension_root: Vec<u8>,
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
    pub votes: Vec<u8>,
}

/// Temporary, until not implemented https://github.com/ergoplatform/sigma-rust/issues/368.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AvlTree;

impl PreHeader {
    /// Dummy instance intended for tests where actual values are not used
    pub fn dummy() -> Self {
        PreHeader {
            version: 1,
            parent_id: vec![0; 32],
            timestamp: 0,
            n_bits: 0,
            height: 0,
            miner_pk: dlog_group::generator().into(),
            votes: Vec::new(),
        }
    }
}

impl Header {
    /// Dummy instance intended for tests where actual values are not used
    pub fn dummy() -> Self {
        Header {
            version: 1,
            id: vec![0; 32],
            parent_id: vec![0; 32],
            ad_proofs_root: vec![0; 32],
            state_root: AvlTree,
            transaction_root: vec![0; 32],
            timestamp: 0,
            n_bits: 0,
            height: 0,
            extension_root: vec![0; 32],
            miner_pk: dlog_group::generator().into(),
            pow_onetime_pk: dlog_group::generator().into(),
            nonce: Vec::new(),
            pow_distance: BigInt::default(),
            votes: Vec::new()
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use num_bigint::BigInt;
    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::mir::header::{AvlTree, Header, PreHeader};
    use crate::sigma_protocol::dlog_group::EcPoint;

    impl Arbitrary for PreHeader {
        type Parameters = ();
        type Strategy = BoxedStrategy<PreHeader>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                vec(any::<u8>(), 32),
                // Timestamps between 2000-2050
                946_674_000_000..2_500_400_300_000u64,
                any::<u64>(),
                0..1_000_000u32,
                any::<Box<EcPoint>>(),
            )
                .prop_map(|(parent_id, timestamp, n_bits, height, miner_pk)| Self {
                    version: 1,
                    parent_id,
                    timestamp,
                    n_bits,
                    height,
                    miner_pk,
                    votes: Vec::new(),
                })
                .boxed()
        }
    }

    impl Arbitrary for Header {
        type Parameters = ();
        type Strategy = BoxedStrategy<Header>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                vec(vec(any::<u8>(), 32), 5),
                // Timestamps between 2000-2050
                946_674_000_000..2_500_400_300_000u64,
                any::<u64>(),
                0..1_000_000u32,
                vec(any::<Box<EcPoint>>(), 2),
            )
                .prop_map(
                    |(mut arbitrary_byte_data, timestamp, n_bits, height, mut pks)| Self {
                        version: 1,
                        id: arbitrary_byte_data
                            .pop()
                            .expect("internal error: empty vec with arbitrary data"),
                        parent_id: arbitrary_byte_data
                            .pop()
                            .expect("internal error: empty vec with arbitrary data"),
                        ad_proofs_root: arbitrary_byte_data
                            .pop()
                            .expect("internal error: empty vec with arbitrary data"),
                        state_root: AvlTree,
                        transaction_root: arbitrary_byte_data
                            .pop()
                            .expect("internal error: empty vec with arbitrary data"),
                        timestamp,
                        n_bits,
                        height,
                        extension_root: arbitrary_byte_data
                            .pop()
                            .expect("internal error: empty vec with arbitrary data"),
                        miner_pk: pks.pop().expect("internal error: empty vec with pk data"),
                        pow_onetime_pk: pks.pop().expect("internal error: empty vec with pk data"),
                        nonce: Vec::new(),
                        pow_distance: BigInt::default(),
                        votes: Vec::new(),
                    }
                )
                .boxed()
        }
    }
}
