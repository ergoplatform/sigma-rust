use ergo_lib_c_core::{constant::*, Error, ErrorPtr};

use crate::delete_ptr;

#[no_mangle]
pub extern "C" fn ergo_wallet_constant_delete(ptr: ConstantPtr) {
    unsafe { delete_ptr(ptr) }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_from_i32(
    constant_out: *mut ConstantPtr,
    value: i32,
) -> ErrorPtr {
    let res = constant_from_i32(constant_out, value);
    Error::c_api_from(res)
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_eq(
    constant_ptr_0: ConstConstantPtr,
    constant_ptr_1: ConstConstantPtr,
) -> bool {
    constant_eq(constant_ptr_0, constant_ptr_1).unwrap()
}
