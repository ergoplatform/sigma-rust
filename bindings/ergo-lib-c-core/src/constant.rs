//! Ergo constant values

use crate::{
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

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

pub unsafe fn constant_eq(
    constant_ptr_0: ConstConstantPtr,
    constant_ptr_1: ConstConstantPtr,
) -> Result<bool, Error> {
    let constant_0 = const_ptr_as_ref(constant_ptr_0, "constant_ptr_0")?;
    let constant_1 = const_ptr_as_ref(constant_ptr_1, "constant_ptr_1")?;
    Ok(constant_0 == constant_1)
}
