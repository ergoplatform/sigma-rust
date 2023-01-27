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

#[cfg(test)]
mod tests {
    use super::FullBlock;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_full_block() {
        // Following JSON taken from the node by:
        //   curl -X GET "https://node.ergo.watch/blocks/96911575efdceb082b974aa3042263be07632de48031aa2204d77d8d5a8240b8" -H "accept: application/json"

        let json: &str = r#"
        {
            "header": {
              "extensionId": "a1c5a5f409fce4d16a501371b11aaaf0e0a44609d8436958c383e12f9c14528c",
              "difficulty": "1371769604669440",
              "votes": "000000",
              "timestamp": 1627249021284,
              "size": 221,
              "stateRoot": "1d3d031ba060245d8184948c6f726a8bb98a1bc621affc4a1dcf0e20226eb27716",
              "height": 540000,
              "nBits": 117759902,
              "version": 2,
              "id": "96911575efdceb082b974aa3042263be07632de48031aa2204d77d8d5a8240b8",
              "adProofsRoot": "aa0d212ec398d9558b2b2f24239963bdd8d2d22f70b6e8b5cfff3474609bcdde",
              "transactionsRoot": "235a6e8f28f54fef5fbcd17d2638eb03ef9cfb331f4b5a50fbb74df4a524dcb4",
              "extensionHash": "badffc4d646e1c2babcf1ce8422b4f2430b6262c947c964671e97486d8bdb601",
              "powSolutions": {
                "pk": "02b3a06d6eaa8671431ba1db4dd427a77f75a5c2acbd71bfb725d38adc2b55f669",
                "w": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
                "n": "0537288a2c246648",
                "d": 0
              },
              "adProofsId": "13856ec4123971268ff0d7493bfa520021c6328ceba648bf39484b45761f4edf",
              "transactionsId": "5871d44565a08892d03f3e4f53a3d98a7f21e549738fff0864bce205916a5bfb",
              "parentId": "c55f05c91fea37f95eff73dfa62e8745f54db6dff5e9f257e39b9c0cfbfd8133"
            },
            "blockTransactions": {
              "headerId": "96911575efdceb082b974aa3042263be07632de48031aa2204d77d8d5a8240b8",
              "transactions": [
                {
                  "id": "d301f351d5d74aa314edd19914e4e593bd0316166c25a09aa222f9b519ee5fdf",
                  "inputs": [
                    {
                      "boxId": "805a5a5293a38c4ef872f5a1b392404a2808f7ca1f149f0874dbddd31a30677f",
                      "spendingProof": {
                        "proofBytes": "",
                        "extension": {}
                      }
                    }
                  ],
                  "dataInputs": [],
                  "outputs": [
                    {
                      "boxId": "b00eee09bb8ad9b3b4d93042fd28c966aa9b225c228732c69cb74656788ae8f0",
                      "value": 56959132500000000,
                      "ergoTree": "101004020e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a7017300730110010204020404040004c0fd4f05808c82f5f6030580b8c9e5ae040580f882ad16040204c0944004c0f407040004000580f882ad16d19683030191a38cc7a7019683020193c2b2a57300007473017302830108cdeeac93a38cc7b2a573030001978302019683040193b1a5730493c2a7c2b2a573050093958fa3730673079973089c73097e9a730a9d99a3730b730c0599c1a7c1b2a5730d00938cc7b2a5730e0001a390c1a7730f",
                      "assets": [],
                      "creationHeight": 540000,
                      "additionalRegisters": {},
                      "transactionId": "d301f351d5d74aa314edd19914e4e593bd0316166c25a09aa222f9b519ee5fdf",
                      "index": 0
                    },
                    {
                      "boxId": "9c700fdcfa7cb5fa83df806b30f69bc1a4690e33b1af77076ced4f7b28c76e37",
                      "value": 67500000000,
                      "ergoTree": "100204a00b08cd02b3a06d6eaa8671431ba1db4dd427a77f75a5c2acbd71bfb725d38adc2b55f669ea02d192a39a8cc7a70173007301",
                      "assets": [],
                      "creationHeight": 540000,
                      "additionalRegisters": {},
                      "transactionId": "d301f351d5d74aa314edd19914e4e593bd0316166c25a09aa222f9b519ee5fdf",
                      "index": 1
                    }
                  ],
                  "size": 344
                }
              ],
              "blockVersion": 2,
              "size": 381
            },
            "extension": {
              "headerId": "96911575efdceb082b974aa3042263be07632de48031aa2204d77d8d5a8240b8",
              "digest": "badffc4d646e1c2babcf1ce8422b4f2430b6262c947c964671e97486d8bdb601",
              "fields": [
                [
                  "0100",
                  "01b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b"
                ],
                [
                  "0101",
                  "01557fd0590616b4f6e51eaf54436d61e5585eebfc5a9e860861fc0876064bd3d9"
                ],
                [
                  "0102",
                  "03296e2707cf72b6a2c71e4966028d8786c7f5425850e9609757ce8b3713f548fe"
                ],
                [
                  "0105",
                  "027ddba9db07cce855cd911c9bee9376be9e16cedf66eeed2175072816c5678cdb"
                ],
                [
                  "0107",
                  "05e31fdeefaee294c99d11cdfcf8a7c28158ba16c7b7ccce6ff98c4bf1b8b65873"
                ],
                [
                  "010c",
                  "01dcf7326a3daf36f5f49e279e24f335a3947bec606eacd722637e45f0cbc8ecd9"
                ],
                [
                  "010d",
                  "013a9bb64834421e8ab964dfb5fcc6f808027559ad7901ccacf6d283d57f069c83"
                ],
                [
                  "010e",
                  "03338bd47eca3694f9e5d2f146abef73582a6520adc4369ec624ccd3343afb598b"
                ],
                [
                  "0111",
                  "018f9b36c08403f4088d31e2f331b136a9b2b0f6c05cd110546d517860c977d49d"
                ],
                [
                  "0112",
                  "02c55f05c91fea37f95eff73dfa62e8745f54db6dff5e9f257e39b9c0cfbfd8133"
                ]
              ]
            },
            "adProofs": null,
            "size": 602
          }
        "#;

        let block_0: FullBlock = serde_json::from_str(json).unwrap();
        let encoded_json = serde_json::to_string(&block_0).unwrap();
        let block_1: FullBlock = serde_json::from_str(&encoded_json).unwrap();
        assert_eq!(block_0, block_1);
    }
}
