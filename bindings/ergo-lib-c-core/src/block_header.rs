//! Block header

use crate::{
    error::*,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
};
use ergo_lib::ergo_chain_types::Header;

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

/// Get `BlockHeader`s id
pub unsafe fn block_header_id(
    block_header_ptr: ConstBlockHeaderPtr,
    block_id_out: *mut BlockIdPtr,
) -> Result<(), Error> {
    let block_header = const_ptr_as_ref(block_header_ptr, "block_header_ptr")?;
    let block_id_out = mut_ptr_as_mut(block_id_out, "block_id_out")?;
    *block_id_out = Box::into_raw(Box::new(BlockId(block_header.0.id.clone())));
    Ok(())
}

/// Block id
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockId(pub(crate) ergo_lib::ergo_chain_types::BlockId);
pub type BlockIdPtr = *mut BlockId;
pub type ConstBlockIdPtr = *const BlockId;
