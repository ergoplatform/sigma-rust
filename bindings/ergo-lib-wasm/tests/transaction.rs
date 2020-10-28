//! Test suite for the Web and headless browsers.

extern crate wasm_bindgen_test;

use ergo_lib_wasm::secret_key::SecretKey;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_random() {
    let sk1 = SecretKey::random_dlog();
    let sk2 = SecretKey::random_dlog();
    assert_ne!(sk1, sk2);
}
