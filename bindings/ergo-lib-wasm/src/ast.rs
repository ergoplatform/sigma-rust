//! Ergo constant values

use crate::error_conversion::conv;
use crate::utils::I64;
use ergo_lib::chain::Base16Str;
use ergo_lib::ergotree_ir::mir::constant::{TryExtractFrom, TryExtractInto};
use ergo_lib::ergotree_ir::serialization::SigmaSerializable;
use ergo_lib::ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use js_sys::Uint8Array;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// Ergo constant(evaluated) values
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct Constant(ergo_lib::ergotree_ir::mir::constant::Constant);

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
        ergo_lib::ergotree_ir::mir::constant::Constant::try_from(bytes)
            .map_err(conv)
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
    pub fn to_i32(&self) -> Result<i32, JsValue> {
        i32::try_extract_from(self.0.clone()).map_err(conv)
    }

    /// Create from i64
    pub fn from_i64(v: &I64) -> Constant {
        Constant(i64::from((*v).clone()).into())
    }

    /// Extract i64 value, returning error if wrong type
    pub fn to_i64(&self) -> Result<I64, JsValue> {
        i64::try_extract_from(self.0.clone())
            .map_err(conv)
            .map(I64::from)
    }

    /// Create from byte array
    pub fn from_byte_array(v: &[u8]) -> Constant {
        Constant(v.to_vec().into())
    }

    /// Extract byte array, returning error if wrong type
    pub fn to_byte_array(&self) -> Result<Uint8Array, JsValue> {
        Vec::<u8>::try_extract_from(self.0.clone())
            .map(|v| Uint8Array::from(v.as_slice()))
            .map_err(conv)
    }

    /// Create `Coll[Long]` from string array
    #[allow(clippy::boxed_local)]
    pub fn from_i64_str_array(arr: Box<[JsValue]>) -> Result<Constant, JsValue> {
        arr.iter()
            .try_fold(vec![], |mut acc, l| {
                let b: i64 = if l.is_string() {
                    let l_str = l
                        .as_string()
                        .ok_or_else(|| JsValue::from_str("i64 as a string"))?;
                    serde_json::from_str(l_str.as_str())
                } else {
                    l.into_serde::<i64>()
                }
                .map_err(|e| {
                    JsValue::from_str(&format!(
                        "Failed to parse i64 from JSON string: {:?} \n with error: {}",
                        l, e
                    ))
                })?;
                acc.push(b);
                Ok(acc)
            })
            .map(|longs| longs.into())
            .map(Constant)
    }

    /// Extract `Coll[Long]` as string array
    #[allow(clippy::boxed_local)]
    pub fn to_i64_str_array(&self) -> Result<Box<[JsValue]>, JsValue> {
        let vec_i64 = self
            .0
            .clone()
            .try_extract_into::<Vec<i64>>()
            .map_err(|e| JsValue::from_str(&format!("Constant has wrong type: {:?}", e)))?;
        Ok(vec_i64
            .iter()
            .map(|it| JsValue::from_str(&it.to_string()))
            .collect())
    }

    /// Parse raw [`EcPoint`] value from bytes and make [`ProveDlog`] constant
    pub fn from_ecpoint_bytes(bytes: &[u8]) -> Result<Constant, JsValue> {
        let ecp = EcPoint::sigma_parse_bytes(bytes).map_err(conv)?;
        let c: ergo_lib::ergotree_ir::mir::constant::Constant = ProveDlog::new(ecp).into();
        Ok(c.into())
    }

    /// Create `(Coll[Byte], Coll[Byte])` tuple Constant
    pub fn from_tuple_coll_bytes(bytes1: &[u8], bytes2: &[u8]) -> Constant {
        let t = (bytes1.to_vec(), bytes2.to_vec());
        let c: ergo_lib::ergotree_ir::mir::constant::Constant = t.into();
        c.into()
    }

    /// Extract `(Coll[Byte], Coll[Byte])` tuple from Constant as array of Uint8Array
    pub fn to_tuple_coll_bytes(&self) -> Result<Vec<Uint8Array>, JsValue> {
        let (bytes1, bytes2) = self
            .0
            .clone()
            .try_extract_into::<(Vec<u8>, Vec<u8>)>()
            .map_err(conv)?;
        Ok(vec![
            Uint8Array::from(bytes1.as_slice()),
            Uint8Array::from(bytes2.as_slice()),
        ])
    }

    /// Create `(Long, Long)` tuple Constant
    pub fn from_tuple_i64(l1: &I64, l2: &I64) -> Constant {
        let c: ergo_lib::ergotree_ir::mir::constant::Constant =
            (i64::from((*l1).clone()), i64::from((*l2).clone())).into();
        c.into()
    }

    /// Extract `(Long, Long)` tuple from Constant as array of strings
    pub fn to_tuple_i64(&self) -> Result<Vec<JsValue>, JsValue> {
        let (l1, l2) = self
            .0
            .clone()
            .try_extract_into::<(i64, i64)>()
            .map_err(conv)?;
        Ok(vec![
            JsValue::from_str(&l1.to_string()),
            JsValue::from_str(&l2.to_string()),
        ])
    }
}
