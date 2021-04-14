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
    pub height: i32,
    /// Public key of miner
    pub miner_pk: Box<dlog_group::EcPoint>,
    /// Votes
    pub votes: Vec<u8>,
}

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
