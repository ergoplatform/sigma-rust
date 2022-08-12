#![allow(clippy::todo)] // TODO: remove

use ergo_lib::ergotree_ir::mir::constant::Constant;
use ergo_lib::ergotree_ir::mir::constant::Literal;
use ergo_lib::ergotree_ir::mir::value::CollKind;
use ergo_lib::ergotree_ir::types::stype::SType;
use js_sys::Array;
use js_sys::Number;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub(crate) enum ConvError {}

#[allow(clippy::unwrap_used)]
pub(crate) fn constant_from_js(val: &JsValue) -> Result<Constant, ConvError> {
    match val.clone().dyn_into::<Array>() {
        Ok(arr) => {
            // TODO: check if it's a tuple
            let mut v: Vec<Literal> = Vec::new();
            for i in 0..arr.length() {
                v.push((arr.get(i).dyn_into::<Number>().unwrap().as_f64().unwrap() as i32).into());
            }
            Ok(Constant {
                tpe: SType::SColl(SType::SInt.into()),
                v: Literal::Coll(CollKind::WrappedColl {
                    elem_tpe: SType::SInt,
                    items: v,
                }),
            })
        }
        Err(_) => todo!(),
    }
}

pub(crate) fn constant_to_js(c: Constant) -> Result<JsValue, ConvError> {
    let arr = Array::new();
    arr.push(&JsValue::from_f64(5.0));
    Ok(arr.into())
}
