use ergo_lib_c_core::constant::*;

use crate::delete_ptr;

#[no_mangle]
pub extern "C" fn ergo_wallet_constant_delete(ptr: ConstantPtr) {
    unsafe { delete_ptr(ptr) }
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_from_i32(constant_out: *mut ConstantPtr, value: i32) {
    #[allow(clippy::unwrap_used)]
    constant_from_i32(constant_out, value).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn ergo_wallet_constant_eq(
    constant_ptr_0: ConstConstantPtr,
    constant_ptr_1: ConstConstantPtr,
) -> bool {
    #[allow(clippy::unwrap_used)]
    constant_eq(constant_ptr_0, constant_ptr_1).unwrap()
}
