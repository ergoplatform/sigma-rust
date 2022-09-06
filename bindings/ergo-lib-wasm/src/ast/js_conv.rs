//! Arbitrary JS type to Ergo data type conversion.

#![allow(clippy::wildcard_enum_match_arm)]

use std::convert::TryFrom;

use ergo_lib::ergotree_ir::bigint256::BigInt256;
use ergo_lib::ergotree_ir::mir::constant::Constant;
use ergo_lib::ergotree_ir::mir::constant::Literal;
use ergo_lib::ergotree_ir::mir::constant::TryExtractFromError;
use ergo_lib::ergotree_ir::mir::constant::TryExtractInto;
use ergo_lib::ergotree_ir::mir::value::CollKind;
use ergo_lib::ergotree_ir::mir::value::NativeColl;
use ergo_lib::ergotree_ir::types::stuple::STuple;
use ergo_lib::ergotree_ir::types::stuple::TupleItems;
use ergo_lib::ergotree_ir::types::stype::SType;
use js_sys::Array;
use js_sys::JsString;
use js_sys::Number;
use js_sys::Uint8Array;
use num_traits::Num;
use sigma_util::AsVecI8;
use sigma_util::AsVecU8;
use thiserror::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

const TUPLE_TOKEN: &str = "Tuple";

/// Encode a JS array as an Ergo tuple.
#[wasm_bindgen]
pub fn array_as_tuple(items: Vec<JsValue>) -> JsValue {
    let arr = Array::new();
    arr.push(&JsValue::from_str(TUPLE_TOKEN));
    for item in items {
        arr.push(&item);
    }
    arr.into()
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ConvError {
    #[error("not supported: {0:?}")]
    NotSupported(JsValue),
    #[error("unexpected: {0:?}")]
    Unexpected(Constant),
    #[error("IO error: {0}")]
    TryExtractFromError(#[from] TryExtractFromError),
    #[error("Failed to parse Long from string: {0}")]
    FailedToParseLongFromString(String),
    #[error("Failed to convert JS BigInt to Ergo BigInt: {0}")]
    FailedToConvertJsBigInt(js_sys::BigInt),
    #[error(
        "Invalid tuple encoding, expected the first array item to be '{}''",
        TUPLE_TOKEN
    )]
    InvalidTupleEncoding,
}

pub(crate) fn constant_from_js(val: &JsValue) -> Result<Constant, ConvError> {
    if let Ok(bytes) = val.clone().dyn_into::<Uint8Array>() {
        Ok(Constant {
            tpe: SType::SColl(SType::SByte.into()),
            v: Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(
                bytes.to_vec().as_vec_i8(),
            ))),
        })
    } else if let Ok(arr) = val.clone().dyn_into::<Array>() {
        if let Ok(str) = arr.get(0).dyn_into::<JsString>() {
            // tuple
            if str != TUPLE_TOKEN {
                return Err(ConvError::InvalidTupleEncoding);
            }
            let mut v: Vec<Constant> = Vec::new();
            for i in 1..arr.length() {
                let elem_const = constant_from_js(&arr.get(i))?;
                v.push(elem_const);
            }
            #[allow(clippy::unwrap_used)]
            Ok(Constant {
                tpe: SType::STuple(STuple {
                    items: TupleItems::try_from(
                        v.clone().into_iter().map(|c| c.tpe).collect::<Vec<SType>>(),
                    )
                    .unwrap(),
                }),
                v: Literal::Tup(
                    TupleItems::try_from(v.into_iter().map(|c| c.v).collect::<Vec<Literal>>())
                        .unwrap(),
                ),
            })
        } else {
            // regular array
            let mut cs: Vec<Constant> = Vec::new();
            for i in 0..arr.length() {
                let elem_const = constant_from_js(&arr.get(i))?;
                cs.push(elem_const);
            }
            let elem_tpe = cs[0].tpe.clone();
            Ok(Constant {
                tpe: SType::SColl(elem_tpe.clone().into()),
                v: Literal::Coll(CollKind::WrappedColl {
                    elem_tpe,
                    items: cs.into_iter().map(|c| c.v).collect(),
                }),
            })
        }
    } else if let Ok(num) = val.clone().dyn_into::<Number>() {
        let c: Constant = (num.value_of() as i32).into();
        Ok(c)
    } else if let Ok(long_js_str) = val.clone().dyn_into::<JsString>() {
        let long_str: String = long_js_str.into();
        let long: i64 = long_str
            .parse::<i64>()
            .map_err(|_| ConvError::FailedToParseLongFromString(long_str))?;
        let c: Constant = long.into();
        Ok(c)
    } else if let Ok(bigint_js) = val.clone().dyn_into::<js_sys::BigInt>() {
        let c: Constant = js_bigint_to_ergo_bigint(bigint_js)?.into();
        Ok(c)
    } else {
        Err(ConvError::NotSupported(val.clone()))
    }
}

pub(crate) fn constant_to_js(c: Constant) -> Result<JsValue, ConvError> {
    Ok(match c.tpe {
        SType::SBoolean => c.v.try_extract_into::<bool>()?.into(),
        SType::SShort => c.v.try_extract_into::<i16>()?.into(),
        SType::SByte => c.v.try_extract_into::<i8>()?.into(),
        SType::SInt => c.v.try_extract_into::<i32>()?.into(),
        SType::SLong => c.v.try_extract_into::<i64>()?.to_string().into(),
        SType::SBigInt => ergo_bigint_to_js_bigint(c.v.try_extract_into::<BigInt256>()?).into(),
        SType::SColl(_) => match c.v {
            Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(v))) => {
                Uint8Array::from(v.as_vec_u8().as_slice()).into()
            }
            Literal::Coll(CollKind::WrappedColl { elem_tpe, items }) => {
                let arr = Array::new();
                for item in items {
                    arr.push(&constant_to_js(Constant {
                        tpe: elem_tpe.clone(),
                        v: item,
                    })?);
                }
                arr.into()
            }
            _ => return Err(ConvError::Unexpected(c)),
        },
        SType::STuple(ref item_tpes) => {
            let vec: Vec<JsValue> = match c.v {
                Literal::Tup(v) => v
                    .into_iter()
                    .zip(item_tpes.clone().items.into_iter())
                    .map(|(v, tpe)| constant_to_js(Constant { tpe, v }))
                    .collect::<Result<Vec<JsValue>, _>>()?,
                _ => return Err(ConvError::Unexpected(c.clone())),
            };
            let arr = Array::new();
            for item in vec {
                arr.push(&item);
            }
            arr.into()
        }
        _ => return Err(ConvError::Unexpected(c.clone())),
    })
}

fn ergo_bigint_to_js_bigint(bigint: BigInt256) -> js_sys::BigInt {
    let bigint_str = bigint.to_string();
    #[allow(clippy::unwrap_used)]
    // since BigInt256 bounds are less the JS BigInt it should not fail
    js_sys::BigInt::new(&bigint_str.into()).unwrap()
}

fn js_bigint_to_ergo_bigint(bigint: js_sys::BigInt) -> Result<BigInt256, ConvError> {
    #[allow(clippy::unwrap_used)]
    // safe, because it can only return an error on invalid radix
    let bigint_js_str: String = bigint.to_string(10).unwrap().into();
    BigInt256::from_str_radix(bigint_js_str.as_str(), 10)
        .map_err(|_| ConvError::FailedToConvertJsBigInt(bigint))
}
