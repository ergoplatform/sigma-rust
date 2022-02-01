use ergo_lib_c_core::{
    address::{
        address_delete, address_from_base58, address_from_mainnet, address_from_testnet,
        address_to_base58, address_type_prefix, AddressPtr, ConstAddressPtr, NetworkPrefix,
    },
    Error,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::ErrorPtr;

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_address_from_testnet(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_testnet(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_address_from_mainnet(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_mainnet(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_address_from_base58(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_base58(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_address_to_base58(
    address: ConstAddressPtr,
    network_prefix: NetworkPrefix,
    _address_str: *mut *const c_char,
) {
    #[allow(clippy::unwrap_used)]
    {
        let s = address_to_base58(address, network_prefix).unwrap();
        *_address_str = CString::new(s).unwrap().into_raw();
    }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_address_type_prefix(address: ConstAddressPtr) -> u8 {
    #[allow(clippy::unwrap_used)]
    (address_type_prefix(address).unwrap() as u8)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_address_delete(address: AddressPtr) {
    address_delete(address)
}
