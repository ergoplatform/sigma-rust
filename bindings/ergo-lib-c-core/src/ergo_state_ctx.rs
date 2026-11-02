//! Ergo blockchain state (for ErgoTree evaluation)
use ergo_lib::chain;

use crate::block_header::BlockHeader;
use crate::collections::ConstCollectionPtr;
use crate::header::PreHeader;
use crate::parameters::ConstParametersPtr;
use crate::util::const_ptr_as_ref;
use crate::Error;

/// Blockchain state (last headers, etc.)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErgoStateContext(pub(crate) chain::ergo_state_context::ErgoStateContext);
pub type ErgoStateContextPtr = *mut ErgoStateContext;
pub type ConstErgoStateContextPtr = *const ErgoStateContext;

/// Create new context from pre-header
pub unsafe fn ergo_state_context_new(
    pre_header_ptr: *const PreHeader,
    headers: ConstCollectionPtr<BlockHeader>,
    parameters_ptr: ConstParametersPtr,
    ergo_state_context_out: *mut ErgoStateContextPtr,
) -> Result<(), Error> {
    let pre_header = const_ptr_as_ref(pre_header_ptr, "pre_header_ptr")?;
    let headers = const_ptr_as_ref(headers, "headers")?;
    let parameters = const_ptr_as_ref(parameters_ptr, "parameters_ptr")?;
    let count = headers.0.len();
    match chain::ergo_state_context::Headers::from_vec(
        headers.0.clone().into_iter().map(|x| x.0).collect(),
    ) {
        Ok(headers) => {
            *ergo_state_context_out = Box::into_raw(Box::new(ErgoStateContext(
                chain::ergo_state_context::ErgoStateContext::new(
                    pre_header.clone().0,
                    headers,
                    parameters.0.clone(),
                ),
            )));
            Ok(())
        }
        Err(_) => Err(Error::Misc(
            format!(
                "Incorrect number of block headers, expected 1..=10 but got {}",
                count
            )
            .into(),
        )),
    }
}

pub unsafe fn ergo_state_context_delete(header: ErgoStateContextPtr) {
    if !header.is_null() {
        let boxed = Box::from_raw(header);
        std::mem::drop(boxed);
    }
}
