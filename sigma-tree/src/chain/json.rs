//! JSON serialization
use super::ErgoBox;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// impl Serialize for BoxId {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let str = base16::encode_lower(&self.0);
//         serializer.serialize_str(&str)
//     }
// }

// impl<'de> Deserialize<'de> for BoxId {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_str(StrVisitor)
//     }
// }

impl Serialize for ErgoBox {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: implement missing fields
        let mut state = s.serialize_struct("box", 8)?;
        state.serialize_field("boxId", &self.box_id())?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("ergoTree", "TBD")?;
        state.serialize_field("assets", "TBD")?;
        state.serialize_field("creationHeight", &self.creation_height)?;
        // state.serialize_field("additionalRegisters", &self.additional_registers)?;
        state.serialize_field("transactionId", "TBD")?;
        state.serialize_field("index", "TBD")?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ErgoBox {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
