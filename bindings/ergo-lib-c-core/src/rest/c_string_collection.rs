use crate::error::*;
use crate::util::{const_ptr_as_ref, mut_ptr_as_mut};
use std::ffi::CString;
use std::os::raw::c_char;

/// Wrapper around a C-array of strings.
#[repr(C)]
pub(crate) struct CStringCollectionInner {
    /// Pointer to the first string of the array
    pub ptr: *mut *mut c_char,
    /// Number of strings in the array
    pub length: usize,
}

#[derive(derive_more::From, derive_more::Into)]
pub struct CStringCollection(pub(crate) CStringCollectionInner);
pub type CStringCollectionPtr = *mut CStringCollection;

impl ergo_lib::ergo_rest::NodeResponse for CStringCollection {}

/// Returns underlying pointer of the array
pub unsafe fn c_string_collection_get_ptr(
    c_string_collection_ptr: CStringCollectionPtr,
) -> Result<*const *const c_char, Error> {
    let c_string_collection = const_ptr_as_ref(c_string_collection_ptr, "c_string_collection_ptr")?;
    Ok(c_string_collection.0.ptr as *const *const c_char)
}

/// Returns the length of the underlying array
pub unsafe fn c_string_collection_get_length(
    c_string_collection_ptr: CStringCollectionPtr,
) -> Result<usize, Error> {
    let c_string_collection = const_ptr_as_ref(c_string_collection_ptr, "c_string_collection_ptr")?;
    Ok(c_string_collection.0.length)
}

/// Deletes a rust allocated Vec of `CString`s. IMPORTANT ASSUMPTION: the vec's length
/// is equal to its capacity.
#[no_mangle]
pub unsafe fn delete_c_string_collection(ptr: CStringCollectionPtr) {
    let coll = mut_ptr_as_mut(ptr, "ptr").unwrap();
    let len = coll.0.length;
    let v = Vec::from_raw_parts(coll.0.ptr, len, len);

    for elem in v {
        let s = CString::from_raw(elem);
        std::mem::drop(s);
    }
}
