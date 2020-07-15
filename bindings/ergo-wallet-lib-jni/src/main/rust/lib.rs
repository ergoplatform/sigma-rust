use ergo_wallet_lib_c_core::{address_from_testnet, AddressPtr};
use jni::{
    objects::{JClass, JString},
    sys::{jboolean, jbyteArray, jint, jlong, jobjectArray, jstring, JNI_FALSE, JNI_TRUE},
    JNIEnv,
};
use std::ptr::null_mut;

#[no_mangle]
pub unsafe extern "system" fn Java_org_ergoplatform_wallet_jni_WalletLib_addressFromTestNet(
    env: JNIEnv,
    _: JClass,
    address_str: JString,
) -> jlong {
    let address_str_j = env
        .get_string(address_str)
        .expect("Couldn't get address String");

    let mut address: AddressPtr = null_mut();
    let result = address_from_testnet(&address_str_j.to_string_lossy(), &mut address);

    if let Some(error) = result.error() {
        let _ = env.throw(error.to_string());
        0
    } else {
        address as jlong
    }
}

