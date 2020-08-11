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
    use crate::{serialization::SigmaSerializable, ErgoTree};
    use serde::{Deserialize, Deserializer, Serializer};

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
}

pub mod transaction {
    use crate::chain::{data_input::DataInput, ErgoBox, Input, TxId};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    pub struct TransactionJson {
        #[cfg_attr(feature = "with-serde", serde(rename = "id"))]
        pub tx_id: TxId,
        /// inputs, that will be spent by this transaction.
        #[cfg_attr(feature = "with-serde", serde(rename = "inputs"))]
        pub inputs: Vec<Input>,
        /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
        /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
        /// included in transaction cost and they do not contain spending proofs.
        #[cfg_attr(feature = "with-serde", serde(rename = "dataInputs"))]
        pub data_inputs: Vec<DataInput>,
        #[cfg_attr(feature = "with-serde", serde(rename = "outputs"))]
        pub outputs: Vec<ErgoBox>,
    }
}

#[cfg(test)]
mod tests {
    use super::super::ergo_box::*;
    use super::super::transaction::*;
    use super::*;
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

        #[test]
        fn tx_roundtrip(t in any::<Transaction>()) {
            let j = serde_json::to_string(&t)?;
            // dbg!(j);
            eprintln!("{}", j);
            let t_parsed: Transaction = serde_json::from_str(&j)?;
            prop_assert_eq![t, t_parsed];
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

    #[test]
    fn parse_ergo_tree_with_constants() {
        let json = r#"
            {"boxId":"dd4e69ae683d7c2d1de2b3174182e6c443fd68abbcc24002ddc99adb599e0193","value":1000000,"ergoTree":"0008cd03f1102eb87a4166bf9fbd6247d087e92e1412b0e819dbb5fbc4e716091ec4e4ec","assets":[],"creationHeight":268539,"additionalRegisters":{},"transactionId":"8204d2bbaabf946f89a27b366d1356eb10241dc1619a70b4e4a4a38b520926ce","index":0}
        "#;
        let b: ergo_box::ErgoBoxFromJson = serde_json::from_str(json).unwrap();
        assert!(b.ergo_tree.proposition().is_ok())
    }
}
