//! Ergo constant values

use crate::{util::mut_ptr_as_mut, Error};

/// Ergo constant(evaluated) values
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant(pub(crate) ergo_lib::ergotree_ir::mir::constant::Constant);
pub type ConstantPtr = *mut Constant;
pub type ConstConstantPtr = *const Constant;

pub unsafe fn constant_from_i32(constant_out: *mut ConstantPtr, value: i32) -> Result<(), Error> {
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    *constant_out = Box::into_raw(Box::new(Constant(value.into())));
    Ok(())
}
