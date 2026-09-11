use std::ffi::CStr;

use ergo_lib_c_core::unsignedbigint256::{
    CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, ConstUnsignedBigIntPtr, Num,
    UnsignedBigInt, UnsignedBigIntPtr, UnsignedBigIntRaw,
};

unsafe fn cast_ptr<'a>(ptr: ConstUnsignedBigIntPtr) -> Option<&'a UnsignedBigIntRaw> {
    const { assert!(align_of::<UnsignedBigIntRaw>() == align_of::<UnsignedBigInt>()) };
    let ptr = ptr.cast::<UnsignedBigIntRaw>();
    ptr.as_ref()
}

unsafe fn wrap_arith_op(
    out: UnsignedBigIntPtr,
    res: impl FnOnce() -> Option<UnsignedBigIntRaw>,
) -> u32 {
    match res() {
        Some(r) => {
            out.cast::<UnsignedBigIntRaw>().write(r);
            0
        }
        None => 1,
    }
}

/// Create a new UnsignedBigInt, set to 0
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_new() -> UnsignedBigInt {
    UnsignedBigInt([0; 4])
}

/// Create a new UnsignedBigInt, set to 0
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_from_long(long: u64) -> UnsignedBigInt {
    UnsignedBigIntRaw::from(long).into()
}

#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_from_str_radix(
    cstr: *const i8,
    radix: u32,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || {
        UnsignedBigIntRaw::from_str_radix(CStr::from_ptr(cstr).to_str().ok()?, radix).ok()
    })
}

/// Add a and b, putting result in out
/// Arithmetic is checked, so if the result overflowed 1 will be returned and nothing will be written to out
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_add(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || cast_ptr(a)?.checked_add(cast_ptr(b)?))
}

/// Subtract b from a, putting result in out
/// Arithmetic is checked, so if the result overflowed 1 will be returned and nothing will be written to out
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_sub(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || cast_ptr(a)?.checked_sub(cast_ptr(b)?))
}

/// Compute a * b, putting result in out
/// Arithmetic is checked, so if the result overflowed 1 will be returned and nothing will be written to out
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_mul(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || cast_ptr(a)?.checked_mul(cast_ptr(b)?))
}

/// Divide a by b, putting result in out
/// Returns 1 if b is zero and nothing will be written to out
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_div(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || cast_ptr(a)?.checked_div(cast_ptr(b)?))
}

/// Compute (a + b) mod modulus
/// Returns 1 if modulus == 0
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_mod_add(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
    modulus: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || {
        cast_ptr(a)?.checked_mod_add(*cast_ptr(b)?, *cast_ptr(modulus)?)
    })
}

/// Compute (a - b) mod modulus
/// Returns 1 if modulus == 0
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_mod_sub(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
    modulus: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || {
        cast_ptr(a)?.checked_mod_sub(*cast_ptr(b)?, *cast_ptr(modulus)?)
    })
}

/// Compute (a * b) mod modulus
/// Returns 1 if modulus == 0
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_mod_mul(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
    modulus: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || {
        cast_ptr(a)?.checked_mod_mul(*cast_ptr(b)?, *cast_ptr(modulus)?)
    })
}

/// Compute modular inverse of a
/// Returns 1 if modular inverse does not exist
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_mod_inv(
    a: ConstUnsignedBigIntPtr,
    modulus: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || cast_ptr(a)?.mod_inv(*cast_ptr(modulus)?))
}

/// Compute a mod modulus
/// Returns 1 if modulus == 0
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_mod(
    a: ConstUnsignedBigIntPtr,
    modulus: ConstUnsignedBigIntPtr,
    out: UnsignedBigIntPtr,
) -> u32 {
    wrap_arith_op(out, || cast_ptr(a)?.checked_rem(cast_ptr(modulus)?))
}

/// Returns true if a == b. Both a and b must be non-null pointers
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_eq(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
) -> bool {
    cast_ptr(a).unwrap_unchecked() == cast_ptr(b).unwrap_unchecked()
}

