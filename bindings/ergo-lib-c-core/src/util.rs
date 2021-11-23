//! Utility functions and types

use crate::error::Error;

/// Try to cast const* pointer to immutable reference.
pub(crate) unsafe fn const_ptr_as_ref<'a, T>(
    ptr: *const T,
    ptr_name: &'static str,
) -> Result<&'a T, Error> {
    if let Some(r) = ptr.as_ref() {
        Ok(r)
    } else {
        Err(Error::InvalidArgument(ptr_name))
    }
}

/// Try to cast mut* pointer to mutable reference.
pub(crate) unsafe fn mut_ptr_as_mut<'a, T>(
    ptr: *mut T,
    ptr_name: &'static str,
) -> Result<&'a mut T, Error> {
    if let Some(r) = ptr.as_mut() {
        Ok(r)
    } else {
        Err(Error::InvalidArgument(ptr_name))
    }
}

/// Simple wrapper around a `Vec<u8>`.
#[derive(Clone)]
pub struct ByteArray(pub Vec<u8>);
pub type ByteArrayPtr = *mut ByteArray;
pub type ConstByteArrayPtr = *const ByteArray;

pub unsafe fn byte_array_from_raw_parts(
    ptr: *const u8,
    len: usize,
    byte_array_out: *mut ByteArrayPtr,
) -> Result<(), Error> {
    let slice = std::slice::from_raw_parts(ptr, len);
    let byte_array_out = mut_ptr_as_mut(byte_array_out, "byte_array_out")?;
    *byte_array_out = Box::into_raw(Box::new(ByteArray(Vec::from(slice))));
    Ok(())
}
