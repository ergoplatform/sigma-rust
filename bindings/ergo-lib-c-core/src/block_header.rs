//! Block header

use std::convert::{TryFrom, TryInto};

use crate::{
    error::*,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
};
use ergo_lib::ergo_chain_types::{Base16DecodedBytes, Header};

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
    *block_id_out = Box::into_raw(Box::new(BlockId(block_header.0.id)));
    Ok(())
}

/// Copy the contents of `transactions_root` field to `output`. Key assumption: exactly 32 bytes of
/// memory have been allocated at the address pointed-to by `output`.
pub unsafe fn block_header_transactions_root(
    block_header_ptr: ConstBlockHeaderPtr,
    output: *mut u8,
) -> Result<(), Error> {
    let block_header = const_ptr_as_ref(block_header_ptr, "block_header_ptr")?;
    let src = Vec::<u8>::from(block_header.0.transaction_root);
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, src.len());
    Ok(())
}

/// Block id
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockId(pub(crate) ergo_lib::ergo_chain_types::BlockId);

pub type BlockIdPtr = *mut BlockId;
pub type ConstBlockIdPtr = *const BlockId;

/// Convert a hex string into a BlockId
pub unsafe fn block_id_from_str(str: &str, block_id_out: *mut BlockIdPtr) -> Result<(), Error> {
    let block_id_out = mut_ptr_as_mut(block_id_out, "block_id_out")?;
    let bytes = Base16DecodedBytes::try_from(str.to_string())?;
    let block_id = bytes
        .try_into()
        .map(|digest| BlockId(ergo_lib::ergo_chain_types::BlockId(digest)))?;
    *block_id_out = Box::into_raw(Box::new(block_id));
    Ok(())
}
