//! JSON serialization

mod base16_bytes;

pub use base16_bytes::Base16DecodedBytes;
pub use base16_bytes::Base16EncodedBytes;

use serde::Serializer;

pub fn serialize_bytes<S, T>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_str(&base16::encode_lower(bytes.as_ref()))
}

pub mod ergo_tree {

    use super::*;
    use crate::ErgoTree;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::convert::TryFrom;

    pub fn serialize<S>(ergo_tree: &ErgoTree, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = ergo_tree.bytes();
        serialize_bytes(&bytes[..], serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ErgoTree, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        String::deserialize(deserializer)
            .and_then(|str| base16::decode(&str).map_err(|err| Error::custom(err.to_string())))
            .and_then(|bytes| {
                ErgoTree::try_from(bytes).map_err(|err| Error::custom(err.to_string()))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::ergo_box::*;
    // use super::*;
    use proptest::prelude::*;
    use register::NonMandatoryRegisters;
    use serde_json;

    proptest! {

        #[test]
        fn ergo_box_roundtrip(b in any::<ErgoBox>()) {
            let j = serde_json::to_string(&b)?;
            // eprintln!("{}", j);
            let b_parsed: ErgoBox = serde_json::from_str(&j)?;
            prop_assert_eq![b, b_parsed];
        }

    }

    #[test]
    fn parse_registers() {
        let json = r#"
        {"R4":"05b0b5cad8e6dbaef44a","R5":"048ce5d4e505"}
        "#;
        let regs: NonMandatoryRegisters = serde_json::from_str(json).unwrap();
        assert_eq!(regs.get_ordered_values().len(), 2)
    }
}
