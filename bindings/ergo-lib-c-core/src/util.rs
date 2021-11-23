//! Utility functions and types

use crate::error::Error;

/// Try to cast const* pointer to immutable reference.
pub unsafe fn const_ptr_as_ref<'a, T>(
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
pub unsafe fn mut_ptr_as_mut<'a, T>(
    ptr: *mut T,
    ptr_name: &'static str,
) -> Result<&'a mut T, Error> {
    if let Some(r) = ptr.as_mut() {
        Ok(r)
    } else {
        Err(Error::InvalidArgument(ptr_name))
    }
}

#[derive(Clone)]
pub struct VecU8(pub(crate) Vec<u8>);

impl VecU8 {
    pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
        let slice = std::slice::from_raw_parts(ptr, len);
        VecU8(Vec::from(slice))
    }
}
