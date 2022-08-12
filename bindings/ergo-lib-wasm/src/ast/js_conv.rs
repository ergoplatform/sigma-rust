// TODO: remove
#![allow(clippy::wildcard_enum_match_arm)]

use std::convert::TryFrom;

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
use js_sys::Number;
use js_sys::Set;
use js_sys::Uint8Array;
use sigma_util::AsVecI8;
use sigma_util::AsVecU8;
use thiserror::Error;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
pub enum ConvError {
    #[error("not supported: {0:?}")]
    NotSupported(JsValue),
    #[error("unexpected: {0:?}")]
    Unexpected(Constant),
    #[error("IO error: {0}")]
    TryExtractFromError(#[from] TryExtractFromError),
}

pub fn constant_from_js(val: &JsValue) -> Result<Constant, ConvError> {
    if let Ok(bytes) = val.clone().dyn_into::<Uint8Array>() {
        Ok(Constant {
            tpe: SType::SColl(SType::SByte.into()),
            v: Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(
                bytes.to_vec().as_vec_i8(),
            ))),
        })
    } else if let Ok(arr) = val.clone().dyn_into::<Array>() {
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
    } else if let Ok(set) = val.clone().dyn_into::<Set>() {
        let mut v: Vec<Constant> = Vec::new();
        for elem in set.keys() {
            #[allow(clippy::unwrap_used)]
            v.push(constant_from_js(&elem.unwrap())?);
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
                TupleItems::try_from(v.into_iter().map(|c| c.v).collect::<Vec<Literal>>()).unwrap(),
            ),
        })
    } else if let Ok(num) = val.clone().dyn_into::<Number>() {
        // TODO: handle error
        #[allow(clippy::unwrap_used)]
        let c: Constant = (num.as_f64().unwrap() as i32).into();
        Ok(c)
    } else {
        Err(ConvError::NotSupported(val.clone()))
    }
}

pub(crate) fn constant_to_js(c: Constant) -> Result<JsValue, ConvError> {
    Ok(match c.tpe {
        SType::SBoolean => c.v.try_extract_into::<bool>()?.into(),
        SType::SInt => c.v.try_extract_into::<i32>()?.into(),
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
            let set = Set::new(&arr);
            set.into()
        }
        _ => return Err(ConvError::Unexpected(c.clone())),
    })
}
