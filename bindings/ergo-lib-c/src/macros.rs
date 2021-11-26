#[allow(unused_imports)]
use paste::paste;

/// `ergo-lib` uses a number of collection types that are simple wrappers around `Vec`. We have a
/// generic type for such a collection in `ergo-lib-c-core::collections::Collection`. A limitation
/// of `cbindgen` is that it cannot process generic functions. This macro generates a C-compatible
/// interface for such a collection for any desired type T.
///
/// As an example the call `make_collection(BlockHeaders, BlockHeader);` generates:
///
///```
///pub type BlockHeadersPtr = CollectionPtr<BlockHeader>;
///pub type ConstBlockHeadersPtr = ConstCollectionPtr<BlockHeader>;
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_wallet_block_headers_new(collection_ptr_out: *mut BlockHeadersPtr) -> ErrorPtr {
///    let res = collection_new(collection_ptr_out);
///    Error::c_api_from(res)
///}
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_wallet_block_headers_delete(collection_ptr_out: BlockHeadersPtr) {
///    collection_delete(collection_ptr_out)
///}
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_wallet_block_headers_len(
///    collection_ptr: ConstBlockHeadersPtr,
///) -> ReturnNum<usize> {
///    match collection_len(collection_ptr) {
///        Ok(value) => ReturnNum {
///            value: value as usize,
///            error: std::ptr::null_mut(),
///        },
///        Err(e) => ReturnNum {
///            value: 0, // Just a dummy value
///            error: Error::c_api_from(Err(e)),
///        },
///    }
///}
///
///#[no_mangle]
///pub unsafe extern "C" fn ergo_wallet_block_headers_get(
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
///pub unsafe extern "C" fn ergo_wallet_block_headers_add(
///    element_ptr: ConstBlockHeaderPtr,
///    collection_ptr_out: BlockHeadersPtr,
///) -> ErrorPtr {
///    Error::c_api_from(collection_add(collection_ptr_out, element_ptr))
///}
///```
#[macro_export]
macro_rules! make_collection {
    ($collection_type_name:ident, $item_type_name:ident) => {
        paste! {
            pub type [<$collection_type_name Ptr>] = ergo_lib_c_core::collections::CollectionPtr<$item_type_name>;
            pub type [<Const $collection_type_name Ptr>] = ergo_lib_c_core::collections::ConstCollectionPtr<$item_type_name>;

            /// Create a new empty collection
            #[no_mangle]
            pub unsafe extern "C" fn [<ergo_wallet_ $collection_type_name:snake _new>](
                collection_ptr_out: *mut [<$collection_type_name Ptr>],
            ) -> ErrorPtr {
                let res = ergo_lib_c_core::collections::collection_new(collection_ptr_out);
                Error::c_api_from(res)
            }

            /// Delete an existing collection
            #[no_mangle]
            pub unsafe extern "C" fn [<ergo_wallet_ $collection_type_name:snake _delete>](ptr_out: [<$collection_type_name Ptr>]) {
                ergo_lib_c_core::collections::collection_delete(ptr_out)
            }

            /// Returns length of an existing collection
            #[no_mangle]
            pub unsafe extern "C" fn [<ergo_wallet_ $collection_type_name:snake _len>](
                collection_ptr: [<Const $collection_type_name Ptr>],
            ) -> crate::ReturnNum<usize> {
                match ergo_lib_c_core::collections::collection_len(collection_ptr) {
                    Ok(value) => crate::ReturnNum {
                        value: value as usize,
                        error: std::ptr::null_mut(),
                    },
                    Err(e) => crate::ReturnNum {
                        value: 0, // Just a dummy value
                        error: Error::c_api_from(Err(e)),
                    },
                }
            }

            /// Returns element at position `index` of an existing collection
            #[no_mangle]
            pub unsafe extern "C" fn [<ergo_wallet_ $collection_type_name:snake _get>](
                collection_ptr: [<Const $collection_type_name Ptr>],
                index: usize,
                element_ptr_out: *mut [<$item_type_name Ptr>],
            ) -> crate::ReturnOption {
                match ergo_lib_c_core::collections::collection_get(collection_ptr, index, element_ptr_out) {
                    Ok(is_some) => crate::ReturnOption {
                        is_some,
                        error: std::ptr::null_mut(),
                    },
                    Err(e) => crate::ReturnOption {
                        is_some: false, // Just a dummy value
                        error: Error::c_api_from(Err(e)),
                    },
                }
            }

            #[no_mangle]
            /// Add an element to collection
            pub unsafe extern "C" fn [<ergo_wallet_ $collection_type_name:snake _add>](
                element_ptr: [<Const $item_type_name Ptr>],
                collection_ptr_out: [<$collection_type_name Ptr>],
            ) -> ErrorPtr {
                Error::c_api_from(ergo_lib_c_core::collections::collection_add(collection_ptr_out, element_ptr))
            }

        }
    };
}
