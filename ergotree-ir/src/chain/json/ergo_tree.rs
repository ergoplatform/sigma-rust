use crate::ergo_tree::ErgoTree;
use crate::serialization::SigmaSerializable;
use serde::{Deserialize, Deserializer, Serializer};

use super::serialize_bytes;

pub fn serialize<S>(ergo_tree: &ErgoTree, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::Error;
    let bytes = ergo_tree
        .sigma_serialize_bytes()
        .map_err(|err| Error::custom(err.to_string()))?;
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
            ErgoTree::sigma_parse_bytes(&bytes).map_err(|error| Error::custom(error.to_string()))
        })
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::chain::json::ergo_box::ErgoBoxFromJson;

    #[test]
    fn parse_ergo_tree_with_constants() {
        let json = r#"
            {"boxId":"dd4e69ae683d7c2d1de2b3174182e6c443fd68abbcc24002ddc99adb599e0193","value":1000000,"ergoTree":"0008cd03f1102eb87a4166bf9fbd6247d087e92e1412b0e819dbb5fbc4e716091ec4e4ec","assets":[],"creationHeight":268539,"additionalRegisters":{},"transactionId":"8204d2bbaabf946f89a27b366d1356eb10241dc1619a70b4e4a4a38b520926ce","index":0}
        "#;
        let b: ErgoBoxFromJson = serde_json::from_str(json).unwrap();
        assert!(b.ergo_tree.proposition().is_ok())
    }
}
