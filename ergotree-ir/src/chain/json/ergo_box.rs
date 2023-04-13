use crate::chain::ergo_box::box_value::BoxValue;
use crate::chain::ergo_box::BoxId;
use crate::chain::ergo_box::BoxTokens;
use crate::chain::ergo_box::ErgoBox;
use crate::chain::ergo_box::ErgoBoxCandidate;
use crate::chain::ergo_box::NonMandatoryRegisters;
use crate::chain::ergo_box::RegisterValue;
use crate::chain::token::Token;
use crate::chain::tx_id::TxId;
use crate::ergo_tree::ErgoTree;
use crate::mir::constant::Constant;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializationError;
use ergo_chain_types::Base16DecodedBytes;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::str::FromStr;

extern crate derive_more;
use derive_more::From;

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod box_value;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxJson {
    #[serde(rename = "boxId", alias = "id")]
    pub box_id: Option<BoxId>,
    /// amount of money associated with the box
    #[serde(rename = "value")]
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    #[serde(rename = "ergoTree", with = "super::ergo_tree")]
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    #[serde(rename = "assets")]
    pub tokens: Vec<Token>,
    ///  additional registers the box can carry over
    #[serde(rename = "additionalRegisters")]
    pub additional_registers: NonMandatoryRegisters,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    #[serde(rename = "creationHeight")]
    pub creation_height: u32,
    /// id of transaction which created the box
    #[serde(rename = "transactionId", alias = "txId")]
    pub transaction_id: TxId,
    /// number of box (from 0 to total number of boxes the transaction with transactionId created - 1)
    #[serde(rename = "index")]
    pub index: u16,
}

impl TryFrom<ErgoBoxJson> for ErgoBox {
    type Error = ErgoBoxFromJsonError;
    fn try_from(box_json: ErgoBoxJson) -> Result<Self, Self::Error> {
        let tokens = if box_json.tokens.is_empty() {
            None
        } else {
            Some(box_json.tokens.try_into().map_err(|_| {
                SigmaSerializationError::NotSupported(
                    "More than ErgoBox::MAX_TOKENS_COUNT tokens are not allowed in a box",
                )
            })?)
        };
        let box_with_zero_id = ErgoBox {
            box_id: BoxId::zero(),
            value: box_json.value,
            ergo_tree: box_json.ergo_tree,
            tokens,
            additional_registers: box_json.additional_registers,
            creation_height: box_json.creation_height,
            transaction_id: box_json.transaction_id,
            index: box_json.index,
        };
        let box_id = box_with_zero_id.calc_box_id()?;
        let ergo_box = ErgoBox {
            box_id,
            ..box_with_zero_id
        };
        match box_json.box_id {
            Some(box_id) => {
                if ergo_box.box_id() == box_id {
                    Ok(ergo_box)
                } else {
                    dbg!(&ergo_box);
                    Err(ErgoBoxFromJsonError::InvalidBoxId {
                        json: box_id,
                        actual: ergo_box.box_id(),
                    })
                }
            }
            None => Ok(ergo_box),
        }
    }
}

