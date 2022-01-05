//! Code to implement `BoxValue` JSON encoding

use std::convert::TryFrom;

use crate::chain::ergo_box::box_value::BoxValue;

/// Helper struct to serialize/deserialize `BoxValue`.
///
/// We use `serde_json::Number` below due to a known `serde_json` bug described here:
/// <https://github.com/serde-rs/json/issues/740>. Basically we can't deserialise any integer types
/// directly within untagged enums when the `arbitrary_precision` feature is used. The workaround is
/// to deserialize as `serde_json::Number` first, then manually convert the type.
#[cfg(feature = "json")]
#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct BoxValueJson(
    // Tries to decode as u64 first, then fallback to string. Encodes as u64 always
    // see details - https://docs.rs/serde_with/1.9.4/serde_with/struct.PickFirst.html
    #[serde_as(as = "serde_with::PickFirst<(_, serde_with::DisplayFromStr)>")]
    pub(crate)  serde_json::Number,
);

impl TryFrom<BoxValueJson> for BoxValue {
    type Error = String;

    fn try_from(value: BoxValueJson) -> Result<Self, Self::Error> {
        if let Some(n) = value.0.as_u64() {
            Ok(BoxValue(n))
        } else {
            Err(String::from("can't convert `BoxValueJson` into `BoxValue`"))
        }
    }
}

impl From<BoxValue> for BoxValueJson {
    fn from(value: BoxValue) -> Self {
        BoxValueJson(serde_json::Number::from(value.0))
    }
}
