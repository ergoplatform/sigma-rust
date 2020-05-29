//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate sigma_tree_wasm;
extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_add() {
    assert_eq!(sigma_tree_wasm::add(1, 1), 2);
}
