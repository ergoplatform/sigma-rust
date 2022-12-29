//! Code to implement `AutolykosSolution` JSON encoding

use std::str::FromStr;

use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde::{Deserialize, Deserializer};

pub(crate) fn as_base16_string<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&base16::encode_lower(value))
}

pub(crate) fn from_base16_string<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| base16::decode(&string).map_err(|err| Error::custom(err.to_string())))
}

/// Serialize `BigInt` as a string
pub(crate) fn bigint_as_str<S>(value: &Option<BigInt>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(value) = value {
        serializer.serialize_str(&value.to_string())
    } else {
        serializer.serialize_unit()
    }
}

/// Deserialize a `BigInt` instance from either a String or from a `serde_json::Number` value.  We
/// need to do this because the JSON specification allows for arbitrarily-large numbers, a feature
/// that Autolykos makes use of to encode the PoW-distance (d) parameter. Note that we also need to
/// use `serde_json` with the `arbitrary_precision` feature for this to work.
pub(crate) fn bigint_from_serde_json_number<'de, D>(
    deserializer: D,
) -> Result<Option<BigInt>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    match DeserializeBigIntFrom::deserialize(deserializer) {
        Ok(s) => match s {
            DeserializeBigIntFrom::String(s) => BigInt::from_str(&s)
                .map(Some)
                .map_err(|e| Error::custom(e.to_string())),
            DeserializeBigIntFrom::SerdeJsonNumber(n) => {
                let bigint = if n.is_f64() {
                    let n_f64 = n
                        .as_f64()
                        .ok_or_else(|| Error::custom("failed to convert JSON number to f64"))?;

                    BigInt::from_f64(n_f64).ok_or_else(|| {
                        Error::custom("failed to create BigInt from f64".to_string())
                    })
                } else {
                    BigInt::from_str(&n.to_string()).map_err(|e| Error::custom(e.to_string()))
                };

                bigint.map(Some)
            }
        },
        Err(e) => Err(Error::custom(e.to_string())),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DeserializeBigIntFrom {
    String(String),
    SerdeJsonNumber(serde_json::Number),
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use serde::de::IntoDeserializer;
    use serde_json::Value;

    use super::bigint_from_serde_json_number;

    #[test]
    fn test_scientific_notion_deser() {
        let pow_d_parameter = r#"4.69094608138843e64"#;
        let j: Value = serde_json::from_str(pow_d_parameter).unwrap();
        let result = bigint_from_serde_json_number(j.into_deserializer()).unwrap();

        assert_eq!(
            result.unwrap().to_string(),
            "46909460813884301641411510982628556119846083366464832536248844288"
        )
    }
}
