use crate::{
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};
use ergo_lib::ergotree_interpreter::sigma_protocol::prover;

/// User-defined variables to be put into context
#[derive(PartialEq, Debug, Clone)]
pub struct ContextExtension(pub prover::ContextExtension);
pub type ContextExtensionPtr = *mut ContextExtension;
pub type ConstContextExtensionPtr = *const ContextExtension;

/// Create new empty ContextExtension instance
pub unsafe fn context_extension_empty(
    context_extension_out: *mut ContextExtensionPtr,
) -> Result<(), Error> {
    let context_extension_out = mut_ptr_as_mut(context_extension_out, "context_extension_out")?;
    *context_extension_out = Box::into_raw(Box::new(ContextExtension(
        prover::ContextExtension::empty(),
    )));
    Ok(())
}

/// Returns the number of elements in the collection
pub unsafe fn context_extension_len(
    context_extension_ptr: ConstContextExtensionPtr,
) -> Result<usize, Error> {
    let context_extension = const_ptr_as_ref(context_extension_ptr, "context_extension_ptr")?;
    Ok(context_extension.0.values.len())
}

/// Returns all keys (represented as u8 values) in the map
pub unsafe fn context_extension_keys(
    context_extension_ptr: ConstContextExtensionPtr,
    output: *mut u8,
) -> Result<(), Error> {
    let context_extension = const_ptr_as_ref(context_extension_ptr, "context_extension_ptr")?;
    let src: Vec<_> = context_extension.0.values.keys().cloned().collect();
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, src.len());
    Ok(())
}

// TODO: get method (needs Constant)
