/// `ergo-lib` uses a number of collection types that are simple wrappers around `Vec`. We have a
/// generic type for such a collection in `ergo-lib-c-core::collections::Collection`. A limitation
/// of `cbindgen` is that it cannot process generic functions. This macro generates a C-compatible
/// interface for such a collection for any desired type T.
///
/// All generated names (the pointer type aliases and the exported functions) are spelled out at
/// the call site: `macro_rules!` cannot concatenate identifiers, and spelling them out keeps the
/// exported C symbols greppable in the source.
///
/// As an example the call
/// `make_collection!{ collection: BlockHeaders, collection_ptr: BlockHeadersPtr, const_collection_ptr: ConstBlockHeadersPtr, item: BlockHeader, item_ptr: BlockHeaderPtr, const_item_ptr: ConstBlockHeaderPtr, new_fn: ergo_lib_block_headers_new, delete_fn: ergo_lib_block_headers_delete, len_fn: ergo_lib_block_headers_len, get_fn: ergo_lib_block_headers_get, add_fn: ergo_lib_block_headers_add }`
/// generates:
///
///```
///pub type BlockHeadersPtr = CollectionPtr<BlockHeader>;
///pub type ConstBlockHeadersPtr = ConstCollectionPtr<BlockHeader>;
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_lib_block_headers_new(collection_ptr_out: *mut BlockHeadersPtr) {
///    #[allow(clippy::unwrap_used)]
///    collection_new(collection_ptr_out).unwrap();
///}
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_lib_block_headers_delete(collection_ptr_out: BlockHeadersPtr) {
///    collection_delete(collection_ptr_out)
///}
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_lib_block_headers_len(
///    collection_ptr: ConstBlockHeadersPtr,
///) -> usize {
///    #[allow(clippy::unwrap_used)]
///    collection_len(collection_ptr).unwrap()
///}
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_lib_block_headers_get(
///    collection_ptr: ConstBlockHeadersPtr,
///    index: usize,
///    element_ptr_out: *mut BlockHeaderPtr,
///) -> ReturnOption {
///    match collection_get(collection_ptr, index, element_ptr_out) {
///       Ok(is_some) => crate::ReturnOption {
///           is_some,
///           error: std::ptr::null_mut(),
///       },
///       Err(e) => crate::ReturnOption {
///           is_some: false, // Just a dummy value
///           error: Error::c_api_from(Err(e)),
///       },
///    }
///}
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_lib_block_headers_add(
///    element_ptr: ConstBlockHeaderPtr,
///    collection_ptr_out: BlockHeadersPtr,
///) {
///    #[allow(clippy::unwrap_used)]
///    collection_add(collection_ptr_out, element_ptr).unwrap();
///}
///```
#[macro_export]
macro_rules! make_collection {
    (
        collection: $collection_type_name:ident,
        collection_ptr: $collection_ptr_name:ident,
        const_collection_ptr: $const_collection_ptr_name:ident,
        item: $item_type_name:ident,
        item_ptr: $item_ptr_name:ident,
        const_item_ptr: $const_item_ptr_name:ident,
        new_fn: $new_fn_name:ident,
        delete_fn: $delete_fn_name:ident,
        len_fn: $len_fn_name:ident,
        get_fn: $get_fn_name:ident,
        add_fn: $add_fn_name:ident
    ) => {
        pub type $collection_ptr_name =
            ergo_lib_c_core::collections::CollectionPtr<$item_type_name>;
        pub type $const_collection_ptr_name =
            ergo_lib_c_core::collections::ConstCollectionPtr<$item_type_name>;

        /// Create a new empty collection
        #[no_mangle]
        pub unsafe extern "C" fn $new_fn_name(collection_ptr_out: *mut $collection_ptr_name) {
            #[allow(clippy::unwrap_used)]
            ergo_lib_c_core::collections::collection_new(collection_ptr_out).unwrap();
        }

        /// Delete an existing collection
        #[no_mangle]
        pub unsafe extern "C" fn $delete_fn_name(ptr_out: $collection_ptr_name) {
            ergo_lib_c_core::collections::collection_delete(ptr_out)
        }

        /// Returns length of an existing collection
        #[no_mangle]
        pub unsafe extern "C" fn $len_fn_name(collection_ptr: $const_collection_ptr_name) -> usize {
            #[allow(clippy::unwrap_used)]
            ergo_lib_c_core::collections::collection_len(collection_ptr).unwrap()
        }

        /// Returns element at position `index` of an existing collection
        #[no_mangle]
        pub unsafe extern "C" fn $get_fn_name(
            collection_ptr: $const_collection_ptr_name,
            index: usize,
            element_ptr_out: *mut $item_ptr_name,
        ) -> $crate::ReturnOption {
            match ergo_lib_c_core::collections::collection_get(
                collection_ptr,
                index,
                element_ptr_out,
            ) {
                Ok(is_some) => $crate::ReturnOption {
                    is_some,
                    error: std::ptr::null_mut(),
                },
                Err(e) => $crate::ReturnOption {
                    is_some: false, // Just a dummy value
                    error: Error::c_api_from(Err(e)),
                },
            }
        }

        #[no_mangle]
        /// Add an element to collection
        pub unsafe extern "C" fn $add_fn_name(
            element_ptr: $const_item_ptr_name,
            collection_ptr_out: $collection_ptr_name,
        ) {
            #[allow(clippy::unwrap_used)]
            ergo_lib_c_core::collections::collection_add(collection_ptr_out, element_ptr).unwrap();
        }
    };
}

/// Generates an equality function for FFI. The exported function name and the const pointer type
/// of the compared values are spelled out at the call site (see `make_collection!` for why).
#[macro_export]
macro_rules! make_ffi_eq {
    (
        eq_fn: $eq_fn_name:ident,
        const_ptr: $const_ptr_name:ident
    ) => {
        #[no_mangle]
        pub unsafe extern "C" fn $eq_fn_name(
            ptr_0: $const_ptr_name,
            ptr_1: $const_ptr_name,
        ) -> bool {
            ergo_lib_c_core::util::deref_eq(ptr_0, ptr_1)
        }
    };
}
