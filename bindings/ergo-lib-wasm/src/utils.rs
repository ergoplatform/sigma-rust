//! Utilities

use wasm_bindgen::prelude::*;

/// Wrapper for i64 for JS/TS
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct I64(i64);

#[wasm_bindgen]
impl I64 {
    /// Create from a standard rust string representation
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(string: &str) -> Result<I64, JsValue> {
        string
            .parse::<i64>()
            .map_err(|e| JsValue::from_str(&format! {"{:?}", e}))
            .map(I64)
    }

    /// String representation of the value for use from environments that don't support i64
    pub fn to_str(&self) -> String {
        format!("{}", self.0)
    }

    /// Get the value as JS number (64-bit float)
    pub fn as_num(&self) -> js_sys::Number {
        js_sys::Number::from(self.0 as f64)
    }
}

impl From<i64> for I64 {
    fn from(v: i64) -> Self {
        I64(v)
    }
}

impl From<I64> for i64 {
    fn from(v: I64) -> Self {
        v.0
    }
}

#[allow(dead_code, missing_docs)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
