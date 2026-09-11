//! Unsigned 256-bit integer type
use derive_more::From;
use ergo_lib::ergotree_ir::unsignedbigint256;
use num_traits::Num;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub};
use wasm_bindgen::prelude::*;

/// Unsigned 256-bit integer type
#[derive(From)]
#[wasm_bindgen]
pub struct UnsignedBigInt(pub(crate) unsignedbigint256::UnsignedBigInt);

fn wrap_arith_op(
    res: Option<unsignedbigint256::UnsignedBigInt>,
) -> Result<UnsignedBigInt, JsError> {
    res.map(UnsignedBigInt)
        .ok_or_else(|| JsError::new("unsignedbigint overflow"))
}

#[wasm_bindgen]
impl UnsignedBigInt {
    /// Create a new UnsignedBigInt from a nonnegative safe integer Number or JS BigInt
    #[wasm_bindgen(constructor)]
    pub fn new(number: wasm_bindgen::JsValue) -> Result<Self, JsError> {
        match number.dyn_into::<js_sys::BigInt>() {
            Ok(bigint) => {
                #[allow(clippy::unwrap_used)]
                // safe, because it can only return an error on invalid radix
                let bigint_js_str: String = bigint.to_string(10).unwrap().into();
                UnsignedBigInt::from_str_radix(bigint_js_str.as_str(), 10)
                    .map_err(|_| JsError::new("failed to convert to UnsignedBigInt"))
            }
            Err(number) => {
                let num = number
                    .as_f64()
                    .ok_or_else(|| JsError::new("UnsignedBigInt.new: expected numeric type"))?;
                if !num.is_finite()
                    || !(0.0..=9_007_199_254_740_991.0).contains(&num)
                    || num.fract() != 0.0
                {
                    return Err(JsError::new(
                        "UnsignedBigInt.new: expected nonnegative safe integer Number; use BigInt for larger integers",
                    ));
                }
                Ok(Self(unsignedbigint256::UnsignedBigInt::from(num as u64)))
            }
        }
    }

    /// Compare two UnsignedBigInts
    #[allow(clippy::should_implement_trait)] // wasm_bindgen doesn't seem to allow you to implement === operator in JS on rust structs, so need to use this method instead
    pub fn eq(&self, other: &UnsignedBigInt) -> bool {
        self.0 == other.0
    }
    /// Create UnsignedBigInt from str with given base
    pub fn from_str_radix(s: &str, radix: u32) -> Result<Self, JsError> {
        unsignedbigint256::UnsignedBigInt::from_str_radix(s, radix)
            .map_err(|e| JsError::new(&format!("{e}")))
            .map(Self)
    }
    /// Add two UnsignedBigInts. If the result overflows an exception will be raised
    pub fn add(&self, other: &UnsignedBigInt) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_add(&other.0))
    }
    /// Subtract other from self. If the result overflows an exception will be raised
    pub fn sub(&self, other: &UnsignedBigInt) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_sub(&other.0))
    }
    /// Multiply self by other. If the result overflows an exception will be raised
    pub fn mul(&self, other: &UnsignedBigInt) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_mul(&other.0))
    }

    /// Divide self by other. Returns an exception if other == 0
    pub fn div(&self, other: &UnsignedBigInt) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_div(&other.0))
    }
    /// Compute (self + other) mod modulus. Returns an exception if modulus == 0
    pub fn mod_add(
        &self,
        other: &UnsignedBigInt,
        modulus: &UnsignedBigInt,
    ) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_mod_add(other.0, modulus.0))
    }
    /// Compute (self - other) mod modulus. Returns an exception if modulus == 0
    pub fn mod_sub(
        &self,
        other: &UnsignedBigInt,
        modulus: &UnsignedBigInt,
    ) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_mod_sub(other.0, modulus.0))
    }

    /// Compute (self * other) mod modulus. Returns an exception if modulus == 0
    pub fn mod_mul(
        &self,
        other: &UnsignedBigInt,
        modulus: &UnsignedBigInt,
    ) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_mod_mul(other.0, modulus.0))
    }

    /// Compute (self mod modulus)
    pub fn rem(&self, modulus: &UnsignedBigInt) -> Result<Self, JsError> {
        wrap_arith_op(self.0.checked_rem(&modulus.0))
    }

    /// Compute modular inverse of self. Returns an exception if modulus == 0 or modular inverse does not exist
    pub fn mod_inv(&self, modulus: &UnsignedBigInt) -> Result<Self, JsError> {
        wrap_arith_op(self.0.mod_inv(modulus.0))
    }
}
