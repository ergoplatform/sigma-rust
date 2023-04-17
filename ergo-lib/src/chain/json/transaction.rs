use std::convert::TryFrom;
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
            .map_err(|e| format!("TryFrom<UnsignedTransactionJson> error: {0}", e))
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
            dbg!(&tx);
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
    fn unsigned_tx_with_coll_box_698() {
        // see https://github.com/ergoplatform/sigma-rust/issues/698
        let json = r#"
{
  "dataInputs": [],
  "inputs": [
    {
      "additionalRegisters": {},
      "assets": [
        {
          "amount": "1",
          "tokenId": "0d9753ffd979a45a561090a88ae2ece69b5044e54579d18bf801d982d9866d95"
        },
        {
          "amount": "1552509626735",
          "tokenId": "012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5f"
        }
      ],
      "boxId": "32a613293027c3af23350172c94f51ff7299dca60fe6886121801f847433d7fd",
      "creationHeight": 984288,
      "ergoTree": "0008cd02e4cb952261186ec0fd2dc4c2baa8dbfd9c8f6012c5efa9f702f9450a58fe221e",
      "extension": {},
      "index": 2,
      "transactionId": "09e6b8a764708bcbf63bdb81ede2e68086f549bde8d687b8823cf7ada7ef767c",
      "value": "1468043922"
    },
    {
      "additionalRegisters": {
        "R4": "0e105061696465696120566f7465204b6579",
        "R5": "0e105061696465696120566f7465204b6579",
        "R6": "0e0130"
      },
      "assets": [
        {
          "amount": "1",
          "tokenId": "64f05569ce927a6e1bc8b9167b1f6d91de91ef3262cb9a175f8bce4533e20e02"
        }
      ],
      "boxId": "bf857af737e17489097908e65cb921e37bf8a44a78a2b033a6ee8cdfcebdf618",
      "creationHeight": 984288,
      "ergoTree": "0008cd02e4cb952261186ec0fd2dc4c2baa8dbfd9c8f6012c5efa9f702f9450a58fe221e",
      "extension": {},
      "index": 2,
      "transactionId": "f5605c35866a080d1850fb72a97e7b0cb3db95953ff47d17a4a8ff42825df12e",
      "value": "1000000"
    }
  ],
  "outputs": [
    {
      "additionalRegisters": {
        "R4": "0e240008cd02e4cb952261186ec0fd2dc4c2baa8dbfd9c8f6012c5efa9f702f9450a58fe221e",
        "R5": "0c63028092f401104904000e200137c91882b759ad46a20e39aa4d035ce32525dc76d021ee643e71d09446400f04020e20f6ff8b7210015545d4b3ac5fc60c908092d035a1a16155c029e8d511627c7a2c0e20efc4f603dea6041286a89f5bd516ac96ea5b25da4f08d76c6927e01d61b22adf040204000402040004000402040c044c04010404040404020e20f5918eb4b0283c669bdd8a195640766c19e40a693a6697b775b08e09052523d40e20767caa80b98e496ad8a9f689c4410ae453327f0f95e95084c0ae206350793b7704000402040004020412040005809bee0204000400040004000402040404000402041205d00f040304000402040204420580897a0e20012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5f040204040400041004100402041005000402040004100400040004000400040004100410040204100402040205000404040404020402040404040100d80dd601db6501fed602b27201730000d6037301d604b27201730200d605dc640be4c6720204640283020e73037304e4e3000ed606e4c6a70410d607b27206730500d608b2a5730600d609e4c672080410d60ab27209730700d60be3044005d60ce4720bd60d8c720c01d196830301938cb2db63087202730800017203938cb2db6308720473090001b4e4b27205730a00730b730c95ed947207720a937207730dd80cd60eb27201730e00d60fdb6308a7d610e4c6a70511d611720bd612720cd613b47210730fb17210d6148c721202d615b2a5731000d616dc640be4c6720e04640283020e73117312e4e3010ed617b2db63087215731300d6188cb2720f73140002d6197cb4e4b272167315007316731796830401938cb2db6308720e7318000172039683080193c27208c2a792c1720899c1a7731993b2db63087208731a00b2720f731b0093b27209731c00b27206731d0093e4c672080511721093e4c672080664e4c6a7066493720a9591b27210731e009d9cb2e4c672040511731f007cb4e4b27205732000732173227323720d7324edafdb0c0e7213d9011a049593721a720d93b27213721a00721490b27213721a00721491b17213720d91db6903db6503feb272107325009683040193cbc27215b4e4b272167326007327732892c172157329938c721701732a928c72170295927218721972187219d802d60ee4c6a70511d60fe4c6720805119594720e720fd809d610b2a4732b00d611e4c6b2a4732c00050ed612adb4db0c0e7211732d9db17211732ed90112047cb472119c7212732f9c9a721273307331d613b072127332d90113599a8c7213018c721302d614e4c6a70664d615e4c67210050ed616dc640a7214027215e4e3010ed617e67216d618e4e3020e96830801927cb4e4dc640ae4c672040464028cb2db6308721073330001e4e3030e73347335721393c27208c2a792c17208c1a793b2db63087208733600b2db6308a7733700937209720693b2720f733800b2720e733900957217d802d619e47216d61aadb4db0c0e7219733a9db17219733bd9011a047cb472199c721a733c9c9a721a733d733e9683020193b2720f733f009a99b2720e734000b0721a7341d9011b599a8c721b018c721b02721393b4720f7342b1720faddc0c1db4720e7343b1720e01addc0c1d721a017212d9011b59998c721b028c721b01d9011b599a8c721b018c721b029683020193b2720f7344009ab2720e734500721393b4720f7346b1720faddc0c1db4720e7347b1720e017212d90119599a8c7219018c72190293db6401e4c672080664db6401957217e4dc640d72140283013c0e0e8602721572117218e4dc640c72140283013c0e0e86027215721172187348e3893c02010b4858ce0425ed4748d0d3a59f2dbf874166a2caaf734655ac5e3f88a68cdd01012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5f904e0310020401110400000000644ec61f485b98eb87153f7c57db4f5ecd75556fddbc403b41acf8441fde8e160900072000d35f8400db49e16a8185956c1fce96819bd407f8597a65120fb6bc02ebbc7f5e00c0843d10230400040204000402040604040500050004000e200137c91882b759ad46a20e39aa4d035ce32525dc76d021ee643e71d09446400f04000e20010b4858ce0425ed4748d0d3a59f2dbf874166a2caaf734655ac5e3f88a68cdd0400040204080400040204040502040604080400040004020402040004020e20c7c537e6c635930ecb4ace95a54926b3ab77698d9f4922f0b1c58ea87156483b0400040204420404040205000502d80ed601db6501fed602b27201730000d603b27201730100d604e4c672030410d605e4c6a70411d606b27205730200d607b27205730300d608b27205730400d609b27205730500d60a9172097306d60be4c6a7050c63d60cb1720bd60db1a5d60ed9010e0c63b0dc0c0f720e01d9011063db630872107307d90110414d0e9a8c7210018c8c72100202d196830701938cb2db6308720273080001730996830301938cb2db63087203730a0001730b937eb27204730c00057206937eb27204730d0005720792db6903db6503fe720895720ad804d60fe4c6a7050c63d610b2a5b1720f00d611e4c672100411d612b27205730e009683090192c17210c1a793db63087210db6308a793b27211730f00720693b27211731000720793b27211731100997209731293b272117313009a7208721293b27211731400721293e4c67210050c63720f93c27210c2a7efaea5d9010f63aedb6308720fd901114d0e938c7211018cb2db6308a773150001afdc0c1d720b01b4a5731699720c7317d9010f3c636393c48c720f01c48c720f0293720d9a9a720c95720a731873199593cbc2b2a599720d731a00b4e4b2dc640be4c6720204640283010e731be4e3000e731c00731d731e731f732093da720e01a49ada720e01a595720a73217322e3893c0100b44a84993674c57c4fc23c6c1bb221470463e4e711b2260ffd8ed01f1aab420102110504020090ea9db2f261000c6301000008cd02e4cb952261186ec0fd2dc4c2baa8dbfd9c8f6012c5efa9f702f9450a58fe221ee3893c01012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5fa08d06000909d9bf168f897d64f00458fc2294adcf89ac0b6e5718cf1199edaa0afc2b2700813096f27f9aedcedda6e766f429c87ecfed43e168c025c4bb2723bb89ff73b400"
      },
      "assets": [
        {
          "amount": "1",
          "tokenId": "64f05569ce927a6e1bc8b9167b1f6d91de91ef3262cb9a175f8bce4533e20e02"
        },
        {
          "amount": "11000",
          "tokenId": "012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5f"
        }
      ],
      "creationHeight": 984291,
      "ergoTree": "100204000e200137c91882b759ad46a20e39aa4d035ce32525dc76d021ee643e71d09446400fd193e4c6b2a4730000040e7301",
      "value": "8000000"
    },
    {
      "additionalRegisters": {},
      "assets": [],
      "creationHeight": 984291,
      "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
      "value": "1000000"
    },
    {
      "additionalRegisters": {},
      "assets": [
        {
          "amount": "1",
          "tokenId": "0d9753ffd979a45a561090a88ae2ece69b5044e54579d18bf801d982d9866d95"
        },
        {
          "amount": "1552509615735",
          "tokenId": "012aec95af24812a01775de090411ba70a648fe859013f896ca2a1a95882ce5f"
        }
      ],
      "creationHeight": 984291,
      "ergoTree": "0008cd02e4cb952261186ec0fd2dc4c2baa8dbfd9c8f6012c5efa9f702f9450a58fe221e",
      "value": "1460043922"
    }
  ]
} 
        "#;
        let tx: UnsignedTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.inputs.len(), 3);
    }
}