impl From<ErgoBox> for ErgoBoxJson {
    fn from(ergo_box: ErgoBox) -> ErgoBoxJson {
        let tokens = ergo_box
            .tokens
            .as_ref()
            .map(BoxTokens::as_vec)
            .cloned()
            .unwrap_or_default(); // JSON serialization for assets requires that tokens be [] instead of null
        ErgoBoxJson {
            box_id: Some(ergo_box.box_id),
            value: ergo_box.value,
            ergo_tree: ergo_box.ergo_tree,
            tokens,
            additional_registers: ergo_box.additional_registers,
            creation_height: ergo_box.creation_height,
            transaction_id: ergo_box.transaction_id,
            index: ergo_box.index,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ErgoBoxCandidateJson {
    /// amount of money associated with the box
    #[serde(rename = "value")]
    pub value: BoxValue,
    /// guarding script, which should be evaluated to true in order to open this box
    #[serde(rename = "ergoTree", with = "super::ergo_tree")]
    pub ergo_tree: ErgoTree,
    /// secondary tokens the box contains
    #[serde(rename = "assets")]
    pub tokens: Vec<Token>,
    ///  additional registers the box can carry over
    #[serde(rename = "additionalRegisters")]
    pub additional_registers: NonMandatoryRegisters,
    /// height when a transaction containing the box was created.
    /// This height is declared by user and should not exceed height of the block,
    /// containing the transaction with this box.
    #[serde(rename = "creationHeight")]
    pub creation_height: u32,
}

impl From<ErgoBoxCandidate> for ErgoBoxCandidateJson {
    fn from(ergo_box_candidate: ErgoBoxCandidate) -> Self {
        let tokens = ergo_box_candidate
            .tokens
            .as_ref()
            .map(BoxTokens::as_vec)
            .cloned()
            .unwrap_or_default(); // JSON serialization for assets requires that tokens be [] instead of null
        ErgoBoxCandidateJson {
            value: ergo_box_candidate.value,
            ergo_tree: ergo_box_candidate.ergo_tree,
            tokens,
            additional_registers: ergo_box_candidate.additional_registers,
            creation_height: ergo_box_candidate.creation_height,
        }
    }
}

impl TryFrom<ErgoBoxCandidateJson> for ErgoBoxCandidate {
    type Error = ErgoBoxFromJsonError;
    fn try_from(box_json: ErgoBoxCandidateJson) -> Result<Self, Self::Error> {
        let tokens = if box_json.tokens.is_empty() {
            None
        } else {
            Some(box_json.tokens.try_into().map_err(|_| {
                SigmaSerializationError::NotSupported(
                    "More than ErgoBox::MAX_TOKENS_COUNT tokens are not allowed in a box",
                )
            })?)
        };
        Ok(ErgoBoxCandidate {
            value: box_json.value,
            ergo_tree: box_json.ergo_tree,
            tokens,
            additional_registers: box_json.additional_registers,
            creation_height: box_json.creation_height,
        })
    }
}

/// Errors on parsing ErgoBox from JSON
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ErgoBoxFromJsonError {
    /// Box id parsed from JSON differs from calculated from box serialized bytes
    #[error(
        "Box id parsed from JSON {json} differs from calculated from box serialized bytes {actual}"
    )]
    InvalidBoxId { json: BoxId, actual: BoxId },
    /// Box serialization failed (id calculation)
    #[error("Box serialization failed (id calculation): {0}")]
    SerializationError(#[from] SigmaSerializationError),
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ConstantHolder(#[serde(deserialize_with = "super::t_as_string_or_struct")] RichConstant);

impl From<ConstantHolder> for RegisterValue {
    fn from(ch: ConstantHolder) -> Self {
        match Constant::sigma_parse_bytes(ch.0.raw_value.0.as_slice()) {
            Ok(c) => RegisterValue::Parsed(c),
            Err(_) => RegisterValue::Unparseable(ch.0.raw_value.0),
        }
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
struct RichConstant {
    #[serde(rename = "rawValue", alias = "serializedValue")]
    raw_value: ConstantWrapper,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(try_from = "Base16DecodedBytes")]
struct ConstantWrapper(Vec<u8>);

#[derive(Error, PartialEq, Eq, Debug, Clone, From)]
pub enum ConstantParsingError {
    #[error("Base16 decoding error: {0}")]
    DecodeError(base16::DecodeError),
    #[error("Deserialization error: {0}")]
    DeserializationError(SigmaParsingError),
}

impl TryFrom<Base16DecodedBytes> for ConstantWrapper {
    type Error = ConstantParsingError;

    fn try_from(Base16DecodedBytes(bytes): Base16DecodedBytes) -> Result<Self, Self::Error> {
        Ok(ConstantWrapper(bytes))
    }
}

impl FromStr for RichConstant {
    type Err = ConstantParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Base16DecodedBytes(bytes) = Base16DecodedBytes::try_from(s)?;
        Ok(RichConstant {
            raw_value: ConstantWrapper(bytes),
        })
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::chain::ergo_box::ErgoBox;
    use crate::chain::ergo_box::NonMandatoryRegisterId;
    use crate::chain::ergo_box::NonMandatoryRegisters;
    use crate::chain::token::Token;
    use crate::mir::constant::Constant;
    use crate::serialization::SigmaSerializable;
    use crate::types::stype::SType;
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;

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
        assert_eq!(regs.len(), 2)
    }

    #[test]
    fn parse_registers_explorer_api_v1() {
        let json = r#"
            {"R4":{"serializedValue":"0500","sigmaType":"SLong","renderedValue":"0"}}
                                                                                                                                           "#;
        let regs: NonMandatoryRegisters = serde_json::from_str(json).unwrap();
        assert!(regs.get_constant(NonMandatoryRegisterId::R4).is_some());
    }

    #[test]
    fn parse_registers_explorer_api_v2() {
        let json = r#"
            {
                "R4": {
                    "decodedValue": "Coll(-89,30,-127,32,-20,-100,-42,0,-25,-9,-25,107,-100,27,10,-97,127,127,-93,109,-48,70,51,-111,27,85,107,-116,97,102,87,45)",
                    "valueType": "Coll[Byte]",
                    "rawValue": "0e20a71e8120ec9cd600e7f7e76b9c1b0a9f7f7fa36dd04633911b556b8c6166572d"
                }
            }
        "#;
        let regs: NonMandatoryRegisters = serde_json::from_str(json).unwrap();
        assert!(regs.get_constant(NonMandatoryRegisterId::R4).is_some());
    }

    #[test]
    fn parse_registers_error() {
        // base16 decoding error
        let json = r#"
        {"R4":"0"}
        "#;
        let regs: Result<NonMandatoryRegisters, _> = serde_json::from_str(json);
        assert!(regs.is_err());
    }

    #[test]
    fn parse_registers_error2() {
        // invalid uparseable constant value
        let json = r#"
            {"R4":"860202660263"}
        "#;
        let regs: Result<NonMandatoryRegisters, _> = serde_json::from_str(json);
        assert!(regs.is_ok());
    }

    #[test]
    fn parse_registers_unit() {
        let json = r#" {"R4":"62"} "#;
        let regs: NonMandatoryRegisters = serde_json::from_str(json).unwrap();
        assert_eq!(regs.len(), 1)
    }

    #[test]
    fn parse_ergo_box() {
        let box_json = r#"{
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }"#;
        let b: ErgoBox = serde_json::from_str(box_json).unwrap();
        assert_eq!(b.value, 67500000000u64.try_into().unwrap());
    }

    #[test]
    fn parse_ergo_box_without_id() {
        let box_json = r#"{
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }"#;
        let b: ErgoBox = serde_json::from_str(box_json).unwrap();
        let id: String = b.box_id().into();
        assert_eq!(
            id,
            "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e"
        );
    }

    #[test]
    fn parse_ergo_box_alternative_box_id_field_name() {
        // check that using "id" field name instead of "boxId" also works
        // used in explorer API
        let box_json = r#"{
          "id": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }"#;
        let b: ErgoBox = serde_json::from_str(box_json).unwrap();
        assert_eq!(b.value, 67500000000u64.try_into().unwrap());
    }

    #[test]
    fn parse_ergo_box_alternative_transaction_id_field_name() {
        // check that using "txId" field name instead of "transactionId" also works
        // used in explorer API
        let box_json = r#"{
          "id": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "txId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }"#;
        let b: ErgoBox = serde_json::from_str(box_json).unwrap();
        assert_eq!(b.value, 67500000000u64.try_into().unwrap());
    }

    #[test]
    fn parse_ergo_box_from_explorer() {
        let box_json = r#"
        {
            "id": "3e762407d99b006d53b6583adcca08ef690b42fb0b2ed7abf63179eb6b9033b2",
            "txId": "93d344aa527e18e5a221db060ea1a868f46b61e4537e6e5f69ecc40334c15e38",
            "value": 2875858910,
            "index": 0,
            "creationHeight": 352126,
            "ergoTree": "101f0400040004020402040004000402050005000580dac4090580dac409050005c00c05c80104000e20b662db51cf2dc39f110a021c2a31c74f0a1a18ffffbf73e8a051a7b8c0f09ebc0580dac40904040404050005feffffffffffffffff01050005e807050005e807050005a0060101050005c00c05a006d81ed601b2db6501fe730000d602b2a5730100d603c17202d604db6308a7d605b27204730200d6068c720502d607db63087202d608b27207730300d6098c720802d60a9472067209d60bb27204730400d60c8c720b02d60db27207730500d60e8c720d02d60f94720c720ed610e4c6a70505d611e4c672020505d612e4c6a70405d613e4c672020405d614b2a5730600d615e4c672140405d61695720a73077215d61795720a72157308d61899c1a77309d619e4c672140505d61a997203730ad61be4c672010405d61ca172189c7212721bd61d9c7213721bd61e9593721d730b730c9d9c721a730d721dd1ededed938cb2db63087201730e0001730fedededed9272037310edec720a720fefed720a720fed939a720672109a72097211939a720c72129a720e7213eded939a721272167213939a721072177211939a72187219721aeded938c720d018c720b01938c7208018c720501938cb27207731100018cb272047312000193721995720f9ca1721b95937212731373149d721c72127216d801d61f997218721c9c9593721f7315731695937210731773189d721f7210721795720f95917216731992721e731a731b95917217731c90721e731d92721e731e",
            "address": "9aFbqNsmDwSxCdcLDKmSxVTL58ms2A39Rpn2zodVzkBN5MzB8zvW5PFX551W1A5vUdFJ3yxwvwgYTTS4JrPQcb5qxBbRDJkGNikuqHRXhnbniK4ajumEj7ot2o7DbcNFaM674fWufQzSGS1KtgMw95ZojyqhswUNbKpYDV1PhKw62bEMdJL9vAvzea4KwKXGUTdYYkcPdQKFWXfrdo2nTS3ucFNxqyTRB3VtZk7AWE3eeNHFcXZ1kLkfrX1ZBjpQ7qrBemHk4KZgS8fzmm6hPSZThiVVtBfQ2CZhJQdAZjRwGrw5TDcZ4BBDAZxg9h13vZ7tQSPsdAtjMFQT1DxbqAruKxX38ZwaQ3UfWmbBpbJEThAQaS4gsCBBSjswrv8BvupxaHZ4oQmA2LZiz4nYaPr8MJtR4fbM9LErwV4yDVMb873bRE5TBF59NipUyHAir7ysajPjbGc8aRLqsMVjntFSCFYx7822RBrj7RRX11CpiGK6vdfKHe3k14EH6YaNXvGSq8DrfNHEK4SgreknTqCgjL6i3EMZKPCW8Lao3Q5tbJFnFjEyntpUDf5zfGgFURxzobeEY4USqFaxyppHkgLjQuFQtDWbYVu3ztQL6hdWHjZXMK4VVvEDeLd1woebD1CyqS5kJHpGa78wQZ4iKygw4ijYrodZpqqEwTXdqwEB6xaLfkxZCBPrYPST3xz67GGTBUFy6zkXP5vwVVM5gWQJFdWCZniAAzBpzHeVq1yzaBp5GTJgr9bfrrAmuX8ra1m125yfeT9sTWroVu",
            "assets": [
                {
                    "tokenId": "2d554219a80c011cc51509e34fa4950965bb8e01de4d012536e766c9ca08bc2c",
                    "index": 0,
                    "amount": 99999999998
                },
                {
                    "tokenId": "bcd5db3a2872f279ef89edaa51a9344a6095ea1f03396874b695b5ba95ff602e",
                    "index": 1,
                    "amount": 99995619990
                },
                {
                    "tokenId": "9f90c012e03bf99397e363fb1571b7999941e0862a217307e3467ee80cf53af7",
                    "index": 2,
                    "amount": 1
                }
            ],
            "additionalRegisters": {
                "R4": "0504",
                "R5": "05d4d59604"
            },
            "spentTransactionId": null,
            "mainChain": true
        }
        "#;
        let b: ErgoBox = serde_json::from_str(box_json).unwrap();
        assert_eq!(b.value, 2875858910u64.try_into().unwrap());
    }

    #[test]
    fn parse_token_amount_as_num() {
        let token_json = r#"
        {
            "tokenId": "2d554219a80c011cc51509e34fa4950965bb8e01de4d012536e766c9ca08bc2c",
            "amount": 99999999998
        }"#;
        let t: Token = serde_json::from_str(token_json).unwrap();
        assert_eq!(t.amount, 99999999998u64.try_into().unwrap());
    }

    #[test]
    fn parse_token_amount_as_str() {
        let token_json = r#"               
        {
            "tokenId": "2d554219a80c011cc51509e34fa4950965bb8e01de4d012536e766c9ca08bc2c",
            "amount": "99999999998"
        }"#;
        let t: Token = serde_json::from_str(token_json).unwrap();
        assert_eq!(t.amount, 99999999998u64.try_into().unwrap());
    }

    #[test]
    fn encode_token_amount_as_num() {
        let token_json = "{\n  \"tokenId\": \"2d554219a80c011cc51509e34fa4950965bb8e01de4d012536e766c9ca08bc2c\",\n  \"amount\": 99999999998\n}";
        let t: Token = serde_json::from_str(token_json).unwrap();
        assert_eq!(t.amount, 99999999998u64.try_into().unwrap());
        let to_json = serde_json::to_string_pretty(&t).unwrap();
        assert_eq!(to_json, token_json);
    }

    #[test]
    fn parse_register_coll_box_issue_695_incorrect_method_id_and_missing_type_spec() {
        // see https://github.com/ergoplatform/sigma-rust/issues/695
        let constant_bytes_str = "0c63028092f401104904000e200137c91882b759ad46a20e39aa4d035ce32525dc76d021ee643e71d09446400f04020e20f6ff8b7210015545d4b3ac5fc60c908092d035a1a16155c029e8d511627c7a2c0e20efc4f603dea6041286a89f5bd516ac96ea5b25da4f08d76c6927e01d61b22adf040204000402040004000402040c044c04010404040404020e20f5918eb4b0283c669bdd8a195640766c19e40a693a6697b775b08e09052523d40e20767caa80b98e496ad8a9f689c4410ae453327f0f95e95084c0ae206350793b7704000402040004020412040005809bee0204000400040004000402040404000402041205d00f040304000402040204420580897a0e20012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5f040204040400041004100402041005000402040004100400040004000400040004100410040204100402040205000404040404020402040404040100d80dd601db6501fed602b27201730000d6037301d604b27201730200d605dc640be4c6720204640283020e73037304e4e3000ed606e4c6a70410d607b27206730500d608b2a5730600d609e4c672080410d60ab27209730700d60be3044005d60ce4720bd60d8c720c01d196830301938cb2db63087202730800017203938cb2db6308720473090001b4e4b27205730a00730b730c95ed947207720a937207730dd80cd60eb27201730e00d60fdb6308a7d610e4c6a70511d611720bd612720cd613b47210730fb17210d6148c721202d615b2a5731000d616dc640be4c6720e04640283020e73117312e4e3010ed617b2db63087215731300d6188cb2720f73140002d6197cb4e4b272167315007316731796830401938cb2db6308720e7318000172039683080193c27208c2a792c1720899c1a7731993b2db63087208731a00b2720f731b0093b27209731c00b27206731d0093e4c672080511721093e4c672080664e4c6a7066493720a9591b27210731e009d9cb2e4c672040511731f007cb4e4b27205732000732173227323720d7324edafdb0c0e7213d9011a049593721a720d93b27213721a00721490b27213721a00721491b17213720d91db6903db6503feb272107325009683040193cbc27215b4e4b272167326007327732892c172157329938c721701732a928c72170295927218721972187219d802d60ee4c6a70511d60fe4c6720805119594720e720fd809d610b2a4732b00d611e4c6b2a4732c00050ed612adb4db0c0e7211732d9db17211732ed90112047cb472119c7212732f9c9a721273307331d613b072127332d90113599a8c7213018c721302d614e4c6a70664d615e4c67210050ed616dc640a7214027215e4e3010ed617e67216d618e4e3020e96830801927cb4e4dc640ae4c672040464028cb2db6308721073330001e4e3030e73347335721393c27208c2a792c17208c1a793b2db63087208733600b2db6308a7733700937209720693b2720f733800b2720e733900957217d802d619e47216d61aadb4db0c0e7219733a9db17219733bd9011a047cb472199c721a733c9c9a721a733d733e9683020193b2720f733f009a99b2720e734000b0721a7341d9011b599a8c721b018c721b02721393b4720f7342b1720faddc0c1db4720e7343b1720e01addc0c1d721a017212d9011b59998c721b028c721b01d9011b599a8c721b018c721b029683020193b2720f7344009ab2720e734500721393b4720f7346b1720faddc0c1db4720e7347b1720e017212d90119599a8c7219018c72190293db6401e4c672080664db6401957217e4dc640d72140283013c0e0e8602721572117218e4dc640c72140283013c0e0e86027215721172187348efc13b02010b4858ce0425ed4748d0d3a59f2dbf874166a2caaf734655ac5e3f88a68cdd01012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5f904e0310020001110400000000644ec61f485b98eb87153f7c57db4f5ecd75556fddbc403b41acf8441fde8e1609000720003b6fd893695e655e17804641c3ed2074731ff34281dafb0e5b50688d0627717300c0843d10230400040204000402040604040500050004000e200137c91882b759ad46a20e39aa4d035ce32525dc76d021ee643e71d09446400f04000e20010b4858ce0425ed4748d0d3a59f2dbf874166a2caaf734655ac5e3f88a68cdd0400040204080400040204040502040604080400040004020402040004020e20c7c537e6c635930ecb4ace95a54926b3ab77698d9f4922f0b1c58ea87156483b0400040204420404040205000502d80ed601db6501fed602b27201730000d603b27201730100d604e4c672030410d605e4c6a70411d606b27205730200d607b27205730300d608b27205730400d609b27205730500d60a9172097306d60be4c6a7050c63d60cb1720bd60db1a5d60ed9010e0c63b0dc0c0f720e01d9011063db630872107307d90110414d0e9a8c7210018c8c72100202d196830701938cb2db6308720273080001730996830301938cb2db63087203730a0001730b937eb27204730c00057206937eb27204730d0005720792db6903db6503fe720895720ad804d60fe4c6a7050c63d610b2a5b1720f00d611e4c672100411d612b27205730e009683090192c17210c1a793db63087210db6308a793b27211730f00720693b27211731000720793b27211731100997209731293b272117313009a7208721293b27211731400721293e4c67210050c63720f93c27210c2a7efaea5d9010f63aedb6308720fd901114d0e938c7211018cb2db6308a773150001afdc0c1d720b01b4a5731699720c7317d9010f3c636393c48c720f01c48c720f0293720d9a9a720c95720a731873199593cbc2b2a599720d731a00b4e4b2dc640be4c6720204640283010e731be4e3000e731c00731d731e731f732093da720e01a49ada720e01a595720a73217322efc13b0100b44a84993674c57c4fc23c6c1bb221470463e4e711b2260ffd8ed01f1aab420102110500020000000c6301c0843d0008cd02e4cb952261186ec0fd2dc4c2baa8dbfd9c8f6012c5efa9f702f9450a58fe221eefc13b01012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5fc0843d00c9ffc2d2fd948bd3217a245eab2c9d829abd0cd3b41a8069006ff2bb38b06e2d00cb93d676f11faa9b7204a875fdb34d55619fc0e933ae956af8c7d8ce4c9e20ca00";
        let constant_bytes = base16::decode(constant_bytes_str).unwrap();
        let c_res = Constant::sigma_parse_bytes(&constant_bytes);
        if let Err(e) = &c_res {
            println!("Failed to parse constant: {}", e);
        }
        assert!(c_res.is_ok());
        assert_eq!(c_res.unwrap().tpe, SType::SColl(Box::new(SType::SBox)));
    }
}
