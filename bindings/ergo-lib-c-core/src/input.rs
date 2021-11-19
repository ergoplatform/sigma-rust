//! Ergo input

use ergo_lib::chain;

use crate::{
    context_extension::{ContextExtension, ContextExtensionPtr},
    ergo_box::{BoxId, BoxIdPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Unsigned inputs used in constructing unsigned transactions
#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedInput(pub chain::transaction::UnsignedInput);
pub type UnsignedInputPtr = *mut UnsignedInput;
pub type ConstUnsignedInputPtr = *const UnsignedInput;

pub unsafe fn unsigned_input_box_id(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    box_id_out: *mut BoxIdPtr,
) -> Result<(), Error> {
    let box_id_out = mut_ptr_as_mut(box_id_out, "box_id_out")?;
    let unsigned_input = const_ptr_as_ref(unsigned_input_ptr, "unsigned_input_ptr")?;
    let box_id = BoxId(unsigned_input.0.box_id.clone());
    *box_id_out = Box::into_raw(Box::new(box_id));
    Ok(())
}

pub unsafe fn unsigned_input_context_extension(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    context_extension_out: *mut ContextExtensionPtr,
) -> Result<(), Error> {
    let context_extension_out = mut_ptr_as_mut(context_extension_out, "context_extension_out")?;
    let unsigned_input = const_ptr_as_ref(unsigned_input_ptr, "unsigned_input_ptr")?;
    let context_extension = ContextExtension(unsigned_input.0.extension.clone());
    *context_extension_out = Box::into_raw(Box::new(context_extension));
    Ok(())
}
