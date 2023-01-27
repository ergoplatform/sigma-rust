//! Block on the Ergo chain

use bounded_vec::BoundedVec;
use ergo_chain_types::Header;
use serde::{Deserialize, Serialize};

use super::transaction::Transaction;

/// Maximum number of transactions that can be contained in a block. See
/// https://github.com/ergoplatform/ergo/blob/fc292f6bc2d3c6ca27ce5f6a316186d8459150cc/src/main/scala/org/ergoplatform/modifiers/history/BlockTransactions.scala#L157
const MAX_NUM_TRANSACTIONS: usize = 10_000_000;

/// Transactions in a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTransactions {
    /// Transactions contained in the block
    pub transactions: BoundedVec<Transaction, 1, MAX_NUM_TRANSACTIONS>,
}

/// A block on the Ergo chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullBlock {
    /// Block header
    pub header: Header,
    /// Transactions in this block
    pub block_transactions: BlockTransactions,
}
