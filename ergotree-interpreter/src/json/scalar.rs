use k256::Scalar;
use serde::Deserializer;
use serde::Serializer;

/// Serializer (used in Wasm bindings)
pub fn serialize<S>(ergo_tree: &Scalar, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    todo!()
    // use serde::ser::Error;
    // let bytes = ergo_tree
    //     .sigma_serialize_bytes()
    //     .map_err(|err| Error::custom(err.to_string()))?;
    // serialize_bytes(&bytes[..], serializer)
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Scalar, D::Error>
where
    D: Deserializer<'de>,
{
    todo!()
    // use serde::de::Error;
    // String::deserialize(deserializer)
    //     .and_then(|str| base16::decode(&str).map_err(|err| Error::custom(err.to_string())))
    //     .and_then(|bytes| {
    //         ErgoTree::sigma_parse_bytes(&bytes).map_err(|error| Error::custom(error.to_string()))
    //     })
}
