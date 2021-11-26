//! Contract, for easier ErgoTree generation

use ergo_lib_c_core::{
    address::ConstAddressPtr,
    contract::*,
    ergo_tree::{ConstErgoTreePtr, ErgoTreePtr},
    Error, ErrorPtr,
};

use std::{ffi::CStr, os::raw::c_char};

use crate::delete_ptr;

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_contract_new(
    ergo_tree_ptr: ConstErgoTreePtr,
    contract_out: *mut ContractPtr,
) {
    #[allow(clippy::unwrap_used)]
    contract_new(ergo_tree_ptr, contract_out).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_contract_pay_to_address(
    address_ptr: ConstAddressPtr,
    contract_out: *mut ContractPtr,
) -> ErrorPtr {
    let res = contract_pay_to_address(address_ptr, contract_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_contract_compile(
    source: *const c_char,
    contract_out: *mut ContractPtr,
) -> ErrorPtr {
    let source = CStr::from_ptr(source).to_string_lossy();
    let res = contract_compile(&source, contract_out);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_contract_ergo_tree(
    contract_ptr: ConstContractPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) {
    #[allow(clippy::unwrap_used)]
    contract_ergo_tree(contract_ptr, ergo_tree_out).unwrap();
}

#[no_mangle]
pub extern "C" fn ergo_wallet_contract_delete(ptr: ContractPtr) {
    unsafe { delete_ptr(ptr) }
}
