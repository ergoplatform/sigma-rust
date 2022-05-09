//! Block header with the current `spendingTransaction`, that can be predicted by a miner before it's formation
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

use crate::block_header::BlockHeader;

/// Block header with the current `spendingTransaction`, that can be predicted
/// by a miner before it's formation
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct PreHeader(ergo_lib::ergotree_ir::chain::preheader::PreHeader);

#[wasm_bindgen]
impl PreHeader {
    /// Create using data from block header
    pub fn from_block_header(block_header: BlockHeader) -> Self {
        let bh: ergo_lib::ergo_chain_types::Header = block_header.into();
        let ph: ergo_lib::ergotree_ir::chain::preheader::PreHeader = bh.into();
        ph.into()
    }
}
