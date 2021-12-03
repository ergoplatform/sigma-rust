//! Ergo constant values

use std::convert::TryFrom;

use crate::{
    ergo_box::{ConstErgoBoxPtr, ErgoBox, ErgoBoxPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};
use ergo_lib::ergotree_ir::{
    base16_str::Base16Str,
    chain::base16_bytes::Base16DecodedBytes,
    mir::constant::{TryExtractFrom, TryExtractInto},
    serialization::SigmaSerializable,
    sigma_protocol::{dlog_group::EcPoint, sigma_boolean::ProveDlog},
};

/// Ergo constant(evaluated) values
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant(pub(crate) ergo_lib::ergotree_ir::mir::constant::Constant);
pub type ConstantPtr = *mut Constant;
pub type ConstConstantPtr = *const Constant;

/// Decode from base16 encoded serialized ErgoTree
pub unsafe fn constant_from_base16_bytes(
    bytes_str: &str,
    constant_out: *mut ConstantPtr,
) -> Result<(), Error> {
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    let bytes = Base16DecodedBytes::try_from(bytes_str.to_string())?;
    let constant = ergo_lib::ergotree_ir::mir::constant::Constant::try_from(bytes).map(Constant)?;
    *constant_out = Box::into_raw(Box::new(constant));
    Ok(())
}

/// Encode as Base16-encoded ErgoTree serialized value or return an error if serialization failed
pub unsafe fn constant_to_base16_str(constant_ptr: ConstConstantPtr) -> Result<String, Error> {
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let s = constant.0.base16_str()?;
    Ok(s)
}

/// Create from i32 value
pub unsafe fn constant_from_i32(value: i32, constant_out: *mut ConstantPtr) -> Result<(), Error> {
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    *constant_out = Box::into_raw(Box::new(Constant(value.into())));
    Ok(())
}

/// Extract i32 value, returning error if wrong type
pub unsafe fn constant_to_i32(constant_ptr: ConstConstantPtr) -> Result<i32, Error> {
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let i = i32::try_extract_from(constant.0.clone())?;
    Ok(i)
}

/// Create from i64
pub unsafe fn constant_from_i64(value: i64, constant_out: *mut ConstantPtr) -> Result<(), Error> {
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    *constant_out = Box::into_raw(Box::new(Constant(value.into())));
    Ok(())
}

/// Extract i64 value, returning error if wrong type
pub unsafe fn constant_to_i64(constant_ptr: ConstConstantPtr) -> Result<i64, Error> {
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let i = i64::try_extract_from(constant.0.clone())?;
    Ok(i)
}

/// Create from byte array
pub unsafe fn constant_from_bytes(
    bytes_ptr: *const u8,
    len: usize,
    constant_out: *mut ConstantPtr,
) -> Result<(), Error> {
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    let bytes = std::slice::from_raw_parts(bytes_ptr, len);
    *constant_out = Box::into_raw(Box::new(Constant(bytes.to_vec().into())));
    Ok(())
}

/// Extract byte array length, returning error if wrong type
pub unsafe fn constant_bytes_len(constant_ptr: ConstConstantPtr) -> Result<usize, Error> {
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let len = Vec::<u8>::try_extract_from(constant.0.clone()).map(|v| v.len())?;
    Ok(len)
}

/// Convert to serialized bytes. Key assumption: enough memory has been allocated at the address
/// pointed-to by `output`. Use `constant_bytes_len` to determine the length of the byte array.
pub unsafe fn constant_to_bytes(
    constant_ptr: ConstConstantPtr,
    output: *mut u8,
) -> Result<(), Error> {
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let src = Vec::<u8>::try_extract_from(constant.0.clone())?;
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, src.len());
    Ok(())
}

/// Parse raw [`EcPoint`] value from bytes and make [`ProveDlog`] constant
pub unsafe fn constant_from_ecpoint_bytes(
    bytes_ptr: *const u8,
    len: usize,
    constant_out: *mut ConstantPtr,
) -> Result<(), Error> {
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    let bytes = std::slice::from_raw_parts(bytes_ptr, len);
    let ecp = EcPoint::sigma_parse_bytes(bytes)?;
    let c: ergo_lib::ergotree_ir::mir::constant::Constant = ProveDlog::new(ecp).into();
    *constant_out = Box::into_raw(Box::new(Constant(c)));
    Ok(())
}

/// Create from ErgoBox value
pub unsafe fn constant_from_ergo_box(
    ergo_box_ptr: ConstErgoBoxPtr,
    constant_out: *mut ConstantPtr,
) -> Result<(), Error> {
    let ergo_box = const_ptr_as_ref(ergo_box_ptr, "ergo_box_ptr")?;
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    let c: ergo_lib::ergotree_ir::mir::constant::Constant = ergo_box.0.clone().into();
    *constant_out = Box::into_raw(Box::new(Constant(c)));
    Ok(())
}

/// Extract ErgoBox value, returning error if wrong type
pub unsafe fn constant_to_ergo_box(
    constant_ptr: ConstConstantPtr,
    ergo_box_out: *mut ErgoBoxPtr,
) -> Result<(), Error> {
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let ergo_box_out = mut_ptr_as_mut(ergo_box_out, "ergo_box_out")?;
    let b = constant
        .0
        .clone()
        .try_extract_into::<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox>()
        .map(Into::into)?;
    *ergo_box_out = Box::into_raw(Box::new(ErgoBox(b)));
    Ok(())
}
