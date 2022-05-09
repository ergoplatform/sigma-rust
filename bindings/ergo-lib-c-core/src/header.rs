//! Block header with the current `spendingTransaction`, that can be predicted by a miner before it's formation

use crate::{block_header::ConstBlockHeaderPtr, error::Error, util::const_ptr_as_ref};

/// Block header with the current `spendingTransaction`, that can be predicted
/// by a miner before its formation
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PreHeader(pub ergo_lib::ergotree_ir::chain::preheader::PreHeader);
pub type PreHeaderPtr = *mut PreHeader;
pub type ConstPreHeaderPtr = *const PreHeader;

/// Create instance using data from block header
pub unsafe fn preheader_from_block_header(
    block_header_ptr: ConstBlockHeaderPtr,
    preheader_out: *mut PreHeaderPtr,
) -> Result<(), Error> {
    let block_header = const_ptr_as_ref(block_header_ptr, "block_header")?;
    let bh: ergo_lib::ergo_chain_types::Header = block_header.0.clone();
    let ph: ergo_lib::ergotree_ir::chain::preheader::PreHeader = bh.into();
    *preheader_out = Box::into_raw(Box::new(PreHeader(ph)));
    Ok(())
}
