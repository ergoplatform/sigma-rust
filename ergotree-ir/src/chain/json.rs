//! JSON serialization

use serde::Serializer;

pub(crate) mod box_value;
pub(crate) mod ergo_box;
pub mod ergo_tree;
pub(crate) mod token;

/// Serialize bytes ([u8]) as base16 encoded string
pub fn serialize_bytes<S, T>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_str(&base16::encode_lower(bytes.as_ref()))
}
