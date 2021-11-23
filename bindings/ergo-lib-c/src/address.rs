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

use crate::{ErrorPtr, ReturnNum};

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_from_testnet(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_testnet(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_from_mainnet(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_mainnet(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_from_base58(
    address_str: *const c_char,
    address_out: *mut AddressPtr,
) -> ErrorPtr {
    let address = CStr::from_ptr(address_str).to_string_lossy();
    let res = address_from_base58(&address, address_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_to_base58(
    address: ConstAddressPtr,
    network_prefix: NetworkPrefix,
    _address_str: *mut *const c_char,
) -> ErrorPtr {
    let res = match address_to_base58(address, network_prefix) {
        Ok(s) => {
            *_address_str = CString::new(s).unwrap().into_raw();
            Ok(())
        }
        Err(e) => Err(e),
    };
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_address_type_prefix(
    address: ConstAddressPtr,
) -> ReturnNum<u8> {
    match address_type_prefix(address) {
        Ok(value) => ReturnNum {
            value: value as u8,
            error: std::ptr::null_mut(),
        },
        Err(e) => ReturnNum {
            value: 0, // Just a dummy value
            error: Error::c_api_from(Err(e)),
        },
    }
}

#[no_mangle]
pub extern "C" fn ergo_wallet_address_delete(address: AddressPtr) {
    address_delete(address)
}