use wasm_bindgen::prelude::*;

/// comment
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i64 {
    a as i64 + b as i64
}
