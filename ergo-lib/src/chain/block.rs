//! Block on the Ergo chain

use bounded_vec::BoundedVec;
use ergo_chain_types::Header;

use super::transaction::Transaction;

/// Maximum number of transactions that can be contained in a block. See
/// https://github.com/ergoplatform/ergo/blob/fc292f6bc2d3c6ca27ce5f6a316186d8459150cc/src/main/scala/org/ergoplatform/modifiers/history/BlockTransactions.scala#L157
const MAX_NUM_TRANSACTIONS: usize = 10_000_000;

/// Transactions in a block
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockTransactions {
    /// Transactions contained in the block
    pub transactions: BoundedVec<Transaction, 1, MAX_NUM_TRANSACTIONS>,
}

/// A block on the Ergo chain
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullBlock {
    /// Block header
    pub header: Header,
    /// Transactions in this block
    #[cfg_attr(feature = "json", serde(rename = "blockTransactions"))]
    pub block_transactions: BlockTransactions,
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::FullBlock;

    #[test]
    fn test_parse_full_block() {
        // Following JSON taken from the node by:
        //   curl -X GET "https://node.ergo.watch/blocks/96911575efdceb082b974aa3042263be07632de48031aa2204d77d8d5a8240b8" -H "accept: application/json"

        let json: &str = r#"
{
  "header" : {
    "extensionId" : "f2b3d4db0504d2df78f661cd980ed7f2b861a2cf9f0acfd1274e4b22bd468abe",
    "difficulty" : "1667667081560064",
    "votes" : "000000",
    "timestamp" : 1645753676103,
    "size" : 221,
    "stateRoot" : "37e32d56e08d94170f8c54ecee1e65d526841524773c3769ea4aaf8bd014d6c018",
    "height" : 693479,
    "nBits" : 117828796,
    "version" : 2,
    "id" : "b17847c0c523660b13d707396ab8301fa3c8a545ddc5acf9ec2803cc2cbb3ef5",
    "adProofsRoot" : "1130ce510264cd3707592f14631fbb994cb3624219f159b5cbf3119efcf25b8c",
    "transactionsRoot" : "57fb74737b99ea1f57e206ce76743e5970da060fed27cea6c8f50b7d587c8beb",
    "extensionHash" : "5fb2be1ff25d365daadbb3cd4908feceff097bb4ad8f6c7f1436a04ffa3bf5cd",
    "powSolutions" : {
      "pk" : "0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891f",
      "w" : "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
      "n" : "b8e6000bf8e4f5fc",
      "d" : 0
    },
    "adProofsId" : "30cf0f1b493a636b4b5554937424dffaaf04b59af6e4a35862031c2c2a8b549e",
    "transactionsId" : "f90ad5454590737a1a54e2b28524db8ba94cc0c14eaf43533cda77acedde5bb9",
    "parentId" : "72a17b1cd5863bbc598f68678a35168ae5e4eadf602edd778eb3b6f7312cdc65"
  },
  "blockTransactions" : {
    "headerId" : "b17847c0c523660b13d707396ab8301fa3c8a545ddc5acf9ec2803cc2cbb3ef5",
    "transactions" : [
      {
        "id" : "ba3f7ac52edb58663f976cf4b390cf57b8e16fc51488c0a944446e5eaa733982",
        "inputs" : [
          {
            "boxId" : "5616b8101a8600ebddf33b55c34554f3b704fc9bf5b7b47b2523a66d1f072e28",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
        ],
        "outputs" : [
          {
            "boxId" : "e34e00e645d12af74f82c74ded1ae3718004bef594371b760c75e630f036b86c",
            "value" : 46656720000000000,
            "ergoTree" : "101004020e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a7017300730110010204020404040004c0fd4f05808c82f5f6030580b8c9e5ae040580f882ad16040204c0944004c0f407040004000580f882ad16d19683030191a38cc7a7019683020193c2b2a57300007473017302830108cdeeac93a38cc7b2a573030001978302019683040193b1a5730493c2a7c2b2a573050093958fa3730673079973089c73097e9a730a9d99a3730b730c0599c1a7c1b2a5730d00938cc7b2a5730e0001a390c1a7730f",
            "assets" : [
            ],
            "creationHeight" : 693479,
            "additionalRegisters" : {
              
            },
            "transactionId" : "ba3f7ac52edb58663f976cf4b390cf57b8e16fc51488c0a944446e5eaa733982",
            "index" : 0
          },
          {
            "boxId" : "9aae966e3e23a0b45f50b373c5d2e7de036a061e36fb63de8dde4f462f329427",
            "value" : 66000000000,
            "ergoTree" : "100204a00b08cd0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891fea02d192a39a8cc7a70173007301",
            "assets" : [
            ],
            "creationHeight" : 693479,
            "additionalRegisters" : {
              
            },
            "transactionId" : "ba3f7ac52edb58663f976cf4b390cf57b8e16fc51488c0a944446e5eaa733982",
            "index" : 1
          }
        ],
        "size" : 344
      },
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
      },
      {
        "id" : "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
        "inputs" : [
          {
            "boxId" : "dded9c2fb796044d39a52e9925196a031fb8ef6ff521e6754d0b6f330e7ece25",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "0586c90d0cf6a82dab48c6a79500364ddbd6f81705f5032b03aa287de43dc638",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
        ],
        "outputs" : [
          {
            "boxId" : "2b2c6cffdd5cb315bfc88830124b285633d2b4edff165a638ef7ac17aa45e084",
            "value" : 67333814889739,
            "ergoTree" : "1999030f0400040204020404040405feffffffffffffffff0105feffffffffffffffff01050004d00f040004000406050005000580dac409d819d601b2a5730000d602e4c6a70404d603db63087201d604db6308a7d605b27203730100d606b27204730200d607b27203730300d608b27204730400d6099973058c720602d60a999973068c7205027209d60bc17201d60cc1a7d60d99720b720cd60e91720d7307d60f8c720802d6107e720f06d6117e720d06d612998c720702720fd6137e720c06d6147308d6157e721206d6167e720a06d6177e720906d6189c72117217d6199c72157217d1ededededededed93c27201c2a793e4c672010404720293b27203730900b27204730a00938c7205018c720601938c7207018c72080193b17203730b9593720a730c95720e929c9c721072117e7202069c7ef07212069a9c72137e7214067e9c720d7e72020506929c9c721372157e7202069c7ef0720d069a9c72107e7214067e9c72127e7202050695ed720e917212730d907216a19d721872139d72197210ed9272189c721672139272199c7216721091720b730e",
            "assets" : [
              {
                "tokenId" : "1d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f",
                "amount" : 1
              },
              {
                "tokenId" : "fa6326a26334f5e933b96470b53b45083374f71912b0d7597f00c2c7ebeb5da6",
                "amount" : 9223371996546264297
              },
              {
                "tokenId" : "003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d0",
                "amount" : 105824543
              }
            ],
            "creationHeight" : 0,
            "additionalRegisters" : {
              "R4" : "04c60f"
            },
            "transactionId" : "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index" : 0
          },
          {
            "boxId" : "e7a483349c1737eedcebe5b7bb2e8efed95774d938562ab77626d9e891895932",
            "value" : 4601812,
            "ergoTree" : "0008cd0279aed8dea2b2a25316d5d49d13bf51c0b2c1dc696974bb4b0c07b5894e998e56",
            "assets" : [
              {
                "tokenId" : "003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d0",
                "amount" : 116
              }
            ],
            "creationHeight" : 0,
            "additionalRegisters" : {
              
            },
            "transactionId" : "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index" : 1
          },
          {
            "boxId" : "805f9337e55365284e1ba736ab0ef30f9f0667d009d424dfb56e4dcc64351be4",
            "value" : 10398188,
            "ergoTree" : "0008cd0273cbc003da723c0a5f416929692e8ec8c2b1e0d9aed69ff681f7581c63e70309",
            "assets" : [
            ],
            "creationHeight" : 0,
            "additionalRegisters" : {
              
            },
            "transactionId" : "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index" : 2
          },
          {
            "boxId" : "5291c872959a81fa1e0f6696b41e911a09be732ab70871d9504a0792249c3633",
            "value" : 5000000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 0,
            "additionalRegisters" : {
              
            },
            "transactionId" : "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index" : 3
          }
        ],
        "size" : 810
      },
      {
        "id" : "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
        "inputs" : [
          {
            "boxId" : "374facc223ff4984b1d5dd295392892a84060f706a26c8212d7daf5abd814e48",
            "spendingProof" : {
              "proofBytes" : "0af556eefd5d0a03bc09313ad8642291b20bd10304cf3f36858b713c77645bd38201fde1a24e0a4e18055354c18426f5241ce650980dd097",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "6407455a1805553de3f68894b046bd4d52a63235148f8284e74b7ab1a0641aa0",
            "spendingProof" : {
              "proofBytes" : "bec07fd4f046702e9fab5c0b8032cb5c9aca273fd1181c51c397c522d1680a286b12609d2b52209dd61c92e7ab98f99e5de2846cc9c057a4",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs" : [
          {
            "boxId" : "a26a61e76d6430e88e3cac9314ca7e0fe203357d0d1d6686471c3d411774d242",
            "value" : 1000000,
            "ergoTree" : "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets" : [
              {
                "tokenId" : "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "0702725e8878d5198ca7f5853dddf35560ddab05ab0a26adae7e664b84162c9962e5",
              "R5" : "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6" : "059cb3aadd02"
            },
            "transactionId" : "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
            "index" : 0
          },
          {
            "boxId" : "8847701f6eaa5629ab350f894a626b533bdf83beab6b12c620ad854cb5cad02d",
            "value" : 2100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
            "index" : 1
          },
          {
            "boxId" : "bff4862131e88d6ac780398e2b7da89aa5dbb478a22aa319af41c61e27bb4167",
            "value" : 516400000,
            "ergoTree" : "0008cd02725e8878d5198ca7f5853dddf35560ddab05ab0a26adae7e664b84162c9962e5",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
            "index" : 2
          }
        ],
        "size" : 631
      },
      {
        "id" : "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
        "inputs" : [
          {
            "boxId" : "9f7bffab806cecc0e5dd95f63d654eeb003fcee36bdf5e602576a6907d482b55",
            "spendingProof" : {
              "proofBytes" : "067713cc8d22ac6c6dfa0870e87c3d92c35ce31d7d4e87321b0d1af453775e5f829be60e6ef30e1d019b4483654c9631c9450b495a4dc8a7",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
        ],
        "outputs" : [
          {
            "boxId" : "5761b7fe988db401551ab777fbe00937df93c72d886221851874b32ebfa1c3f2",
            "value" : 1000000,
            "ergoTree" : "0008cd03a3591b90c96a48923dae01861e14f48419e17ab13f1d36823b6914d4d656bd1f",
            "assets" : [
              {
                "tokenId" : "472c3d4ecaa08fb7392ff041ee2e6af75f4a558810a74b28600549d5392810e8",
                "amount" : 47696000000
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
            "index" : 0
          },
          {
            "boxId" : "90a0ed9da366367c88cca4d8c63bbca4e77f96deef885f48df564f857a78d374",
            "value" : 1000000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
            "index" : 1
          },
          {
            "boxId" : "caa6c72765455b673f753274507bde0548b5b5c6f1d5fbb21b7d93b8ec18e7b9",
            "value" : 366775815551,
            "ergoTree" : "0008cd027163c8e38f64bb6df20679c26b81518a49603be43e6691ca798b6baa003abc19",
            "assets" : [
              {
                "tokenId" : "472c3d4ecaa08fb7392ff041ee2e6af75f4a558810a74b28600549d5392810e8",
                "amount" : 100000000000
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
            "index" : 2
          }
        ],
        "size" : 344
      },
      {
        "id" : "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
        "inputs" : [
          {
            "boxId" : "df1a50bddb2afb70479f130292a69bf1df1524e44442b5f3e3cfd2452aa6de81",
            "spendingProof" : {
              "proofBytes" : "91027169db56dc1d98a960d05f698d88a409108c079143bed6677ece572d0c0bee4e90a2160327151de78cdd773c08b8e856af9fd79a6085",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "0f7ce74d938750dee1fb47709ab27901810123a8d25e765fec5f60a8e283e342",
            "spendingProof" : {
              "proofBytes" : "5535dddd86417e56ec61a217ab9355121703a6243c529bba918b6dc27b69e455b6affc7154b816a21ceb7d3ee158dbcd04ae7e13a65dd9dc",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs" : [
          {
            "boxId" : "31b9ee4ae67f421e16d38f1513ad06994539e65d8ac0a7e1d4141c35e256001b",
            "value" : 1000000,
            "ergoTree" : "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets" : [
              {
                "tokenId" : "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "0702c1d434dac8765fc1269af82958d8aa350da53907096b35f7747cc372a7e6e69d",
              "R5" : "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6" : "059cb3aadd02"
            },
            "transactionId" : "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
            "index" : 0
          },
          {
            "boxId" : "85dc5b034f8815952cdbad90aa7b50b68b56702e76b89ca799e2dc0e47b35eca",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
            "index" : 1
          },
          {
            "boxId" : "f6558bcfac2b03facf55b2ad2714cd7eefe51d3a90ead323fc2860d4e101270c",
            "value" : 9347700000,
            "ergoTree" : "0008cd02c1d434dac8765fc1269af82958d8aa350da53907096b35f7747cc372a7e6e69d",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
            "index" : 2
          }
        ],
        "size" : 630
      },
      {
        "id" : "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
        "inputs" : [
          {
            "boxId" : "57e3f78aa182e5d575625b0832b77e0a31caf52ed1f79a093f1f51602d75bb7f",
            "spendingProof" : {
              "proofBytes" : "826b43caead76bc2a1f158e42c1f372bdbc481dad2fd05889d0713b9c68457c4ec018107460203e932963bb16465f35cf71b3687fe0f1b85",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "4f6d379aefb353cdf32ffa6ad65294b6460ea20b8628b46ea3743cf519f5f738",
            "spendingProof" : {
              "proofBytes" : "d1579debf190477899a1d889441b62003f272b3d53e9013d484f02ede6cbe91e673d6c984be3f5d586c99e82540fb2c67b2a46b439330615",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs" : [
          {
            "boxId" : "b43ba3e7dfdc766f48b434d9b7007e4c5f496a6bec4bc7298f33c8f8bebbcf5b",
            "value" : 1000000,
            "ergoTree" : "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets" : [
              {
                "tokenId" : "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "070331b99a9fcc7bceb0a238446cdab944402dd4b2e79f9dcab898ec3b46aea285c8",
              "R5" : "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6" : "059cb3aadd02"
            },
            "transactionId" : "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
            "index" : 0
          },
          {
            "boxId" : "5b44961a26230f4c37dba57125149284fe808986bfb939bf35aefce35eae554a",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
            "index" : 1
          },
          {
            "boxId" : "1bd0d3cadfd72d823e98880ddfcee15981c1d6bcf90725e72186744f5377a7ce",
            "value" : 9300400000,
            "ergoTree" : "0008cd0333920f80ca39477cb57ccdff9847ed6cbd46cf2c7237b6b085979622349910e9",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
            "index" : 2
          }
        ],
        "size" : 630
      },
      {
        "id" : "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
        "inputs" : [
          {
            "boxId" : "1574df6b63c6794a981f112c6accad303322a0322502286498c9d44b243b0d5b",
            "spendingProof" : {
              "proofBytes" : "4d28e70c901fb884e9969135096c46f3154a791b159b78b3c83b42d489efc5b1dff6a8337b2be141b6eac9a20618256f03f1da5a8bdcf85a",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "b2fbe2d21b2b994f62faf4640a90c2b85842f0ea5430abc19dbe012cb449cb56",
            "spendingProof" : {
              "proofBytes" : "f1e40312ecd9d8c1c7dc4f3ddbd7a20c4cd44acfd22ce3daeaa24d109ea7cb1d9d94ed89658d267bf1001c54b90907838784e9c37fe11bc1",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs" : [
          {
            "boxId" : "0c3c66a0cdff6b3a1c6c3a698655fa92143c96b83465b25c7a64af3a8594dd3a",
            "value" : 1000000,
            "ergoTree" : "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets" : [
              {
                "tokenId" : "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "070274524ee849e4e45f58c46164ac609902bb374fc9375f097ee1af2ef1152ab9bf",
              "R5" : "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6" : "059cb3aadd02"
            },
            "transactionId" : "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
            "index" : 0
          },
          {
            "boxId" : "9d383716046dab202a2cb6b285ef391b46ccfc32839b427bf1e210374cbfcf59",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
            "index" : 1
          },
          {
            "boxId" : "056d67b049020bf92923c752df9f363a1c41ffe0b2e2bfafbb86d63e62538558",
            "value" : 9259700000,
            "ergoTree" : "0008cd0274524ee849e4e45f58c46164ac609902bb374fc9375f097ee1af2ef1152ab9bf",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
            "index" : 2
          }
        ],
        "size" : 630
      },
      {
        "id" : "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
        "inputs" : [
          {
            "boxId" : "d811aff2532aa4e3c871fe3ce1367b3fbef24b686b425d0134274cb84a119bd0",
            "spendingProof" : {
              "proofBytes" : "fe48e6ab221c92d27d027492b2083c0d2385b8f609de13e3906a10e5c0279d3e212ed575f5980c8b091b9e6ebc6ccd51d8b2f245df0273ce",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "0c5deb864523818625329ba7e991dd3ebf0eb5dc9edc48dcbb19f2ac16b7eefe",
            "spendingProof" : {
              "proofBytes" : "7670d50456b7300a4755d9266d804d5b4377d13149311a9e81ac6ea25713e64d81e894ec77e73cc9a4fff698478df06cbdc1785662dd9858",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs" : [
          {
            "boxId" : "09b804f8d925b9a279143c3bfbd6878024ae64fffaa5c2590f1987b6b78bdd82",
            "value" : 1000000,
            "ergoTree" : "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets" : [
              {
                "tokenId" : "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "0702caad8ef6771ad15ebb0a2aa9b7e84b9c48962976061d1af3e73767203d2f2bb1",
              "R5" : "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6" : "059cb3aadd02"
            },
            "transactionId" : "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
            "index" : 0
          },
          {
            "boxId" : "18aaabf889a4d3a111b07a2beb932700cf6e0db4f2c654044aef507f1860449c",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
            "index" : 1
          },
          {
            "boxId" : "279d6975dc591abd96fb18460808ac5d6f46e9c9e228ba40f4dc3f6dd7e7d2c6",
            "value" : 10253450000,
            "ergoTree" : "0008cd02caad8ef6771ad15ebb0a2aa9b7e84b9c48962976061d1af3e73767203d2f2bb1",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
            "index" : 2
          }
        ],
        "size" : 630
      },
      {
        "id" : "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
        "inputs" : [
          {
            "boxId" : "0cd3c09ac65fa259ddc68c93dcdf78395650f05b3df046241117df23a5580f92",
            "spendingProof" : {
              "proofBytes" : "397cf85ba400a931ba146d971df08b624a10f5f7aa16a065b09d1185af7840519d21772ce8b905daad0e8e75abbfea348b75cb1fa76121cf",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "90007c1f6b06a013366556c0e450bc4766ac5d0c09a2d99727c64201add1b144",
            "spendingProof" : {
              "proofBytes" : "cacb926366f7845b24d1b1cec05b9a9e022fa9fccb2b828ec9971511b3c5e191cdd0308b1f62e564fbadcb49d1614b7100a85c6407f1c383",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs" : [
          {
            "boxId" : "5e164aaaf0c1a2ef7c15332df4813bfcf8309c8baabeb75a711f4b707558007f",
            "value" : 1000000,
            "ergoTree" : "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets" : [
              {
                "tokenId" : "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "0703082348fd5d0c27d7aa89cd460a58fea2932f12147a04985e500bd9ad64695d58",
              "R5" : "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6" : "059cb3aadd02"
            },
            "transactionId" : "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
            "index" : 0
          },
          {
            "boxId" : "f259d6bf9d4d5564401a2ac4da7e3eabe6c01288a177a23f360b34280fe262cf",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
            "index" : 1
          },
          {
            "boxId" : "03ca62471d9ac5dd9fd67ac8d895f6f868957dad07ce3d796b5ac80590bbe5e9",
            "value" : 6802400000,
            "ergoTree" : "0008cd03082348fd5d0c27d7aa89cd460a58fea2932f12147a04985e500bd9ad64695d58",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
            "index" : 2
          }
        ],
        "size" : 630
      },
      {
        "id" : "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
        "inputs" : [
          {
            "boxId" : "aef21868eb2055412ae6f5dbc74d9ff6024a6b68a21015703ae8ed580d463ea9",
            "spendingProof" : {
              "proofBytes" : "cab2409bc996779a18b89f4b9a75a7a2192046195fef7233d2c3070acf60dbb497c34cfe3394e6f7a3dba6754120f0e768b4ab5121c46f10",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "5ce7e907eab35d5b01d84c1baf02bf2f08cb480bcb51d04bd8a0489945b62bd0",
            "spendingProof" : {
              "proofBytes" : "c04f3299d5eca7f98c8c05aeff1874351dfb5d3149d07e194324b881a6692c843a2aad1291d7773f426a7dfe7d86dc53276a99de97b0b976",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "9d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b"
          }
        ],
        "outputs" : [
          {
            "boxId" : "f59f9be73b4983f55f71556a08fd989152a46b3147c6a8fffd3d85974b8ea1aa",
            "value" : 1000000,
            "ergoTree" : "100604000400050004000e20002693cd6c3dc7c156240dd1c7370e50c4d1f84a752c2f74d93a20cc22c2899d0e204759889b16a97b0c7ab5ccb30c7fafb7d9e17fd6dc41ab86ae380784abe03e4cd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7ed938cb2db6308720373030001730493cbc272037305cd7202",
            "assets" : [
              {
                "tokenId" : "01e6498911823f4d36deaf49a964e883b2c4ae2a4530926f18b9c1411ab2a2c2",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "0703a7405d595770313bae0b88f97cf0543750df771f0d183283a4b0f86127ad4f29",
              "R5" : "0e209d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b",
              "R6" : "05d0d4affcc4a9ad01"
            },
            "transactionId" : "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
            "index" : 0
          },
          {
            "boxId" : "483f1420cc1afce2ea539075654800516c9a8341a3b41d9ba435ed9f217ceb50",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
            "index" : 1
          },
          {
            "boxId" : "fa43dd7eb02e0a04132bac2dc1da2fecc715bfdfc7ffdfa218566c5667a8ae38",
            "value" : 1400000,
            "ergoTree" : "0008cd03a7405d595770313bae0b88f97cf0543750df771f0d183283a4b0f86127ad4f29",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
            "index" : 2
          }
        ],
        "size" : 673
      },
      {
        "id" : "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
        "inputs" : [
          {
            "boxId" : "15f172920d10dffdaa1ef1e949ce00b481189c76e7e413850f4cf3253d3f3ce9",
            "spendingProof" : {
              "proofBytes" : "f0066c7cad01b57cff3b88cec0bfc6eed1d2c345659e56749fb0c0b6ae3bd5393cf97dfcab3d270a3d82d01062172e4f06f3a8b8c98fe7e8",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "4f0a60665a97930e251c7a0b0ad14a3f011e98e4c6e3b18ce29a2e441abfaa26",
            "spendingProof" : {
              "proofBytes" : "64a1feed5111f18cd06a087043b7dd72fe656ed0f1cb376e88da7560fc13ef390c85418fec5bf4a2d69e84ae300f351fcf0599e43e699cc0",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "9d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b"
          }
        ],
        "outputs" : [
          {
            "boxId" : "ab8a21128911f35f20fcc9c5365c93c1d13b06a31f2538fed1e5f15548fe14a3",
            "value" : 1000000,
            "ergoTree" : "100604000400050004000e20002693cd6c3dc7c156240dd1c7370e50c4d1f84a752c2f74d93a20cc22c2899d0e204759889b16a97b0c7ab5ccb30c7fafb7d9e17fd6dc41ab86ae380784abe03e4cd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7ed938cb2db6308720373030001730493cbc272037305cd7202",
            "assets" : [
              {
                "tokenId" : "01e6498911823f4d36deaf49a964e883b2c4ae2a4530926f18b9c1411ab2a2c2",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "070365a4195d508bb07b740b10a07b4451ee82637582acb5f5cee13c6efacf34c3dd",
              "R5" : "0e209d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b",
              "R6" : "05d0d4affcc4a9ad01"
            },
            "transactionId" : "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
            "index" : 0
          },
          {
            "boxId" : "7a8537700e23539c951f380f455ce58cab45ccd4daf4711fa13532b2f8060ff0",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
            "index" : 1
          },
          {
            "boxId" : "ad73fe5df8dff873832484d78f736dbd1ab5df1e012a756236c8c99613223511",
            "value" : 600000,
            "ergoTree" : "0008cd03553448c194fdd843c87d080f5e8ed983f5bb2807b13b45a9683bba8c7bfb5ae8",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
            "index" : 2
          }
        ],
        "size" : 673
      },
      {
        "id" : "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
        "inputs" : [
          {
            "boxId" : "6747e29538033243f576876c740a263dccb8fae346003c8c9c8495ebdc7a4594",
            "spendingProof" : {
              "proofBytes" : "e87848a97c2b84fb490da538b1e9b688a29d29a4559309ee306a65e035a04c7ce3de833447f06af3f4efd133cadd5696841b78f7f3c2c53e",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "03f214059a610d7c548012d81fc168fc4319296ac7a470ab68ab01d9c77a4288",
            "spendingProof" : {
              "proofBytes" : "83530b19f1b8cf7cbb6f0bfa3790e915b150c4571c08b16910e45afeba768325f80c0558a8ef6cbfa6925f38f02d1014516e44039e955fe9",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
          {
            "boxId" : "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs" : [
          {
            "boxId" : "e6d67fa45224bcb7840ab37dbd31cd8e0796b9981c22348738d683f10f16f301",
            "value" : 1000000,
            "ergoTree" : "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets" : [
              {
                "tokenId" : "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount" : 1
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              "R4" : "07021fab219a58d2e1e8edfd3e2ad7cf09a35687246c084477db0bce5412f43acdbe",
              "R5" : "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6" : "059cb3aadd02"
            },
            "transactionId" : "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
            "index" : 0
          },
          {
            "boxId" : "b546b26276a660013ae70ac937a05d6f4e2be9eda04aff6e06b1d2cfe8b974a8",
            "value" : 1000000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
            "index" : 1
          },
          {
            "boxId" : "4ee7b8d47a65741007bbdd57e3a98826768fdc025c1b991c4db64c6e4b555098",
            "value" : 973000000,
            "ergoTree" : "0008cd021fab219a58d2e1e8edfd3e2ad7cf09a35687246c084477db0bce5412f43acdbe",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
            "index" : 2
          }
        ],
        "size" : 630
      },
      {
        "id" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
        "inputs" : [
          {
            "boxId" : "393b6d3f9407b7bd57cf4f0688af283c27291006b1d06367f040f97bafb93292",
            "spendingProof" : {
              "proofBytes" : "c2faadce21f3449ba7af8008cfda03267d3ce00d870a22fbcdeaa94ac9db1f9b9eba4540c060e8ea1597db1486ac6d6f724e7f693db3ec0d",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "4aba8806c160dde733b75553ea3416768f626036689d63bf6ac9527601a3f29c",
            "spendingProof" : {
              "proofBytes" : "a9213cd0c04e5d3b629928976b19e894de7cbaeb42dc030d686754cce337679f5974f30961a18e9e22f235c0d6a4ffcfc78b4792a150418f",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "ed9b322d4d8fc19c169e6b27d2c51c961d651eb774cb2a7021e19b8a668c671d",
            "spendingProof" : {
              "proofBytes" : "3cc5fb4ad52ef85288438a83310e4923d53132a17d2dd7cc3f3f798c1d631a44f509334a7b9e283ca8c5d573c9e111439110ffd6193466c6",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
        ],
        "outputs" : [
          {
            "boxId" : "10c771c1e82c0adbfdde0cbbb46efbd9b81e601bae23ec58b5a9c76ea79ced55",
            "value" : 500899165,
            "ergoTree" : "0008cd02f57847102bb00c49fc9e282ca309362ccc1cb60a7325eac877a4dfe5429b27f0",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 0
          },
          {
            "boxId" : "c077c8d09ed25c328d7f967930e4e97e34a16b20d62cac2eae9bd5609df55865",
            "value" : 115818869,
            "ergoTree" : "0008cd032fac9ccb7b7eae1154d4fccfd2b0055e480b1e32e32b2a3e2211720d11069998",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 1
          },
          {
            "boxId" : "d90604b060e854a7e2fa09ecf4ff0f72925027d510b4bc83fa4dde56dedda9f3",
            "value" : 103642985,
            "ergoTree" : "0008cd03375b266b2adf40238c5262e7fb3f410258b12bd44aa2d7164c31600df0c800d5",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 2
          },
          {
            "boxId" : "40a9ca07ad8427bbcdb7addda7e97a65f0853e8c8716fe38739b030206b1d14f",
            "value" : 100224481,
            "ergoTree" : "0008cd02d5eb343a08f3aadbeb0ed335f797d281ca84cbc4ebcbf516e811819669ec8519",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 3
          },
          {
            "boxId" : "3a19eb29f4d992c2a1f024fdf368d44bfd3747a59abedc0d62eb3f18ba762e86",
            "value" : 1007967428,
            "ergoTree" : "0008cd030a4d37ec5cb6a162eaebed5b06aaedda8b7bd00f78617602533aff5ece5ff9ae",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 4
          },
          {
            "boxId" : "71530f43997f645b26959aeb5e72150cccf087f31b54e4f7d3d757bea0cd381f",
            "value" : 1101843768,
            "ergoTree" : "0008cd023e240c637fb6db1f216d70be25529a92b3d659aaae720416ab3f31dc062a9c58",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 5
          },
          {
            "boxId" : "20d7ebc8fe8ce92e8ae25bc7d9b8bd594f6583409161f46ded55c5ed017f35e9",
            "value" : 110135512,
            "ergoTree" : "0008cd028de5ba65755a2730d28723ebc1b8b90666e07c241867d36dd454bffea5af5e3d",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 6
          },
          {
            "boxId" : "8ff6199a103ce0a8a6da02f8400fa874ffbc0f40c161bb2a6b92890516478806",
            "value" : 1070860330,
            "ergoTree" : "0008cd0390b635357f495d00e47a02063de1282f6be4368360d11c4d4491d22a9f623a1a",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 7
          },
          {
            "boxId" : "5c491f75860fc54b19ace1c5b347fcec8df1bf7501cb35bfcb2ff9d6af805352",
            "value" : 1003979123,
            "ergoTree" : "0008cd03609f01770f7adafceee903b671397243dc485997da1de583fd705d02866eb388",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 8
          },
          {
            "boxId" : "ceeb01db3787ff8636d097e5629bb335922824974d37f5134b99cb2160d53baa",
            "value" : 1002690045,
            "ergoTree" : "0008cd03b6d232407286f986979ce5b65c62715a5f128100eb595734aafc2a0c8e1d5b97",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 9
          },
          {
            "boxId" : "9b8de79e0de3ccf91720ca1ce5cb1f09f39f103b71f8dc019f23273dcd348b31",
            "value" : 1037676707,
            "ergoTree" : "0008cd035b54389bd7ea7d2e6874e17cacd1d185d4c73dda2a99b53ea4fbaf73c81e80c6",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 10
          },
          {
            "boxId" : "60246e0dabb1447f04b7b528d24072aeed159825f27196849cb8a57703deb0f6",
            "value" : 100499662,
            "ergoTree" : "0008cd031e78a57f892aaac1c1323b6d615ba7febacd9f052ec0ffd673d5e109624bd3ce",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 11
          },
          {
            "boxId" : "27e3e0c12bcfa4fb59a2521a7b2b1cb5bf1a9014cec3ec4dbf9873ee9a0e05ea",
            "value" : 100058510,
            "ergoTree" : "0008cd02dc8ff1ce487528e0942199178c8095302c929dd0346a07fee963f1d95352a7bc",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 12
          },
          {
            "boxId" : "450c3815754971079df2ea8df6f6f10dbb254c316b2acf879f2710bd5b36e52b",
            "value" : 1017105941,
            "ergoTree" : "0008cd02b538841f9e03b8784136267fdfab13a7b3746e63960746b665291e34590d24d6",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 13
          },
          {
            "boxId" : "8afb3977a384a9f23709ae824be188ed8964e96cc6245835eb24c54f016d5c22",
            "value" : 1021790843,
            "ergoTree" : "0008cd036fcd0fc3151ae86ae9ae5ac9bd00148de5c448b0326998e8fe6e25984f00d232",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 14
          },
          {
            "boxId" : "9343586195c06fc58eb229ec9e4b8ce8c3ae3579e28ea3b25fa5dc98e15a4553",
            "value" : 1004118201,
            "ergoTree" : "0008cd0376e48a1be1e32d3f0830ee4ccd6eec75c979a8eccef066531f1e1d96e7e25b49",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 15
          },
          {
            "boxId" : "6e2ef984458fabfd1d32c3f01943a281ab1b1ad98ce1b2fc7bd65020c28ed720",
            "value" : 10033979953,
            "ergoTree" : "0008cd02a06ccbca2250b6d904f2dad206e5a7dd5d387d92d8a1d7e2db357db1b392c128",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 16
          },
          {
            "boxId" : "f24dba2d15493a807f42adf7316fb7df9e976c193b6b69f4d3e1f7ccc59e813c",
            "value" : 1000740079,
            "ergoTree" : "0008cd02526c822c9111f115b79405118294bce173b287dff540a50ee4b61c176c977299",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 17
          },
          {
            "boxId" : "0a951204596dc8a795ca28951c746622f55b855a2ea300397b38f20d636108de",
            "value" : 101396721,
            "ergoTree" : "0008cd02c881b58b9012d829789ec2a072fcfd4148a7f0917a57285a0f2e91e0b48bb07a",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 18
          },
          {
            "boxId" : "c9b37a216d65647b8b23864563823a221decd1b277ddafcf6783c8993cd63507",
            "value" : 1033691668,
            "ergoTree" : "0008cd03e7e50df47f1ff07953c75e7b12552939159a37013af8c9993063e0d1b91b9d22",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 19
          },
          {
            "boxId" : "55002ee68590a54fb8f984a69f5ef196b64ad8ebe374defa68dcaf9c39adcbd4",
            "value" : 106401930,
            "ergoTree" : "0008cd02a92664f7ca68185691cf23cbe9d4412465fe3af64bafc7c328b6d25900ed5a84",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 20
          },
          {
            "boxId" : "e465e8d52d947b5dcc3798766b0f4106ecdf5c5d3d5f02a11c23a9791f0f58f5",
            "value" : 115432395,
            "ergoTree" : "0008cd03376ef34624615a8f03f28161b9be44c0c5754a5cb2dd594129924244a5b48f42",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 21
          },
          {
            "boxId" : "6be6ce5cd1a5ee8c17f1685ad8f0e22d37d2cabc8cf00b5fbc40df88af1b2d80",
            "value" : 1010521524,
            "ergoTree" : "0008cd02afd95b35c437fe539b44f660ee88f15bc6e08ad2214bdb5afa857282da2f7aa3",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 22
          },
          {
            "boxId" : "6d31a003416d56f656cae9e2ccbf4bfd5bd76820543b3719f72193be8b4697dc",
            "value" : 1029866438,
            "ergoTree" : "0008cd03592ee05344456d2717e1810de431b1f078bad845af2f3375e5530199fa387907",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 23
          },
          {
            "boxId" : "055d5247be36ecbdad82167e7cef890d1f3cc8efdb21c3680f8c26f461fbf312",
            "value" : 1009153034,
            "ergoTree" : "0008cd03408d8f0c58972c759ee613e62b2df77ef37114c0d0beb8554e0d1340fe0eadde",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 24
          },
          {
            "boxId" : "b522d732134f15f7f9af286d8aa2e062b8f1cc9f19ff04f69cd4871d83f95bfc",
            "value" : 510420797,
            "ergoTree" : "0008cd026230d8638bb67a7a8a6a0e9c841536488978e57eeae1562f56fd932bd7b51e7c",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 25
          },
          {
            "boxId" : "aaf8f5fdb34f30be10979d20146ee796beb72211a32e5bda3e2b0c0835faa14a",
            "value" : 1011404511,
            "ergoTree" : "0008cd0395ff09b7c98dd1123f9785af34577d2c3a09cab992f20a40ae4be781e8605018",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 26
          },
          {
            "boxId" : "69fdcefb383f972e4058ac13c5afe658792a6902674ca41925e442f2b98632b7",
            "value" : 1100000,
            "ergoTree" : "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets" : [
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 27
          },
          {
            "boxId" : "74c5d8916264e5714903ccf1d0f73cac17846529b3be4f045eff94251961e60b",
            "value" : 11353040072956,
            "ergoTree" : "0008cd029ed28cae37942d25d5cc5f0ade4b1b2e03e18b05c4f3233018bf67c817709f93",
            "assets" : [
              {
                "tokenId" : "d1ae85958e31d24cffde6f09c9d492819fad950dafb76b17edcd80badd6fe8ef",
                "amount" : 1
              },
              {
                "tokenId" : "0cd8c9f416e5b1ca9f986a7f10a84191dfb85941619e49e53c0dc30ebf83324b",
                "amount" : 10000
              }
            ],
            "creationHeight" : 693477,
            "additionalRegisters" : {
              
            },
            "transactionId" : "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index" : 28
          }
        ],
        "size" : 1737
      },
      {
        "id" : "db40eab029fa1dbab09476fbb4342c99bf4c1b07cf5a943c48019702b0e7c16f",
        "inputs" : [
          {
            "boxId" : "00ddbeb981c0b08536f72ea41e07a25adbf7bf104ee59b865619a21676e64715",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "18aaabf889a4d3a111b07a2beb932700cf6e0db4f2c654044aef507f1860449c",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "483f1420cc1afce2ea539075654800516c9a8341a3b41d9ba435ed9f217ceb50",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "5291c872959a81fa1e0f6696b41e911a09be732ab70871d9504a0792249c3633",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "5b44961a26230f4c37dba57125149284fe808986bfb939bf35aefce35eae554a",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "69fdcefb383f972e4058ac13c5afe658792a6902674ca41925e442f2b98632b7",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "7a8537700e23539c951f380f455ce58cab45ccd4daf4711fa13532b2f8060ff0",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "85dc5b034f8815952cdbad90aa7b50b68b56702e76b89ca799e2dc0e47b35eca",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "8847701f6eaa5629ab350f894a626b533bdf83beab6b12c620ad854cb5cad02d",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "90a0ed9da366367c88cca4d8c63bbca4e77f96deef885f48df564f857a78d374",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "9d383716046dab202a2cb6b285ef391b46ccfc32839b427bf1e210374cbfcf59",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "b546b26276a660013ae70ac937a05d6f4e2be9eda04aff6e06b1d2cfe8b974a8",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          },
          {
            "boxId" : "f259d6bf9d4d5564401a2ac4da7e3eabe6c01288a177a23f360b34280fe262cf",
            "spendingProof" : {
              "proofBytes" : "",
              "extension" : {
                
              }
            }
          }
        ],
        "dataInputs" : [
        ],
        "outputs" : [
          {
            "boxId" : "06bb97c824358fe7aec8b91c04587402aa3ec0007934b4a3156b3bffcd8a3356",
            "value" : 22900000,
            "ergoTree" : "100204a00b08cd0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891fea02d192a39a8cc7a70173007301",
            "assets" : [
            ],
            "creationHeight" : 693479,
            "additionalRegisters" : {
              
            },
            "transactionId" : "db40eab029fa1dbab09476fbb4342c99bf4c1b07cf5a943c48019702b0e7c16f",
            "index" : 0
          }
        ],
        "size" : 509
      }
    ],
    "blockVersion" : 2,
    "size" : 11100
  },
  "extension" : {
    "headerId" : "b17847c0c523660b13d707396ab8301fa3c8a545ddc5acf9ec2803cc2cbb3ef5",
    "digest" : "5fb2be1ff25d365daadbb3cd4908feceff097bb4ad8f6c7f1436a04ffa3bf5cd",
    "fields" : [
      [
        "0100",
        "01b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b"
      ],
      [
        "0101",
        "041155d54de65f0130fae142aa4cf5a7728b7c30f5939d33fddf077e2008040a15"
      ],
      [
        "0105",
        "01116a6c1d030c62d333df6d518e26887745e5251d6d2270e5560fe4cce85ad0a3"
      ],
      [
        "0106",
        "015aad19a4b658e59ec098f06c4f0b6f3317b09e6a6fe9e49be340933e709a5a1e"
      ],
      [
        "0107",
        "039501b674e3e4678a659d9abf63c079558305ae1dbc3d5f97cd07195b2423ddd5"
      ],
      [
        "010a",
        "02ed35fa3373a6035aca1552005380ef67f9ce90bcb651f6dbee64db3f1f5efdc9"
      ],
      [
        "010c",
        "02afe4739c3fd01466c309d3b0d27bfdda18c5570ec2ede33b68792f8f8f315be8"
      ],
      [
        "010e",
        "037508c526d5371c95ef344e39b46709ee918b904a91c9f35cb26b6794eab9233f"
      ],
      [
        "0111",
        "02addf788724adae14e0fe75538b91b8bc192564e2355819a2715d376bb1229ee8"
      ],
      [
        "0113",
        "0272a17b1cd5863bbc598f68678a35168ae5e4eadf602edd778eb3b6f7312cdc65"
      ]
    ]
  },
  "adProofs" : null,
  "size" : 11321
}

        "#;

        let block_0: FullBlock = serde_json::from_str(json).unwrap();
        let encoded_json = serde_json::to_string(&block_0).unwrap();
        let block_1: FullBlock = serde_json::from_str(&encoded_json).unwrap();
        assert_eq!(block_0, block_1);
    }

    #[ignore = "remove this test when 677 is closed"]
    #[test]
    fn parse_i677() {
        let json = r#"
{
  "header": {
    "extensionId": "f2b3d4db0504d2df78f661cd980ed7f2b861a2cf9f0acfd1274e4b22bd468abe",
    "difficulty": "1667667081560064",
    "votes": "000000",
    "timestamp": 1645753676103,
    "size": 221,
    "stateRoot": "37e32d56e08d94170f8c54ecee1e65d526841524773c3769ea4aaf8bd014d6c018",
    "height": 693479,
    "nBits": 117828796,
    "version": 2,
    "id": "b17847c0c523660b13d707396ab8301fa3c8a545ddc5acf9ec2803cc2cbb3ef5",
    "adProofsRoot": "1130ce510264cd3707592f14631fbb994cb3624219f159b5cbf3119efcf25b8c",
    "transactionsRoot": "57fb74737b99ea1f57e206ce76743e5970da060fed27cea6c8f50b7d587c8beb",
    "extensionHash": "5fb2be1ff25d365daadbb3cd4908feceff097bb4ad8f6c7f1436a04ffa3bf5cd",
    "powSolutions": {
      "pk": "0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891f",
      "w": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
      "n": "b8e6000bf8e4f5fc",
      "d": 0
    },
    "adProofsId": "30cf0f1b493a636b4b5554937424dffaaf04b59af6e4a35862031c2c2a8b549e",
    "transactionsId": "f90ad5454590737a1a54e2b28524db8ba94cc0c14eaf43533cda77acedde5bb9",
    "parentId": "72a17b1cd5863bbc598f68678a35168ae5e4eadf602edd778eb3b6f7312cdc65"
  },
  "blockTransactions": {
    "headerId": "b17847c0c523660b13d707396ab8301fa3c8a545ddc5acf9ec2803cc2cbb3ef5",
    "transactions": [
      {
        "id": "ba3f7ac52edb58663f976cf4b390cf57b8e16fc51488c0a944446e5eaa733982",
        "inputs": [
          {
            "boxId": "5616b8101a8600ebddf33b55c34554f3b704fc9bf5b7b47b2523a66d1f072e28",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          }
        ],
        "dataInputs": [],
        "outputs": [
          {
            "boxId": "e34e00e645d12af74f82c74ded1ae3718004bef594371b760c75e630f036b86c",
            "value": 46656720000000000,
            "ergoTree": "101004020e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a7017300730110010204020404040004c0fd4f05808c82f5f6030580b8c9e5ae040580f882ad16040204c0944004c0f407040004000580f882ad16d19683030191a38cc7a7019683020193c2b2a57300007473017302830108cdeeac93a38cc7b2a573030001978302019683040193b1a5730493c2a7c2b2a573050093958fa3730673079973089c73097e9a730a9d99a3730b730c0599c1a7c1b2a5730d00938cc7b2a5730e0001a390c1a7730f",
            "assets": [],
            "creationHeight": 693479,
            "additionalRegisters": {},
            "transactionId": "ba3f7ac52edb58663f976cf4b390cf57b8e16fc51488c0a944446e5eaa733982",
            "index": 0
          },
          {
            "boxId": "9aae966e3e23a0b45f50b373c5d2e7de036a061e36fb63de8dde4f462f329427",
            "value": 66000000000,
            "ergoTree": "100204a00b08cd0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891fea02d192a39a8cc7a70173007301",
            "assets": [],
            "creationHeight": 693479,
            "additionalRegisters": {},
            "transactionId": "ba3f7ac52edb58663f976cf4b390cf57b8e16fc51488c0a944446e5eaa733982",
            "index": 1
          }
        ],
        "size": 344
      },
      {
        "id": "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
        "inputs": [
          {
            "boxId": "59f2856068c56264d290520043044ace138a3a80d414748d0e4dcd0806188546",
            "spendingProof": {
              "proofBytes": "",
              "extension": {
                "0": "04c60f",
                "5": "0514",
                "10": "0eee03101808cd0279aed8dea2b2a25316d5d49d13bf51c0b2c1dc696974bb4b0c07b5894e998e56040005e0e0a447040404060402040004000e201d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f0e20003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d005e201058c85a2010514040404c60f06010104d00f05e0e0a44704c60f0e691005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304050005000580ade2040100d803d6017300d602b2a4730100d6037302eb027201d195ed93b1a4730393b1db630872027304d804d604db63087202d605b2a5730500d606b2db63087205730600d6077e8c72060206edededededed938cb2720473070001730893c27205d07201938c72060173099272077e730a06927ec172050699997ec1a7069d9c72077e730b067e730c067e720306909c9c7e8cb27204730d0002067e7203067e730e069c9a7207730f9a9c7ec17202067e7310067e9c73117e7312050690b0ada5d90108639593c272087313c1720873147315d90108599a8c7208018c72080273167317",
                "1": "0e20003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d0",
                "6": "0580ade204",
                "9": "0580b48913",
                "2": "05e201",
                "7": "0e201d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f",
                "3": "05e0e0a447",
                "8": "0580ade204",
                "4": "058c85a201"
              }
            }
          }
        ],
        "dataInputs": [],
        "outputs": [
          {
            "boxId": "0586c90d0cf6a82dab48c6a79500364ddbd6f81705f5032b03aa287de43dc638",
            "value": 94750000,
            "ergoTree": "101808cd0279aed8dea2b2a25316d5d49d13bf51c0b2c1dc696974bb4b0c07b5894e998e56040005e0e0a447040404060402040004000e201d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f0e20003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d005e201058c85a2010514040404c60f06010104d00f05e0e0a44704c60f0e691005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304050005000580ade2040100d803d6017300d602b2a4730100d6037302eb027201d195ed93b1a4730393b1db630872027304d804d604db63087202d605b2a5730500d606b2db63087205730600d6077e8c72060206edededededed938cb2720473070001730893c27205d07201938c72060173099272077e730a06927ec172050699997ec1a7069d9c72077e730b067e730c067e720306909c9c7e8cb27204730d0002067e7203067e730e069c9a7207730f9a9c7ec17202067e7310067e9c73117e7312050690b0ada5d90108639593c272087313c1720873147315d90108599a8c7208018c72080273167317",
            "assets": [],
            "creationHeight": 693475,
            "additionalRegisters": {},
            "transactionId": "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
            "index": 0
          },
          {
            "boxId": "4b99c2ef8496a491d176ecaf789d9e1d9aad0c2bf3e70b32e8bad73f48c722b9",
            "value": 250000,
            "ergoTree": "100e04000500059a0505d00f04020404040608cd03c6543ac8e8059748b1c6209ee419dd49a19ffaf5712a2f34a9412016a3a1d96708cd035b736bebf0c5393f78329f6894af84d1864c7496cc65ddc250ef60cdd75df52008cd021b63e19ab452c84cdc6687242e8494957b1f11e3750c8c184a8425f8a8171d9b05060580ade2040580a8d6b907040ad806d601b2a5730000d602b0a47301d9010241639a8c720201c18c720202d6039d9c730272027303d604b2a5730400d605b2a5730500d606b2a5730600d1968306019683020193c17201720393c27201d073079683020193c17204720393c27204d073089683020193c17205720393c27205d073099683020192c17206999972029c730a7203730b93c2a7c27206927202730c93b1a5730d",
            "assets": [],
            "creationHeight": 693475,
            "additionalRegisters": {},
            "transactionId": "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
            "index": 1
          },
          {
            "boxId": "00ddbeb981c0b08536f72ea41e07a25adbf7bf104ee59b865619a21676e64715",
            "value": 5000000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693475,
            "additionalRegisters": {},
            "transactionId": "c8520befd345ff40fcf244b44ffe8cea29c8b116b174cfaf4f2a521604d531a4",
            "index": 2
          }
        ],
        "size": 1562
      },
      {
        "id": "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
        "inputs": [
          {
            "boxId": "dded9c2fb796044d39a52e9925196a031fb8ef6ff521e6754d0b6f330e7ece25",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "0586c90d0cf6a82dab48c6a79500364ddbd6f81705f5032b03aa287de43dc638",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          }
        ],
        "dataInputs": [],
        "outputs": [
          {
            "boxId": "2b2c6cffdd5cb315bfc88830124b285633d2b4edff165a638ef7ac17aa45e084",
            "value": 67333814889739,
            "ergoTree": "1999030f0400040204020404040405feffffffffffffffff0105feffffffffffffffff01050004d00f040004000406050005000580dac409d819d601b2a5730000d602e4c6a70404d603db63087201d604db6308a7d605b27203730100d606b27204730200d607b27203730300d608b27204730400d6099973058c720602d60a999973068c7205027209d60bc17201d60cc1a7d60d99720b720cd60e91720d7307d60f8c720802d6107e720f06d6117e720d06d612998c720702720fd6137e720c06d6147308d6157e721206d6167e720a06d6177e720906d6189c72117217d6199c72157217d1ededededededed93c27201c2a793e4c672010404720293b27203730900b27204730a00938c7205018c720601938c7207018c72080193b17203730b9593720a730c95720e929c9c721072117e7202069c7ef07212069a9c72137e7214067e9c720d7e72020506929c9c721372157e7202069c7ef0720d069a9c72107e7214067e9c72127e7202050695ed720e917212730d907216a19d721872139d72197210ed9272189c721672139272199c7216721091720b730e",
            "assets": [
              {
                "tokenId": "1d5afc59838920bb5ef2a8f9d63825a55b1d48e269d7cecee335d637c3ff5f3f",
                "amount": 1
              },
              {
                "tokenId": "fa6326a26334f5e933b96470b53b45083374f71912b0d7597f00c2c7ebeb5da6",
                "amount": 9223371996546264000
              },
              {
                "tokenId": "003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d0",
                "amount": 105824543
              }
            ],
            "creationHeight": 0,
            "additionalRegisters": {
              "R4": "04c60f"
            },
            "transactionId": "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index": 0
          },
          {
            "boxId": "e7a483349c1737eedcebe5b7bb2e8efed95774d938562ab77626d9e891895932",
            "value": 4601812,
            "ergoTree": "0008cd0279aed8dea2b2a25316d5d49d13bf51c0b2c1dc696974bb4b0c07b5894e998e56",
            "assets": [
              {
                "tokenId": "003bd19d0187117f130b62e1bcab0939929ff5c7709f843c5c4dd158949285d0",
                "amount": 116
              }
            ],
            "creationHeight": 0,
            "additionalRegisters": {},
            "transactionId": "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index": 1
          },
          {
            "boxId": "805f9337e55365284e1ba736ab0ef30f9f0667d009d424dfb56e4dcc64351be4",
            "value": 10398188,
            "ergoTree": "0008cd0273cbc003da723c0a5f416929692e8ec8c2b1e0d9aed69ff681f7581c63e70309",
            "assets": [],
            "creationHeight": 0,
            "additionalRegisters": {},
            "transactionId": "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index": 2
          },
          {
            "boxId": "5291c872959a81fa1e0f6696b41e911a09be732ab70871d9504a0792249c3633",
            "value": 5000000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 0,
            "additionalRegisters": {},
            "transactionId": "7467d1dbc527d4b94db416c6e2ee6b9fdc5bd59067247e143d6241a929f2720f",
            "index": 3
          }
        ],
        "size": 810
      },
      {
        "id": "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
        "inputs": [
          {
            "boxId": "374facc223ff4984b1d5dd295392892a84060f706a26c8212d7daf5abd814e48",
            "spendingProof": {
              "proofBytes": "0af556eefd5d0a03bc09313ad8642291b20bd10304cf3f36858b713c77645bd38201fde1a24e0a4e18055354c18426f5241ce650980dd097",
              "extension": {}
            }
          },
          {
            "boxId": "6407455a1805553de3f68894b046bd4d52a63235148f8284e74b7ab1a0641aa0",
            "spendingProof": {
              "proofBytes": "bec07fd4f046702e9fab5c0b8032cb5c9aca273fd1181c51c397c522d1680a286b12609d2b52209dd61c92e7ab98f99e5de2846cc9c057a4",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs": [
          {
            "boxId": "a26a61e76d6430e88e3cac9314ca7e0fe203357d0d1d6686471c3d411774d242",
            "value": 1000000,
            "ergoTree": "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets": [
              {
                "tokenId": "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "0702725e8878d5198ca7f5853dddf35560ddab05ab0a26adae7e664b84162c9962e5",
              "R5": "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6": "059cb3aadd02"
            },
            "transactionId": "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
            "index": 0
          },
          {
            "boxId": "8847701f6eaa5629ab350f894a626b533bdf83beab6b12c620ad854cb5cad02d",
            "value": 2100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
            "index": 1
          },
          {
            "boxId": "bff4862131e88d6ac780398e2b7da89aa5dbb478a22aa319af41c61e27bb4167",
            "value": 516400000,
            "ergoTree": "0008cd02725e8878d5198ca7f5853dddf35560ddab05ab0a26adae7e664b84162c9962e5",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "e2a4e542c75efcc3b57127b2f1bdab02b288940e18b85ecd0fde5affaf6bde6d",
            "index": 2
          }
        ],
        "size": 631
      },
      {
        "id": "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
        "inputs": [
          {
            "boxId": "9f7bffab806cecc0e5dd95f63d654eeb003fcee36bdf5e602576a6907d482b55",
            "spendingProof": {
              "proofBytes": "067713cc8d22ac6c6dfa0870e87c3d92c35ce31d7d4e87321b0d1af453775e5f829be60e6ef30e1d019b4483654c9631c9450b495a4dc8a7",
              "extension": {}
            }
          }
        ],
        "dataInputs": [],
        "outputs": [
          {
            "boxId": "5761b7fe988db401551ab777fbe00937df93c72d886221851874b32ebfa1c3f2",
            "value": 1000000,
            "ergoTree": "0008cd03a3591b90c96a48923dae01861e14f48419e17ab13f1d36823b6914d4d656bd1f",
            "assets": [
              {
                "tokenId": "472c3d4ecaa08fb7392ff041ee2e6af75f4a558810a74b28600549d5392810e8",
                "amount": 47696000000
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
            "index": 0
          },
          {
            "boxId": "90a0ed9da366367c88cca4d8c63bbca4e77f96deef885f48df564f857a78d374",
            "value": 1000000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
            "index": 1
          },
          {
            "boxId": "caa6c72765455b673f753274507bde0548b5b5c6f1d5fbb21b7d93b8ec18e7b9",
            "value": 366775815551,
            "ergoTree": "0008cd027163c8e38f64bb6df20679c26b81518a49603be43e6691ca798b6baa003abc19",
            "assets": [
              {
                "tokenId": "472c3d4ecaa08fb7392ff041ee2e6af75f4a558810a74b28600549d5392810e8",
                "amount": 100000000000
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "b8be938b914038563860e80dcb07bd22fcb8ed389f2f15693c582552563ad91f",
            "index": 2
          }
        ],
        "size": 344
      },
      {
        "id": "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
        "inputs": [
          {
            "boxId": "df1a50bddb2afb70479f130292a69bf1df1524e44442b5f3e3cfd2452aa6de81",
            "spendingProof": {
              "proofBytes": "91027169db56dc1d98a960d05f698d88a409108c079143bed6677ece572d0c0bee4e90a2160327151de78cdd773c08b8e856af9fd79a6085",
              "extension": {}
            }
          },
          {
            "boxId": "0f7ce74d938750dee1fb47709ab27901810123a8d25e765fec5f60a8e283e342",
            "spendingProof": {
              "proofBytes": "5535dddd86417e56ec61a217ab9355121703a6243c529bba918b6dc27b69e455b6affc7154b816a21ceb7d3ee158dbcd04ae7e13a65dd9dc",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs": [
          {
            "boxId": "31b9ee4ae67f421e16d38f1513ad06994539e65d8ac0a7e1d4141c35e256001b",
            "value": 1000000,
            "ergoTree": "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets": [
              {
                "tokenId": "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "0702c1d434dac8765fc1269af82958d8aa350da53907096b35f7747cc372a7e6e69d",
              "R5": "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6": "059cb3aadd02"
            },
            "transactionId": "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
            "index": 0
          },
          {
            "boxId": "85dc5b034f8815952cdbad90aa7b50b68b56702e76b89ca799e2dc0e47b35eca",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
            "index": 1
          },
          {
            "boxId": "f6558bcfac2b03facf55b2ad2714cd7eefe51d3a90ead323fc2860d4e101270c",
            "value": 9347700000,
            "ergoTree": "0008cd02c1d434dac8765fc1269af82958d8aa350da53907096b35f7747cc372a7e6e69d",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "12fce6c883e4afacb8812438dba6740d14270cbb479d2385c22d32558734d556",
            "index": 2
          }
        ],
        "size": 630
      },
      {
        "id": "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
        "inputs": [
          {
            "boxId": "57e3f78aa182e5d575625b0832b77e0a31caf52ed1f79a093f1f51602d75bb7f",
            "spendingProof": {
              "proofBytes": "826b43caead76bc2a1f158e42c1f372bdbc481dad2fd05889d0713b9c68457c4ec018107460203e932963bb16465f35cf71b3687fe0f1b85",
              "extension": {}
            }
          },
          {
            "boxId": "4f6d379aefb353cdf32ffa6ad65294b6460ea20b8628b46ea3743cf519f5f738",
            "spendingProof": {
              "proofBytes": "d1579debf190477899a1d889441b62003f272b3d53e9013d484f02ede6cbe91e673d6c984be3f5d586c99e82540fb2c67b2a46b439330615",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs": [
          {
            "boxId": "b43ba3e7dfdc766f48b434d9b7007e4c5f496a6bec4bc7298f33c8f8bebbcf5b",
            "value": 1000000,
            "ergoTree": "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets": [
              {
                "tokenId": "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "070331b99a9fcc7bceb0a238446cdab944402dd4b2e79f9dcab898ec3b46aea285c8",
              "R5": "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6": "059cb3aadd02"
            },
            "transactionId": "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
            "index": 0
          },
          {
            "boxId": "5b44961a26230f4c37dba57125149284fe808986bfb939bf35aefce35eae554a",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
            "index": 1
          },
          {
            "boxId": "1bd0d3cadfd72d823e98880ddfcee15981c1d6bcf90725e72186744f5377a7ce",
            "value": 9300400000,
            "ergoTree": "0008cd0333920f80ca39477cb57ccdff9847ed6cbd46cf2c7237b6b085979622349910e9",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "1a975484ce1c0fc9823fcef3aeb9be3fcb9d79fbfc4e537782f2a7abc08c7bd5",
            "index": 2
          }
        ],
        "size": 630
      },
      {
        "id": "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
        "inputs": [
          {
            "boxId": "1574df6b63c6794a981f112c6accad303322a0322502286498c9d44b243b0d5b",
            "spendingProof": {
              "proofBytes": "4d28e70c901fb884e9969135096c46f3154a791b159b78b3c83b42d489efc5b1dff6a8337b2be141b6eac9a20618256f03f1da5a8bdcf85a",
              "extension": {}
            }
          },
          {
            "boxId": "b2fbe2d21b2b994f62faf4640a90c2b85842f0ea5430abc19dbe012cb449cb56",
            "spendingProof": {
              "proofBytes": "f1e40312ecd9d8c1c7dc4f3ddbd7a20c4cd44acfd22ce3daeaa24d109ea7cb1d9d94ed89658d267bf1001c54b90907838784e9c37fe11bc1",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs": [
          {
            "boxId": "0c3c66a0cdff6b3a1c6c3a698655fa92143c96b83465b25c7a64af3a8594dd3a",
            "value": 1000000,
            "ergoTree": "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets": [
              {
                "tokenId": "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "070274524ee849e4e45f58c46164ac609902bb374fc9375f097ee1af2ef1152ab9bf",
              "R5": "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6": "059cb3aadd02"
            },
            "transactionId": "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
            "index": 0
          },
          {
            "boxId": "9d383716046dab202a2cb6b285ef391b46ccfc32839b427bf1e210374cbfcf59",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
            "index": 1
          },
          {
            "boxId": "056d67b049020bf92923c752df9f363a1c41ffe0b2e2bfafbb86d63e62538558",
            "value": 9259700000,
            "ergoTree": "0008cd0274524ee849e4e45f58c46164ac609902bb374fc9375f097ee1af2ef1152ab9bf",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "3a8592eb9959b8a5c233893dca6665e0e3c979f9bc8489825e2b1790978be978",
            "index": 2
          }
        ],
        "size": 630
      },
      {
        "id": "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
        "inputs": [
          {
            "boxId": "d811aff2532aa4e3c871fe3ce1367b3fbef24b686b425d0134274cb84a119bd0",
            "spendingProof": {
              "proofBytes": "fe48e6ab221c92d27d027492b2083c0d2385b8f609de13e3906a10e5c0279d3e212ed575f5980c8b091b9e6ebc6ccd51d8b2f245df0273ce",
              "extension": {}
            }
          },
          {
            "boxId": "0c5deb864523818625329ba7e991dd3ebf0eb5dc9edc48dcbb19f2ac16b7eefe",
            "spendingProof": {
              "proofBytes": "7670d50456b7300a4755d9266d804d5b4377d13149311a9e81ac6ea25713e64d81e894ec77e73cc9a4fff698478df06cbdc1785662dd9858",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs": [
          {
            "boxId": "09b804f8d925b9a279143c3bfbd6878024ae64fffaa5c2590f1987b6b78bdd82",
            "value": 1000000,
            "ergoTree": "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets": [
              {
                "tokenId": "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "0702caad8ef6771ad15ebb0a2aa9b7e84b9c48962976061d1af3e73767203d2f2bb1",
              "R5": "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6": "059cb3aadd02"
            },
            "transactionId": "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
            "index": 0
          },
          {
            "boxId": "18aaabf889a4d3a111b07a2beb932700cf6e0db4f2c654044aef507f1860449c",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
            "index": 1
          },
          {
            "boxId": "279d6975dc591abd96fb18460808ac5d6f46e9c9e228ba40f4dc3f6dd7e7d2c6",
            "value": 10253450000,
            "ergoTree": "0008cd02caad8ef6771ad15ebb0a2aa9b7e84b9c48962976061d1af3e73767203d2f2bb1",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "acefa286100a0ebe3b8c140ca7bdcaf6064bd445943cbd61301add6d35e7bc3a",
            "index": 2
          }
        ],
        "size": 630
      },
      {
        "id": "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
        "inputs": [
          {
            "boxId": "0cd3c09ac65fa259ddc68c93dcdf78395650f05b3df046241117df23a5580f92",
            "spendingProof": {
              "proofBytes": "397cf85ba400a931ba146d971df08b624a10f5f7aa16a065b09d1185af7840519d21772ce8b905daad0e8e75abbfea348b75cb1fa76121cf",
              "extension": {}
            }
          },
          {
            "boxId": "90007c1f6b06a013366556c0e450bc4766ac5d0c09a2d99727c64201add1b144",
            "spendingProof": {
              "proofBytes": "cacb926366f7845b24d1b1cec05b9a9e022fa9fccb2b828ec9971511b3c5e191cdd0308b1f62e564fbadcb49d1614b7100a85c6407f1c383",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs": [
          {
            "boxId": "5e164aaaf0c1a2ef7c15332df4813bfcf8309c8baabeb75a711f4b707558007f",
            "value": 1000000,
            "ergoTree": "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets": [
              {
                "tokenId": "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "0703082348fd5d0c27d7aa89cd460a58fea2932f12147a04985e500bd9ad64695d58",
              "R5": "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6": "059cb3aadd02"
            },
            "transactionId": "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
            "index": 0
          },
          {
            "boxId": "f259d6bf9d4d5564401a2ac4da7e3eabe6c01288a177a23f360b34280fe262cf",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
            "index": 1
          },
          {
            "boxId": "03ca62471d9ac5dd9fd67ac8d895f6f868957dad07ce3d796b5ac80590bbe5e9",
            "value": 6802400000,
            "ergoTree": "0008cd03082348fd5d0c27d7aa89cd460a58fea2932f12147a04985e500bd9ad64695d58",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "cde040fd056c6fc7003f702d94224626f297e615948b14d0398c1ff5043c7e67",
            "index": 2
          }
        ],
        "size": 630
      },
      {
        "id": "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
        "inputs": [
          {
            "boxId": "aef21868eb2055412ae6f5dbc74d9ff6024a6b68a21015703ae8ed580d463ea9",
            "spendingProof": {
              "proofBytes": "cab2409bc996779a18b89f4b9a75a7a2192046195fef7233d2c3070acf60dbb497c34cfe3394e6f7a3dba6754120f0e768b4ab5121c46f10",
              "extension": {}
            }
          },
          {
            "boxId": "5ce7e907eab35d5b01d84c1baf02bf2f08cb480bcb51d04bd8a0489945b62bd0",
            "spendingProof": {
              "proofBytes": "c04f3299d5eca7f98c8c05aeff1874351dfb5d3149d07e194324b881a6692c843a2aad1291d7773f426a7dfe7d86dc53276a99de97b0b976",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "9d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b"
          }
        ],
        "outputs": [
          {
            "boxId": "f59f9be73b4983f55f71556a08fd989152a46b3147c6a8fffd3d85974b8ea1aa",
            "value": 1000000,
            "ergoTree": "100604000400050004000e20002693cd6c3dc7c156240dd1c7370e50c4d1f84a752c2f74d93a20cc22c2899d0e204759889b16a97b0c7ab5ccb30c7fafb7d9e17fd6dc41ab86ae380784abe03e4cd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7ed938cb2db6308720373030001730493cbc272037305cd7202",
            "assets": [
              {
                "tokenId": "01e6498911823f4d36deaf49a964e883b2c4ae2a4530926f18b9c1411ab2a2c2",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "0703a7405d595770313bae0b88f97cf0543750df771f0d183283a4b0f86127ad4f29",
              "R5": "0e209d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b",
              "R6": "05d0d4affcc4a9ad01"
            },
            "transactionId": "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
            "index": 0
          },
          {
            "boxId": "483f1420cc1afce2ea539075654800516c9a8341a3b41d9ba435ed9f217ceb50",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
            "index": 1
          },
          {
            "boxId": "fa43dd7eb02e0a04132bac2dc1da2fecc715bfdfc7ffdfa218566c5667a8ae38",
            "value": 1400000,
            "ergoTree": "0008cd03a7405d595770313bae0b88f97cf0543750df771f0d183283a4b0f86127ad4f29",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "093b87a69800612b3f24c2507f7cee2e892c444aa85b60720b4b22dd76420503",
            "index": 2
          }
        ],
        "size": 673
      },
      {
        "id": "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
        "inputs": [
          {
            "boxId": "15f172920d10dffdaa1ef1e949ce00b481189c76e7e413850f4cf3253d3f3ce9",
            "spendingProof": {
              "proofBytes": "f0066c7cad01b57cff3b88cec0bfc6eed1d2c345659e56749fb0c0b6ae3bd5393cf97dfcab3d270a3d82d01062172e4f06f3a8b8c98fe7e8",
              "extension": {}
            }
          },
          {
            "boxId": "4f0a60665a97930e251c7a0b0ad14a3f011e98e4c6e3b18ce29a2e441abfaa26",
            "spendingProof": {
              "proofBytes": "64a1feed5111f18cd06a087043b7dd72fe656ed0f1cb376e88da7560fc13ef390c85418fec5bf4a2d69e84ae300f351fcf0599e43e699cc0",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "9d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b"
          }
        ],
        "outputs": [
          {
            "boxId": "ab8a21128911f35f20fcc9c5365c93c1d13b06a31f2538fed1e5f15548fe14a3",
            "value": 1000000,
            "ergoTree": "100604000400050004000e20002693cd6c3dc7c156240dd1c7370e50c4d1f84a752c2f74d93a20cc22c2899d0e204759889b16a97b0c7ab5ccb30c7fafb7d9e17fd6dc41ab86ae380784abe03e4cd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7ed938cb2db6308720373030001730493cbc272037305cd7202",
            "assets": [
              {
                "tokenId": "01e6498911823f4d36deaf49a964e883b2c4ae2a4530926f18b9c1411ab2a2c2",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "070365a4195d508bb07b740b10a07b4451ee82637582acb5f5cee13c6efacf34c3dd",
              "R5": "0e209d79024a4531a6381aa10a04dbd34d5e91abe415ba917f2fea3a373b6804b90b",
              "R6": "05d0d4affcc4a9ad01"
            },
            "transactionId": "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
            "index": 0
          },
          {
            "boxId": "7a8537700e23539c951f380f455ce58cab45ccd4daf4711fa13532b2f8060ff0",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
            "index": 1
          },
          {
            "boxId": "ad73fe5df8dff873832484d78f736dbd1ab5df1e012a756236c8c99613223511",
            "value": 600000,
            "ergoTree": "0008cd03553448c194fdd843c87d080f5e8ed983f5bb2807b13b45a9683bba8c7bfb5ae8",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "cbd214e64a3c374e8952cde3115b6467c07af4fd8d9ed71e329882985d0f4d51",
            "index": 2
          }
        ],
        "size": 673
      },
      {
        "id": "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
        "inputs": [
          {
            "boxId": "6747e29538033243f576876c740a263dccb8fae346003c8c9c8495ebdc7a4594",
            "spendingProof": {
              "proofBytes": "e87848a97c2b84fb490da538b1e9b688a29d29a4559309ee306a65e035a04c7ce3de833447f06af3f4efd133cadd5696841b78f7f3c2c53e",
              "extension": {}
            }
          },
          {
            "boxId": "03f214059a610d7c548012d81fc168fc4319296ac7a470ab68ab01d9c77a4288",
            "spendingProof": {
              "proofBytes": "83530b19f1b8cf7cbb6f0bfa3790e915b150c4571c08b16910e45afeba768325f80c0558a8ef6cbfa6925f38f02d1014516e44039e955fe9",
              "extension": {}
            }
          }
        ],
        "dataInputs": [
          {
            "boxId": "470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5"
          }
        ],
        "outputs": [
          {
            "boxId": "e6d67fa45224bcb7840ab37dbd31cd8e0796b9981c22348738d683f10f16f301",
            "value": 1000000,
            "ergoTree": "100504000400050004000e20011d3364de07e5a26f0c4eef0852cddb387039a921b7154ef3cab22c6eda887fd803d601b2a5730000d602e4c6a70407d603b2db6501fe730100ea02d1ededededed93e4c672010407720293e4c67201050ec5720391e4c672010605730293c27201c2a793db63087201db6308a7938cb2db63087203730300017304cd7202",
            "assets": [
              {
                "tokenId": "8c27dd9d8a35aac1e3167d58858c0a8b4059b277da790552e37eba22df9b9035",
                "amount": 1
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {
              "R4": "07021fab219a58d2e1e8edfd3e2ad7cf09a35687246c084477db0bce5412f43acdbe",
              "R5": "0e20470cc080b181520d18ca6cf1d5f451b83f0df68f638c0debd2303a88203a57a5",
              "R6": "059cb3aadd02"
            },
            "transactionId": "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
            "index": 0
          },
          {
            "boxId": "b546b26276a660013ae70ac937a05d6f4e2be9eda04aff6e06b1d2cfe8b974a8",
            "value": 1000000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
            "index": 1
          },
          {
            "boxId": "4ee7b8d47a65741007bbdd57e3a98826768fdc025c1b991c4db64c6e4b555098",
            "value": 973000000,
            "ergoTree": "0008cd021fab219a58d2e1e8edfd3e2ad7cf09a35687246c084477db0bce5412f43acdbe",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "90307f9d38cf2dab5605b154895ac4862fdafaa5236c4582fb04f14525ce0fd1",
            "index": 2
          }
        ],
        "size": 630
      },
      {
        "id": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
        "inputs": [
          {
            "boxId": "393b6d3f9407b7bd57cf4f0688af283c27291006b1d06367f040f97bafb93292",
            "spendingProof": {
              "proofBytes": "c2faadce21f3449ba7af8008cfda03267d3ce00d870a22fbcdeaa94ac9db1f9b9eba4540c060e8ea1597db1486ac6d6f724e7f693db3ec0d",
              "extension": {}
            }
          },
          {
            "boxId": "4aba8806c160dde733b75553ea3416768f626036689d63bf6ac9527601a3f29c",
            "spendingProof": {
              "proofBytes": "a9213cd0c04e5d3b629928976b19e894de7cbaeb42dc030d686754cce337679f5974f30961a18e9e22f235c0d6a4ffcfc78b4792a150418f",
              "extension": {}
            }
          },
          {
            "boxId": "ed9b322d4d8fc19c169e6b27d2c51c961d651eb774cb2a7021e19b8a668c671d",
            "spendingProof": {
              "proofBytes": "3cc5fb4ad52ef85288438a83310e4923d53132a17d2dd7cc3f3f798c1d631a44f509334a7b9e283ca8c5d573c9e111439110ffd6193466c6",
              "extension": {}
            }
          }
        ],
        "dataInputs": [],
        "outputs": [
          {
            "boxId": "10c771c1e82c0adbfdde0cbbb46efbd9b81e601bae23ec58b5a9c76ea79ced55",
            "value": 500899165,
            "ergoTree": "0008cd02f57847102bb00c49fc9e282ca309362ccc1cb60a7325eac877a4dfe5429b27f0",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 0
          },
          {
            "boxId": "c077c8d09ed25c328d7f967930e4e97e34a16b20d62cac2eae9bd5609df55865",
            "value": 115818869,
            "ergoTree": "0008cd032fac9ccb7b7eae1154d4fccfd2b0055e480b1e32e32b2a3e2211720d11069998",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 1
          },
          {
            "boxId": "d90604b060e854a7e2fa09ecf4ff0f72925027d510b4bc83fa4dde56dedda9f3",
            "value": 103642985,
            "ergoTree": "0008cd03375b266b2adf40238c5262e7fb3f410258b12bd44aa2d7164c31600df0c800d5",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 2
          },
          {
            "boxId": "40a9ca07ad8427bbcdb7addda7e97a65f0853e8c8716fe38739b030206b1d14f",
            "value": 100224481,
            "ergoTree": "0008cd02d5eb343a08f3aadbeb0ed335f797d281ca84cbc4ebcbf516e811819669ec8519",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 3
          },
          {
            "boxId": "3a19eb29f4d992c2a1f024fdf368d44bfd3747a59abedc0d62eb3f18ba762e86",
            "value": 1007967428,
            "ergoTree": "0008cd030a4d37ec5cb6a162eaebed5b06aaedda8b7bd00f78617602533aff5ece5ff9ae",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 4
          },
          {
            "boxId": "71530f43997f645b26959aeb5e72150cccf087f31b54e4f7d3d757bea0cd381f",
            "value": 1101843768,
            "ergoTree": "0008cd023e240c637fb6db1f216d70be25529a92b3d659aaae720416ab3f31dc062a9c58",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 5
          },
          {
            "boxId": "20d7ebc8fe8ce92e8ae25bc7d9b8bd594f6583409161f46ded55c5ed017f35e9",
            "value": 110135512,
            "ergoTree": "0008cd028de5ba65755a2730d28723ebc1b8b90666e07c241867d36dd454bffea5af5e3d",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 6
          },
          {
            "boxId": "8ff6199a103ce0a8a6da02f8400fa874ffbc0f40c161bb2a6b92890516478806",
            "value": 1070860330,
            "ergoTree": "0008cd0390b635357f495d00e47a02063de1282f6be4368360d11c4d4491d22a9f623a1a",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 7
          },
          {
            "boxId": "5c491f75860fc54b19ace1c5b347fcec8df1bf7501cb35bfcb2ff9d6af805352",
            "value": 1003979123,
            "ergoTree": "0008cd03609f01770f7adafceee903b671397243dc485997da1de583fd705d02866eb388",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 8
          },
          {
            "boxId": "ceeb01db3787ff8636d097e5629bb335922824974d37f5134b99cb2160d53baa",
            "value": 1002690045,
            "ergoTree": "0008cd03b6d232407286f986979ce5b65c62715a5f128100eb595734aafc2a0c8e1d5b97",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 9
          },
          {
            "boxId": "9b8de79e0de3ccf91720ca1ce5cb1f09f39f103b71f8dc019f23273dcd348b31",
            "value": 1037676707,
            "ergoTree": "0008cd035b54389bd7ea7d2e6874e17cacd1d185d4c73dda2a99b53ea4fbaf73c81e80c6",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 10
          },
          {
            "boxId": "60246e0dabb1447f04b7b528d24072aeed159825f27196849cb8a57703deb0f6",
            "value": 100499662,
            "ergoTree": "0008cd031e78a57f892aaac1c1323b6d615ba7febacd9f052ec0ffd673d5e109624bd3ce",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 11
          },
          {
            "boxId": "27e3e0c12bcfa4fb59a2521a7b2b1cb5bf1a9014cec3ec4dbf9873ee9a0e05ea",
            "value": 100058510,
            "ergoTree": "0008cd02dc8ff1ce487528e0942199178c8095302c929dd0346a07fee963f1d95352a7bc",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 12
          },
          {
            "boxId": "450c3815754971079df2ea8df6f6f10dbb254c316b2acf879f2710bd5b36e52b",
            "value": 1017105941,
            "ergoTree": "0008cd02b538841f9e03b8784136267fdfab13a7b3746e63960746b665291e34590d24d6",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 13
          },
          {
            "boxId": "8afb3977a384a9f23709ae824be188ed8964e96cc6245835eb24c54f016d5c22",
            "value": 1021790843,
            "ergoTree": "0008cd036fcd0fc3151ae86ae9ae5ac9bd00148de5c448b0326998e8fe6e25984f00d232",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 14
          },
          {
            "boxId": "9343586195c06fc58eb229ec9e4b8ce8c3ae3579e28ea3b25fa5dc98e15a4553",
            "value": 1004118201,
            "ergoTree": "0008cd0376e48a1be1e32d3f0830ee4ccd6eec75c979a8eccef066531f1e1d96e7e25b49",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 15
          },
          {
            "boxId": "6e2ef984458fabfd1d32c3f01943a281ab1b1ad98ce1b2fc7bd65020c28ed720",
            "value": 10033979953,
            "ergoTree": "0008cd02a06ccbca2250b6d904f2dad206e5a7dd5d387d92d8a1d7e2db357db1b392c128",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 16
          },
          {
            "boxId": "f24dba2d15493a807f42adf7316fb7df9e976c193b6b69f4d3e1f7ccc59e813c",
            "value": 1000740079,
            "ergoTree": "0008cd02526c822c9111f115b79405118294bce173b287dff540a50ee4b61c176c977299",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 17
          },
          {
            "boxId": "0a951204596dc8a795ca28951c746622f55b855a2ea300397b38f20d636108de",
            "value": 101396721,
            "ergoTree": "0008cd02c881b58b9012d829789ec2a072fcfd4148a7f0917a57285a0f2e91e0b48bb07a",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 18
          },
          {
            "boxId": "c9b37a216d65647b8b23864563823a221decd1b277ddafcf6783c8993cd63507",
            "value": 1033691668,
            "ergoTree": "0008cd03e7e50df47f1ff07953c75e7b12552939159a37013af8c9993063e0d1b91b9d22",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 19
          },
          {
            "boxId": "55002ee68590a54fb8f984a69f5ef196b64ad8ebe374defa68dcaf9c39adcbd4",
            "value": 106401930,
            "ergoTree": "0008cd02a92664f7ca68185691cf23cbe9d4412465fe3af64bafc7c328b6d25900ed5a84",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 20
          },
          {
            "boxId": "e465e8d52d947b5dcc3798766b0f4106ecdf5c5d3d5f02a11c23a9791f0f58f5",
            "value": 115432395,
            "ergoTree": "0008cd03376ef34624615a8f03f28161b9be44c0c5754a5cb2dd594129924244a5b48f42",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 21
          },
          {
            "boxId": "6be6ce5cd1a5ee8c17f1685ad8f0e22d37d2cabc8cf00b5fbc40df88af1b2d80",
            "value": 1010521524,
            "ergoTree": "0008cd02afd95b35c437fe539b44f660ee88f15bc6e08ad2214bdb5afa857282da2f7aa3",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 22
          },
          {
            "boxId": "6d31a003416d56f656cae9e2ccbf4bfd5bd76820543b3719f72193be8b4697dc",
            "value": 1029866438,
            "ergoTree": "0008cd03592ee05344456d2717e1810de431b1f078bad845af2f3375e5530199fa387907",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 23
          },
          {
            "boxId": "055d5247be36ecbdad82167e7cef890d1f3cc8efdb21c3680f8c26f461fbf312",
            "value": 1009153034,
            "ergoTree": "0008cd03408d8f0c58972c759ee613e62b2df77ef37114c0d0beb8554e0d1340fe0eadde",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 24
          },
          {
            "boxId": "b522d732134f15f7f9af286d8aa2e062b8f1cc9f19ff04f69cd4871d83f95bfc",
            "value": 510420797,
            "ergoTree": "0008cd026230d8638bb67a7a8a6a0e9c841536488978e57eeae1562f56fd932bd7b51e7c",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 25
          },
          {
            "boxId": "aaf8f5fdb34f30be10979d20146ee796beb72211a32e5bda3e2b0c0835faa14a",
            "value": 1011404511,
            "ergoTree": "0008cd0395ff09b7c98dd1123f9785af34577d2c3a09cab992f20a40ae4be781e8605018",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 26
          },
          {
            "boxId": "69fdcefb383f972e4058ac13c5afe658792a6902674ca41925e442f2b98632b7",
            "value": 1100000,
            "ergoTree": "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304",
            "assets": [],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 27
          },
          {
            "boxId": "74c5d8916264e5714903ccf1d0f73cac17846529b3be4f045eff94251961e60b",
            "value": 11353040072956,
            "ergoTree": "0008cd029ed28cae37942d25d5cc5f0ade4b1b2e03e18b05c4f3233018bf67c817709f93",
            "assets": [
              {
                "tokenId": "d1ae85958e31d24cffde6f09c9d492819fad950dafb76b17edcd80badd6fe8ef",
                "amount": 1
              },
              {
                "tokenId": "0cd8c9f416e5b1ca9f986a7f10a84191dfb85941619e49e53c0dc30ebf83324b",
                "amount": 10000
              }
            ],
            "creationHeight": 693477,
            "additionalRegisters": {},
            "transactionId": "f30b3044a2b0c83555086d236c59fe9bdc3553deea742bf5010c7185082d5759",
            "index": 28
          }
        ],
        "size": 1737
      },
      {
        "id": "db40eab029fa1dbab09476fbb4342c99bf4c1b07cf5a943c48019702b0e7c16f",
        "inputs": [
          {
            "boxId": "00ddbeb981c0b08536f72ea41e07a25adbf7bf104ee59b865619a21676e64715",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "18aaabf889a4d3a111b07a2beb932700cf6e0db4f2c654044aef507f1860449c",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "483f1420cc1afce2ea539075654800516c9a8341a3b41d9ba435ed9f217ceb50",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "5291c872959a81fa1e0f6696b41e911a09be732ab70871d9504a0792249c3633",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "5b44961a26230f4c37dba57125149284fe808986bfb939bf35aefce35eae554a",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "69fdcefb383f972e4058ac13c5afe658792a6902674ca41925e442f2b98632b7",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "7a8537700e23539c951f380f455ce58cab45ccd4daf4711fa13532b2f8060ff0",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "85dc5b034f8815952cdbad90aa7b50b68b56702e76b89ca799e2dc0e47b35eca",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "8847701f6eaa5629ab350f894a626b533bdf83beab6b12c620ad854cb5cad02d",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "90a0ed9da366367c88cca4d8c63bbca4e77f96deef885f48df564f857a78d374",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "9d383716046dab202a2cb6b285ef391b46ccfc32839b427bf1e210374cbfcf59",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "b546b26276a660013ae70ac937a05d6f4e2be9eda04aff6e06b1d2cfe8b974a8",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          },
          {
            "boxId": "f259d6bf9d4d5564401a2ac4da7e3eabe6c01288a177a23f360b34280fe262cf",
            "spendingProof": {
              "proofBytes": "",
              "extension": {}
            }
          }
        ],
        "dataInputs": [],
        "outputs": [
          {
            "boxId": "06bb97c824358fe7aec8b91c04587402aa3ec0007934b4a3156b3bffcd8a3356",
            "value": 22900000,
            "ergoTree": "100204a00b08cd0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891fea02d192a39a8cc7a70173007301",
            "assets": [],
            "creationHeight": 693479,
            "additionalRegisters": {},
            "transactionId": "db40eab029fa1dbab09476fbb4342c99bf4c1b07cf5a943c48019702b0e7c16f",
            "index": 0
          }
        ],
        "size": 509
      }
    ],
    "blockVersion": 2,
    "size": 11100
  },
  "extension": {
    "headerId": "b17847c0c523660b13d707396ab8301fa3c8a545ddc5acf9ec2803cc2cbb3ef5",
    "digest": "5fb2be1ff25d365daadbb3cd4908feceff097bb4ad8f6c7f1436a04ffa3bf5cd",
    "fields": [
      [
        "0100",
        "01b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b"
      ],
      [
        "0101",
        "041155d54de65f0130fae142aa4cf5a7728b7c30f5939d33fddf077e2008040a15"
      ],
      [
        "0105",
        "01116a6c1d030c62d333df6d518e26887745e5251d6d2270e5560fe4cce85ad0a3"
      ],
      [
        "0106",
        "015aad19a4b658e59ec098f06c4f0b6f3317b09e6a6fe9e49be340933e709a5a1e"
      ],
      [
        "0107",
        "039501b674e3e4678a659d9abf63c079558305ae1dbc3d5f97cd07195b2423ddd5"
      ],
      [
        "010a",
        "02ed35fa3373a6035aca1552005380ef67f9ce90bcb651f6dbee64db3f1f5efdc9"
      ],
      [
        "010c",
        "02afe4739c3fd01466c309d3b0d27bfdda18c5570ec2ede33b68792f8f8f315be8"
      ],
      [
        "010e",
        "037508c526d5371c95ef344e39b46709ee918b904a91c9f35cb26b6794eab9233f"
      ],
      [
        "0111",
        "02addf788724adae14e0fe75538b91b8bc192564e2355819a2715d376bb1229ee8"
      ],
      [
        "0113",
        "0272a17b1cd5863bbc598f68678a35168ae5e4eadf602edd778eb3b6f7312cdc65"
      ]
    ]
  },
  "adProofs": null,
  "size": 11321
}

        "#;
        let block_0: FullBlock = serde_json::from_str(json).unwrap();
        let encoded_json = serde_json::to_string(&block_0).unwrap();
        let block_1: FullBlock = serde_json::from_str(&encoded_json).unwrap();
        assert_eq!(block_0, block_1);
    }
}
