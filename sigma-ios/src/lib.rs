use std::os::raw::{c_int, c_longlong};

#[no_mangle]
pub extern "C" fn add_numbers(x: c_int, y: c_int) -> c_longlong {
    x as i64 + y as i64
}
