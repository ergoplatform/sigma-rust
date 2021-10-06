//! Blockchain state
use std::convert::TryInto;

use ergotree_ir::chain::header::Header;
use ergotree_ir::chain::preheader::PreHeader;

/// Blockchain state (last headers, etc.)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext {
    /// Block header with the current `spendingTransaction`, that can be predicted
    /// by a miner before it's formation
    pub pre_header: PreHeader,
    /// Fixed number of last block headers in descending order (first header is the newest one)
    pub headers: [Header; 10],
}

impl ErgoStateContext {
    /// Dummy instance intended for tests where actual values are not used
    pub fn dummy() -> ErgoStateContext {
        let headers = vec![Header::dummy(); 10]
            .try_into()
            .expect("internal error: Headers array length isn't eq to 10");
        ErgoStateContext {
            pre_header: PreHeader::dummy(),
            headers,
        }
    }
}
