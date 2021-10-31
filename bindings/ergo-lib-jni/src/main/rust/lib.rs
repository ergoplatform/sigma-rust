//! JNI bindings fo ergo-lib

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
// #![deny(missing_docs)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
extern crate log;

mod exception;

use ergo_lib_c_core::address::{address_delete, address_from_testnet, AddressPtr};
use exception::unwrap_exc_or;
use jni::{
    objects::{JClass, JString},
    sys::jlong,
    JNIEnv,
};
use std::{panic, ptr::null_mut};

#[no_mangle]
pub unsafe extern "system" fn Java_org_ergoplatform_wallet_jni_WalletLib_addressFromTestNet(
    env: JNIEnv,
    _: JClass,
    address_str: JString,
) -> jlong {
    let res = panic::catch_unwind(|| {
        let address_str_j = env
            .get_string(address_str)
            .expect("Couldn't get address String");

        let mut address: AddressPtr = null_mut();
        let result = address_from_testnet(&address_str_j.to_string_lossy(), &mut address);

        if let Some(error) = result.err() {
            let _ = env.throw(error.to_string());
            Ok(0)
        } else {
            Ok(address as jlong)
        }
    });
    unwrap_exc_or(&env, res, 0)
}

#[no_mangle]
pub extern "system" fn Java_org_ergoplatform_wallet_jni_WalletLib_addressDelete(
    _: JNIEnv,
    _: JClass,
    address: jlong,
) {
    let address_ptr: AddressPtr = address as AddressPtr;
    if !address_ptr.is_null() {
        address_delete(address_ptr);
    }
}
