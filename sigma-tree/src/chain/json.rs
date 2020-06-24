//! JSON serialization

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

pub mod register {
    use serde::ser::{SerializeMap, Serializer};
    use serde::{Deserialize, Deserializer};
    // use sigma_ser::serializer::SerializationError;
    use crate::chain::register::{NonMandatoryRegisterId, NonMandatoryRegisters};
    use sigma_ser::serializer::SigmaSerializable;
    use std::collections::HashMap;
    // use sigma_ser::vlq_encode;

    pub fn serialize<S>(registers: &NonMandatoryRegisters, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(registers.len()))?;
        for (reg_id, constant) in registers.get_ordered_pairs().into_iter() {
            let constant_bytes = constant.sigma_serialise_bytes();
            let encoded = base16::encode_lower(&constant_bytes[..]);
            map.serialize_entry(&reg_id, &encoded)?;
        }
        map.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NonMandatoryRegisters, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mymap: Result<HashMap<NonMandatoryRegisterId, String>, D::Error> =
            HashMap::deserialize(deserializer);
        // mymap.and_then(|regs| regs.in)
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ergo_box::*;
    // use super::*;
    use proptest::prelude::*;
    use serde_json;

    proptest! {

        #[test]
        #[ignore]
        fn ergo_box_roundtrip(b in any::<ErgoBox>()) {
            let j = serde_json::to_string(&b)?;
            let b_parsed: ErgoBox = serde_json::from_str(&j)?;
            prop_assert_eq![b, b_parsed];
        }
    }
}
