//! Block header with the current `spendingTransaction`, that can be predicted by a miner before it's formation

use crate::{block_header::ConstBlockHeaderPtr, error::Error, util::const_ptr_as_ref};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PreHeader(pub ergo_lib::ergotree_ir::chain::preheader::PreHeader);
pub type PreHeaderPtr = *mut PreHeader;
pub type ConstPreHeaderPtr = *const PreHeader;

pub unsafe fn preheader_from_block_header(
    block_header: ConstBlockHeaderPtr,
    preheader_out: *mut PreHeaderPtr,
) -> Result<(), Error> {
    let block_header = const_ptr_as_ref(block_header, "block_header")?;
    let bh: ergo_lib::ergotree_ir::chain::header::Header = block_header.0.clone();
    let ph: ergo_lib::ergotree_ir::chain::preheader::PreHeader = bh.into();
    *preheader_out = Box::into_raw(Box::new(PreHeader(ph)));
    Ok(())
}

pub fn preheader_delete(header: PreHeaderPtr) {
    if !header.is_null() {
        let boxed = unsafe { Box::from_raw(header) };
        std::mem::drop(boxed);
    }
}