/// Compares a and b, returns -1 if a < b, 0 if a == b and 1 if a > b. Both a and b must be non-null pointers
#[no_mangle]
pub unsafe extern "C" fn ergo_lib_u256_cmp(
    a: ConstUnsignedBigIntPtr,
    b: ConstUnsignedBigIntPtr,
) -> i8 {
    match cast_ptr(a)
        .unwrap_unchecked()
        .cmp(cast_ptr(b).unwrap_unchecked())
    {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use super::{
        ergo_lib_u256_add, ergo_lib_u256_div, ergo_lib_u256_mod_inv, ergo_lib_u256_mod_sub,
        ergo_lib_u256_new,
    };
    use crate::{ergo_lib_u256_cmp, ergo_lib_u256_from_long, ergo_lib_u256_from_str_radix};
    use std::ffi::CString;

    #[test]
    fn division_returns_quotient() {
        // Inputs and output are distinct, initialized, correctly aligned values.
        unsafe {
            let a = ergo_lib_u256_from_long(10);
            let b = ergo_lib_u256_from_long(2);
            let mut out = ergo_lib_u256_from_long(99);
            assert_eq!(ergo_lib_u256_div(&a, &b, &mut out), 0);
            assert_eq!(out.0, [5, 0, 0, 0]);
        }
    }

    #[test]
    fn division_truncates_remainder() {
        // Inputs and output are distinct, initialized, correctly aligned values.
        unsafe {
            let a = ergo_lib_u256_from_long(11);
            let b = ergo_lib_u256_from_long(2);
            let mut out = ergo_lib_u256_from_long(99);
            assert_eq!(ergo_lib_u256_div(&a, &b, &mut out), 0);
            assert_eq!(out.0, [5, 0, 0, 0]);
        }
    }

    #[test]
    fn division_by_zero_preserves_output() {
        // Inputs and output are distinct, initialized, correctly aligned values.
        unsafe {
            let a = ergo_lib_u256_from_long(10);
            let zero = ergo_lib_u256_new();
            let mut out = ergo_lib_u256_from_long(99);
            assert_eq!(ergo_lib_u256_div(&a, &zero, &mut out), 1);
            assert_eq!(out.0, [99, 0, 0, 0]);
        }
    }

    fn assert_mod_sub_zero_preserves_output(a: u64, b: u64) {
        // Inputs and output are distinct, initialized, correctly aligned values.
        unsafe {
            let a = ergo_lib_u256_from_long(a);
            let b = ergo_lib_u256_from_long(b);
            let zero = ergo_lib_u256_new();
            let mut out = ergo_lib_u256_from_long(99);
            assert_eq!(ergo_lib_u256_mod_sub(&a, &b, &zero, &mut out), 1);
            assert_eq!(out.0, [99, 0, 0, 0]);
        }
    }

    #[test]
    fn mod_sub_zero_modulus_greater_operands_preserves_output() {
        assert_mod_sub_zero_preserves_output(5, 3);
    }

    #[test]
    fn mod_sub_zero_modulus_equal_operands_preserves_output() {
        assert_mod_sub_zero_preserves_output(3, 3);
    }

    #[test]
    fn mod_sub_zero_modulus_less_operands_preserves_output() {
        assert_mod_sub_zero_preserves_output(1, 3);
    }

    #[test]
    fn mod_inv_zero_modulus_preserves_output() {
        // Inputs and output are distinct, initialized, correctly aligned values.
        unsafe {
            let a = ergo_lib_u256_from_long(5);
            let zero = ergo_lib_u256_new();
            let mut out = ergo_lib_u256_from_long(99);
            assert_eq!(ergo_lib_u256_mod_inv(&a, &zero, &mut out), 1);
            assert_eq!(out.0, [99, 0, 0, 0]);
        }
    }

    #[test]
    fn test_ops() {
        unsafe {
            let bigint = ergo_lib_u256_new();
            let mut out = ergo_lib_u256_new();
            assert_eq!(ergo_lib_u256_add(&bigint, &bigint, &mut out), 0);
            let buf = CString::new("10").unwrap();
            assert_eq!(
                ergo_lib_u256_from_str_radix(buf.into_raw(), 10, &mut out),
                0
            );
            let ten = ergo_lib_u256_from_long(10);
            assert_eq!(ergo_lib_u256_cmp(&out, &ten), 0);
            assert_eq!(ergo_lib_u256_cmp(&out, &bigint), 1);
            assert_eq!(ergo_lib_u256_cmp(&bigint, &out), -1);
        }
    }
}
