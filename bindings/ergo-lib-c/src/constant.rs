use ergo_lib_c_core::constant::ConstantPtr;

use crate::delete_ptr;

#[no_mangle]
pub extern "C" fn ergo_wallet_constant_delete(ptr: ConstantPtr) {
    unsafe { delete_ptr(ptr) }
}
