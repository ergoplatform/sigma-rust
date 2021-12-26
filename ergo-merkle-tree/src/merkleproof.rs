use crate::{concatenate_hashes, prefixed_hash};

use serde::Serialize;
use serde_repr::*;

// Serializes an array of bytes in base 16 format
fn serialize_base64<T: AsRef<[u8]>, S: serde::Serializer>(
    digest: T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&base16::encode_lower(digest.as_ref()))
}

// Serializes each node's hash as base16
fn serialize_nodes<T: AsRef<[u8]>, S: serde::Serializer>(
    nodes: &[(T, NodeSide)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(nodes.len()))?;
    for node in nodes {
        seq.serialize_element(&(&base16::encode_lower(node.0.as_ref()), node.1))?;
    }
    seq.end()
}

/// The side the merkle node is on in the tree
#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NodeSide {
    /// Node is on the left side of the current level
    Left = 0,
    /// Node is on the righ side of the current level
    Right = 1,
}

impl std::convert::TryFrom<u8> for NodeSide {
    type Error = &'static str;
    fn try_from(side: u8) -> Result<Self, Self::Error> {
        match side {
            0 => Ok(NodeSide::Left),
            1 => Ok(NodeSide::Right),
            _ => Err("Side is out of bounds"),
        }
    }
}

/// A MerkleProof type. Given leaf data and levels (bottom-upwards), the root hash can be computed and validated
#[derive(Clone, Debug, Serialize)]
pub struct MerkleProof {
    #[serde(rename = "leafData")]
    #[serde(serialize_with = "serialize_base64")]
    leaf_data: Vec<u8>,
    #[serde(serialize_with = "serialize_nodes")]
    levels: Vec<([u8; 32], NodeSide)>,
}

impl MerkleProof {
    /// Creates a new merkle proof with given leaf data and level data (bottom-upwards)
    /// You can verify it against a Blakeb256 root hash by using [`Self::valid()`]
    pub fn new(leaf_data: &[u8], levels: &[([u8; 32], NodeSide)]) -> Self {
        MerkleProof {
            leaf_data: leaf_data.to_owned(),
            levels: levels.to_owned(),
        }
    }

    /// Validates the Merkle Proof against the expected root hash
    pub fn valid(&self, expected_root: &[u8; 32]) -> bool {
        let leaf_hash = prefixed_hash(0, &self.leaf_data); // Prefix hash with 0 (leaf node)
        let hash = self
            .levels
            .iter()
            .fold(leaf_hash, |prev_hash, (hash, side)| match side {
                NodeSide::Left => prefixed_hash(1, &concatenate_hashes(&prev_hash, hash)), // Prefix hash with 1 (internal node hash)
                NodeSide::Right => prefixed_hash(1, &concatenate_hashes(hash, &prev_hash)),
            });

        &*hash == expected_root
    }
}

#[cfg(test)]
mod test {
    use crate::MerkleProof;
    use crate::NodeSide;

    // Ported client Merkle tree verification example from  https://github.com/ergoplatform/ergo/blob/master/src/test/scala/org/ergoplatform/examples/LiteClientExamples.scala
    #[test]
    fn miner_proof() {
        let msg_preimage = "01fb9e35f8a73c128b73e8fde5c108228060d68f11a69359ee0fb9bfd84e7ecde6d19957ccbbe75b075b3baf1cac6126b6e80b5770258f4cec29fbde92337faeec74c851610658a40f5ae74aa3a4babd5751bd827a6ccc1fe069468ef487cb90a8c452f6f90ab0b6c818f19b5d17befd85de199d533893a359eb25e7804c8b5d7514d784c8e0e52dabae6e89a9d6ed9c84388b228e7cdee09462488c636a87931d656eb8b40f82a507008ccacbee05000000";
        let msg_preimage = base16::decode(msg_preimage).unwrap();

        let tx_id = "642c15c62553edd8fd9af9a6f754f3c7a6c03faacd0c9b9d5b7d11052c6c6fe8";
        let levels_encoded = "0139b79af823a92aa72ced2c6d9e7f7f4687de5b5af7fab0ad205d3e54bda3f3ae";

        let mut levels = base16::decode(levels_encoded).unwrap();
        let side: NodeSide = levels.remove(0).try_into().unwrap(); // first byte encodes side information (0 = Left, 1 = Right)

        let tx_root = &msg_preimage[65..97];

        assert_eq!(levels.len(), 32);
        let tx_id = base16::decode(&tx_id).unwrap();
        let proof = MerkleProof::new(&tx_id, &[(levels[0..32].try_into().unwrap(), side)]);
        assert!(proof.valid(tx_root.try_into().unwrap()));
    }
}
