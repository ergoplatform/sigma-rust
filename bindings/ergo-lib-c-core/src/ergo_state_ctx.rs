//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;

use crate::block_header::BlockHeader;
use crate::collections::ConstCollectionPtr;
use crate::header::PreHeader;
use crate::util::const_ptr_as_ref;
use crate::Error;
use std::convert::TryInto;

/// Blockchain state (last headers, etc.)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext(chain::ergo_state_context::ErgoStateContext);
pub type ErgoStateContextPtr = *mut ErgoStateContext;
pub type ConstErgoStateContextPtr = *const ErgoStateContext;

pub unsafe fn ergo_state_context_new(
    pre_header_ptr: *const PreHeader,
    headers: ConstCollectionPtr<BlockHeader>,
    ergo_state_context_out: *mut ErgoStateContextPtr,
) -> Result<(), Error> {
    let pre_header = const_ptr_as_ref(pre_header_ptr, "pre_header_ptr")?;
    let headers = const_ptr_as_ref(headers, "headers")?;
    match headers.0.len() {
        10 => {
            *ergo_state_context_out = Box::into_raw(Box::new(ErgoStateContext(
                chain::ergo_state_context::ErgoStateContext::new(
                    pre_header.clone().0,
                    headers
                        .0
                        .clone()
                        .into_iter()
                        .map(|x| x.0)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                ),
            )));
            Ok(())
        }
        h => Err(Error::Misc(
            format!("Not enough block headers, expected 10 but got {}", h).into(),
        )),
    }
}

pub fn ergo_state_context_delete(header: ErgoStateContextPtr) {
    if !header.is_null() {
        let boxed = unsafe { Box::from_raw(header) };
        std::mem::drop(boxed);
    }
}
