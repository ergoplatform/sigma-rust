//! Block header

use crate::{error::*, util::mut_ptr_as_mut};
use ergo_lib::ergotree_ir::chain::header::Header;

/// Block header
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockHeader(pub Header);
pub type BlockHeaderPtr = *mut BlockHeader;
pub type ConstBlockHeaderPtr = *const BlockHeader;

/// Parse BlockHeader array from JSON (Node API)
pub unsafe fn block_header_from_json(
    json: &str,
    block_header_out: *mut BlockHeaderPtr,
) -> Result<(), Error> {
    let block_header_out = mut_ptr_as_mut(block_header_out, "block_header_out")?;
    let header = serde_json::from_str(json).map(BlockHeader)?;
    *block_header_out = Box::into_raw(Box::new(header));
    Ok(())
}
