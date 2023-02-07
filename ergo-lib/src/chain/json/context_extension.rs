use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use ergotree_ir::{mir::constant::Constant, serialization::SigmaSerializable};
use indexmap::IndexMap;
use serde::{ser::SerializeMap, Deserialize, Serialize};

#[cfg_attr(
    feature = "json",
    derive(Deserialize),
    serde(try_from = "indexmap::IndexMap<String, String>"),
    serde(remote = "ContextExtension")
)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ContextExtensionSerde {
    values: IndexMap<u8, Constant>,
}

#[cfg(feature = "json")]
impl Serialize for ContextExtensionSerde {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (k, v) in &self.values {
            map.serialize_entry(
                &format!("{}", k),
                &base16::encode_lower(&v.sigma_serialize_bytes().map_err(Error::custom)?),
            )?;
        }
        map.end()
    }
}

impl From<ContextExtension> for ContextExtensionSerde {
    fn from(ce: ContextExtension) -> Self {
        ContextExtensionSerde { values: ce.values }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use ergo_chain_types::Digest32;

    use crate::chain::transaction::Transaction;

    use super::*;

    #[test]
    fn parse_empty_context_extension() {
        let mut de = serde_json::Deserializer::from_str("{}");
        let c: ContextExtension = ContextExtensionSerde::deserialize(&mut de).unwrap();
        assert_eq!(c, ContextExtension::empty());
    }

    #[test]
    fn parse_context_extension() {
        let json = r#"
        {"1" :"05b0b5cad8e6dbaef44a", "3":"048ce5d4e505"}
        "#;
        let mut de = serde_json::Deserializer::from_str(json);
        let c: ContextExtension = ContextExtensionSerde::deserialize(&mut de).unwrap();
        assert_eq!(c.values.len(), 2);
        assert!(c.values.get(&1u8).is_some());
        assert!(c.values.get(&3u8).is_some());
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
}
