//! Block header

use crate::{error::*, util::mut_ptr_as_mut};
use ergo_lib::ergotree_ir::chain::header::Header;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockHeader(Header);
pub type BlockHeaderPtr = *mut BlockHeader;

pub unsafe fn block_header_from_json(
    json: &str,
    block_header_out: *mut BlockHeaderPtr,
) -> Result<(), Error> {
    let block_header_out = mut_ptr_as_mut(block_header_out, "block_header_out")?;
    let header = serde_json::from_str(json)
        .map(BlockHeader)
        .map_err(|_| Error::Misc("BlockHeader: can't deserialize from JSON".into()))?;
    *block_header_out = Box::into_raw(Box::new(header));
    Ok(())
}

pub fn block_header_delete(header: BlockHeaderPtr) {
    if !header.is_null() {
        let boxed = unsafe { Box::from_raw(header) };
        std::mem::drop(boxed);
    }
}
