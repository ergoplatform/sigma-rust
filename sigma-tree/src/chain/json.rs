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
    use sigma_ser::serializer::SigmaSerializable;

    pub fn serialize<S>(ergo_tree: &ErgoTree, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = ergo_tree.sigma_serialise_bytes();
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
                ErgoTree::sigma_parse_bytes(bytes).map_err(|error| Error::custom(error.to_string()))
            })
    }
}

pub mod ergo_box {
    use crate::{
        chain::{box_value::BoxValue, register::NonMandatoryRegisters, BoxId, TokenAmount, TxId},
        ErgoTree,
    };
    use serde::Deserialize;
    use thiserror::Error;

    #[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
    pub struct ErgoBoxFromJson {
        #[serde(rename = "boxId")]
        pub box_id: BoxId,
        /// amount of money associated with the box
        #[serde(rename = "value")]
        pub value: BoxValue,
        /// guarding script, which should be evaluated to true in order to open this box
        #[serde(rename = "ergoTree", with = "super::ergo_tree")]
        pub ergo_tree: ErgoTree,
        /// secondary tokens the box contains
        #[serde(rename = "assets")]
        pub tokens: Vec<TokenAmount>,
        ///  additional registers the box can carry over
        #[serde(rename = "additionalRegisters")]
        pub additional_registers: NonMandatoryRegisters,
        /// height when a transaction containing the box was created.
        /// This height is declared by user and should not exceed height of the block,
        /// containing the transaction with this box.
        #[serde(rename = "creationHeight")]
        pub creation_height: u32,
        /// id of transaction which created the box
        #[serde(rename = "transactionId")]
        pub transaction_id: TxId,
        /// number of box (from 0 to total number of boxes the transaction with transactionId created - 1)
        #[serde(rename = "index")]
        pub index: u16,
    }

    #[derive(Error, PartialEq, Eq, Debug, Clone)]
    pub enum ErgoBoxFromJsonError {
        #[error("Box id parsed from JSON differs from calculated from box serialized bytes")]
        InvalidBoxId,
    }
}

#[cfg(test)]
mod tests {
    use super::super::ergo_box::*;
    // use super::*;
    use proptest::prelude::*;
    use register::NonMandatoryRegisters;

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
