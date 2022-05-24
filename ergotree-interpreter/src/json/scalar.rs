use std::convert::TryFrom;

use ergotree_ir::chain::json::serialize_bytes;
use k256::Scalar;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;

use crate::sigma_protocol::GroupSizedBytes;

pub fn serialize<S>(scalar: &Scalar, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bytes = scalar.to_bytes();
    serialize_bytes(bytes.as_slice(), serializer)
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Scalar, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|str| base16::decode(&str).map_err(|err| Error::custom(err.to_string())))
        .and_then(|bytes| {
            Ok(Scalar::from(
                GroupSizedBytes::try_from(bytes).map_err(|err| Error::custom(err.to_string()))?,
            ))
        })
}
