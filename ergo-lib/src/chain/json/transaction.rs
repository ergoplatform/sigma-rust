use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryFrom;
use thiserror::Error;

use crate::chain::transaction::unsigned::UnsignedTransaction;
use crate::chain::transaction::{DataInput, Input, Transaction, TransactionError, UnsignedInput};
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::tx_id::TxId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct TransactionJson {
    #[cfg_attr(feature = "json", serde(rename = "id"))]
    pub tx_id: TxId,
    /// inputs, that will be spent by this transaction.
    #[cfg_attr(feature = "json", serde(rename = "inputs"))]
    pub inputs: Vec<Input>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    #[cfg_attr(feature = "json", serde(rename = "dataInputs"))]
    pub data_inputs: Vec<DataInput>,
    #[cfg_attr(feature = "json", serde(rename = "outputs"))]
    pub outputs: Vec<ErgoBox>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct UnsignedTransactionJson {
    /// unsigned inputs, that will be spent by this transaction.
    #[cfg_attr(feature = "json", serde(rename = "inputs"))]
    pub inputs: Vec<UnsignedInput>,
    /// inputs, that are not going to be spent by transaction, but will be reachable from inputs
    /// scripts. `dataInputs` scripts will not be executed, thus their scripts costs are not
    /// included in transaction cost and they do not contain spending proofs.
    #[cfg_attr(feature = "json", serde(rename = "dataInputs"))]
    pub data_inputs: Vec<DataInput>,
    /// box candidates to be created by this transaction
    #[cfg_attr(feature = "json", serde(rename = "outputs"))]
    pub outputs: Vec<ErgoBoxCandidate>,
}

impl From<UnsignedTransaction> for UnsignedTransactionJson {
    fn from(v: UnsignedTransaction) -> Self {
        UnsignedTransactionJson {
            inputs: v.inputs.as_vec().clone(),
            data_inputs: v
                .data_inputs
                .map(|di| di.as_vec().clone())
                .unwrap_or_default(),
            outputs: v.output_candidates.as_vec().clone(),
        }
    }
}

impl TryFrom<UnsignedTransactionJson> for UnsignedTransaction {
    // We never return this type but () fails to compile (can't format) and ! is experimental
    type Error = String;
    fn try_from(tx_json: UnsignedTransactionJson) -> Result<Self, Self::Error> {
        UnsignedTransaction::new_from_vec(tx_json.inputs, tx_json.data_inputs, tx_json.outputs)
            .map_err(|e| format!("TryFrom<UnsignedTransactionJson> error: {0:?}", e))
    }
}

impl From<Transaction> for TransactionJson {
    fn from(v: Transaction) -> Self {
        TransactionJson {
            tx_id: v.id(),
            inputs: v.inputs.as_vec().clone(),
            data_inputs: v
                .data_inputs
                .map(|di| di.as_vec().clone())
                .unwrap_or_default(),
            outputs: v.outputs.to_vec(),
        }
    }
}

/// Errors on parsing Transaction from JSON
#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[allow(missing_docs)]
pub enum TransactionFromJsonError {
    #[error(
        "Tx id parsed from JSON {expected} differs from calculated from serialized bytes {actual}"
    )]
    InvalidTxId { expected: TxId, actual: TxId },
    #[error("Tx error: {0}")]
    TransactionError(#[from] TransactionError),
}

impl TryFrom<TransactionJson> for Transaction {
    type Error = TransactionFromJsonError;
    fn try_from(tx_json: TransactionJson) -> Result<Self, Self::Error> {
        let output_candidates: Vec<ErgoBoxCandidate> =
            tx_json.outputs.iter().map(|o| o.clone().into()).collect();
        let tx = Transaction::new_from_vec(tx_json.inputs, tx_json.data_inputs, output_candidates)?;
        if tx.tx_id == tx_json.tx_id {
            Ok(tx)
        } else {
            //dbg!(&tx);
            Err(TransactionFromJsonError::InvalidTxId {
                expected: tx_json.tx_id,
                actual: tx.tx_id,
            })
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::unwrap_used)]
mod tests {
    use crate::chain::transaction::unsigned::UnsignedTransaction;
    use crate::chain::transaction::Transaction;
    use ergo_chain_types::Digest32;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn tx_roundtrip(t in any::<Transaction>()) {
            let j = serde_json::to_string(&t)?;
            eprintln!("{}", j);
            let t_parsed: Transaction = serde_json::from_str(&j)?;
            prop_assert_eq![t, t_parsed];
        }

        #[test]
        fn unsigned_tx_roundtrip(t in any::<UnsignedTransaction>()) {
            let j = serde_json::to_string(&t)?;
            eprintln!("{}", j);
            let t_parsed: UnsignedTransaction = serde_json::from_str(&j)?;
            prop_assert_eq![t, t_parsed];
        }

    }

