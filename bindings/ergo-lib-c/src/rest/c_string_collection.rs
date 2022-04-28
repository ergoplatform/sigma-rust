use std::os::raw::c_char;

use ergo_lib_c_core::rest::c_string_collection::*;

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_c_string_collection_get_ptr(
    c_string_collection_ptr: CStringCollectionPtr,
) -> *const *const c_char {
    #[allow(clippy::unwrap_used)]
    c_string_collection_get_ptr(c_string_collection_ptr).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_c_string_collection_get_length(
    c_string_collection_ptr: CStringCollectionPtr,
) -> usize {
    #[allow(clippy::unwrap_used)]
    c_string_collection_get_length(c_string_collection_ptr).unwrap()
}

/// Drop `CStringCollection`
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_c_string_collection_delete(ptr: CStringCollectionPtr) {
    delete_c_string_collection(ptr);
}
