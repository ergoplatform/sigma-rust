//! Ergo constant values

use std::convert::TryFrom;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::utils::I64;

/// Ergo constant(evaluated) values
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant(ergo_lib::ast::Constant);

#[wasm_bindgen]
impl Constant {
    /// Decode from Base16-encoded ErgoTree serialized value
    pub fn decode_from_base16(base16_bytes_str: String) -> Result<Constant, JsValue> {
        let bytes = ergo_lib::chain::Base16DecodedBytes::try_from(base16_bytes_str.clone())
            .map_err(|_| {
                JsValue::from_str(&format!(
                    "failed to decode base16 from: {}",
                    base16_bytes_str.clone()
                ))
            })?;
        ergo_lib::ast::Constant::try_from(bytes)
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))
            .map(Constant)
    }

    /// Encode as Base16-encoded ErgoTree serialized value
    pub fn encode_to_base16(&self) -> String {
        self.0.base16_str()
    }

    /// Create from i32 value
    pub fn from_i32(v: i32) -> Constant {
        Constant(v.into())
    }

    /// Extract i32 value, returning error if wrong type
    pub fn as_i32(&self) -> Result<i32, JsValue> {
        match self.0.v {
            ergo_lib::ast::ConstantVal::Int(v) => Ok(v),
            _ => Err(JsValue::from_str(&format!(
                "expected i32, found: {:?}",
                self.0.v
            ))),
        }
    }

    /// Create from i64
    pub fn from_i64(v: &I64) -> Constant {
        Constant(i64::from((*v).clone()).into())
    }

    /// Extract i64 value, returning error if wrong type
    pub fn as_i64(&self) -> Result<I64, JsValue> {
        match self.0.v {
            ergo_lib::ast::ConstantVal::Long(v) => Ok(v.into()),
            _ => Err(JsValue::from_str(&format!(
                "expected i64, found: {:?}",
                self.0.v
            ))),
        }
    }

    /// Create from byte array
    pub fn from_byte_array(v: &[u8]) -> Constant {
        Constant(v.to_vec().into())
    }

    /// Extract byte array, returning error if wrong type
    pub fn as_byte_array(&self) -> Result<Uint8Array, JsValue> {
        match self.0.v.clone() {
            ergo_lib::ast::ConstantVal::Coll(ergo_lib::ast::ConstantColl::Primitive(
                ergo_lib::ast::CollPrim::CollByte(coll_bytes),
            )) => {
                let u8_bytes: Vec<u8> = coll_bytes.into_iter().map(|b| b as u8).collect();
                Ok(Uint8Array::from(u8_bytes.as_slice()))
            }
            _ => Err(JsValue::from_str(&format!(
                "expected byte array, found: {:?}",
                self.0.v
            ))),
        }
    }
}
