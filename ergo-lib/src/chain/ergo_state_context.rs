//! Blockchain state
use ergotree_ir::mir::header::PreHeader;

/// Blockchain state (last headers, etc.)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext {
    /// Block header with the current `spendingTransaction`, that can be predicted
    /// by a miner before it's formation
    pub pre_header: PreHeader,
}

impl ErgoStateContext {
    /// Dummy instance intended for tests where actual values are not used
    pub fn dummy() -> ErgoStateContext {
        ErgoStateContext {
            pre_header: PreHeader::dummy(),
        }
    }
}