    #[test]
    fn item_order_preservation_685() {
        let tx_json = r#"
{
        "id" : "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
        "inputs" : [
          {
            "boxId" : "59f2856068c56264d290520043044ace138a3a80d414748d0e4dcd0806188546",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                "0" : "04c60f",
                "5" : "0514",
                "10" : "0eee03101808cd0279aed8dea2b2a25316d5d49d13bf51c0b2c1dc696974bb4b0c07b5894e998e56040005e0e0a447040404060402040004000e201d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f0e20003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d005e201058c85a2010514040404c60f06010104d00f05e0e0a44704c60f0e691005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304050005000580ade2040100d803d6017300d602b2a4730100d6037302eb027201d195ed93b1a4730393b1db630872027304d804d604db63087202d605b2a5730500d606b2db63087205730600d6077e8c72060206edededededed938cb2720473070001730893c27205d07201938c72060173099272077e730a06927ec172050699997ec1a7069d9c72077e730b067e730c067e720306909c9c7e8cb27204730d0002067e7203067e730e069c9a7207730f9a9c7ec17202067e7310067e9c73117e7312050690b0ada5d90108639593c272087313c1720873147315d90108599a8c7208018c72080273167317",
                "1" : "0e20003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d0",
                "6" : "0580ade204",
                "9" : "0580b48913",
                "2" : "05e201",
                "7" : "0e201d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f",
                "3" : "05e0e0a447",
                "8" : "0580ade204",
                "4" : "058c85a201"
              }
            }
          }
        ],
        "dataInputs" : [
        ],
        "outputs" : [
          {
            "boxId" : "0586c90d0cf6a82dab48c6a79500364ddbd6f81705f5032b03aa287de43dc638",
            "value" : 94750000,
            "ergoTree" : "101808cd0279aed8dea2b2a25316d5d49d13bf51c0b2c1dc696974bb4b0c07b5894e998e56040005e0e0a447040404060402040004000e201d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f0e20003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d005e201058c85a2010514040404c60f06010104d00f05e0e0a44704c60f0e691005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304050005000580ade2040100d803d6017300d602b2a4730100d6037302eb027201d195ed93b1a4730393b1db630872027304d804d604db63087202d605b2a5730500d606b2db63087205730600d6077e8c72060206edededededed938cb2720473070001730893c27205d07201938c72060173099272077e730a06927ec172050699997ec1a7069d9c72077e730b067e730c067e720306909c9c7e8cb27204730d0002067e7203067e730e069c9a7207730f9a9c7ec17202067e7310067e9c73117e7312050690b0ada5d90108639593c272087313c1720873147315d90108599a8c7208018c72080273167317",
            "assets" : [
            ],
            "creationHeight" : 693475,
            "additionalRegisters" : {

            },
            "transactionId" : "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
            "index" : 0
          },
          {
            "boxId" : "4b99c2ef8496a491d176ecaf789d9e1d9aad0c2bf3e70b32e8bad73f48c722b9",
            "value" : 250000,
            "ergoTree" : "100e04000500059a0505d00f04020404040608cd03c6543ac8e8059748b1c6209ee419dd49a19ffaf5712a2f34a9412016a3a1d96708cd035b736bebf0c5393f78329f6894af84d1864c7496cc65ddc250ef60cdd75df52008cd021b63e19ab452c84cdc6687242e8494957b1f11e3750c8c184a8425f8a8171d9b05060580ade2040580a8d6b907040ad806d601b2a5730000d602b0a47301d9010241639a8c720201c18c720202d6039d9c730272027303d604b2a5730400d605b2a5730500d606b2a5730600d1968306019683020193c17201720393c27201d073079683020193c17204720393c27204d073089683020193c17205720393c27205d073099683020192c17206999972029c730a7203730b93c2a7c27206927202730c93b1a5730d",
            "assets" : [
            ],
            "creationHeight" : 693475,
            "additionalRegisters" : {

            },
            "transactionId" : "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
            "index" : 1
          },
          {
            "boxId" : "00ddbeb981c0b08536f72ea41e07a25adbf7bf104ee59b865619a21676e64715",
            "value" : 5000000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693475,
            "additionalRegisters" : {

            },
            "transactionId" : "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
            "index" : 2
          }
        ],
        "size" : 1562
      }
    "#;
        let tx: Transaction = serde_json::from_str(tx_json).unwrap();
        assert_eq!(
            tx.id(),
            Digest32::try_from(
                "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4".to_string()
            )
            .unwrap()
            .into()
        );
    }

    #[test]
    fn parse_node_tx_with_unsized_tree_that_does_not_parse() {
        // Transaction as the node 6.0.7 API serves it, with one output whose unsized tree
        // the node accepts but this library cannot parse. The transaction and box ids are
        // the node's.
        let tx_json = r#"{"id":"3d912605036eb79a02bbc2525be5d554c480b1186eb4402a826a41fb9c2a62af","inputs":[{"boxId":"854599b094f0b455521c65fbc8848cfd27aa6d6617bfa437f39aec584cf39161","spendingProof":{"proofBytes":"","extension":{}}}],"dataInputs":[],"outputs":[{"boxId":"1815bfcc63bd0bc1ec1112e50aca444d2509374d23054dbceccee6e18909fdeb","value":1000000000,"ergoTree":"00d193b2100202040200000402","assets":[],"creationHeight":1650000,"additionalRegisters":{},"transactionId":"3d912605036eb79a02bbc2525be5d554c480b1186eb4402a826a41fb9c2a62af","index":0}],"size":61}"#;
        let tx_id = "3d912605036eb79a02bbc2525be5d554c480b1186eb4402a826a41fb9c2a62af";
        let box_id = "1815bfcc63bd0bc1ec1112e50aca444d2509374d23054dbceccee6e18909fdeb";
        let tx: Transaction = serde_json::from_str(tx_json).unwrap();
        assert_eq!(
            tx.id(),
            Digest32::try_from(tx_id.to_string()).unwrap().into()
        );
        let id: String = tx.outputs.first().box_id().into();
        assert_eq!(id, box_id);
    }
}
