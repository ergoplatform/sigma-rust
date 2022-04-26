use crate::INTERNAL_PREFIX;
use crate::{prefixed_hash, prefixed_hash2};
use ergo_chain_types::Digest32;

/// The side the merkle node is on in the tree
#[cfg_attr(
    feature = "json",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeSide {
    /// Node is on the left side of the current level
    Left = 0,
    /// Node is on the right side of the current level
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

/// A LevelNode used for MerkleProof verification, consists of a 32 byte hash and side it is on in tree

#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(into = "crate::json::LevelNodeJson"),
    serde(try_from = "crate::json::LevelNodeJson")
)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelNode {
    /// Hash for LevelNode. Use [`LevelNode::empty_node`] to create a node with no hash
    pub hash: Option<Digest32>,
    /// Node Side in Merkle Tree
    pub side: NodeSide,
}

impl LevelNode {
    /// Constructs a new levelnode from a 32 byte hash
    pub fn new(hash: Digest32, side: NodeSide) -> Self {
        Self {
            hash: Some(hash),
            side,
        }
    }
    /// Creates a new level node with no associated hash
    pub fn empty_node(side: NodeSide) -> Self {
        Self { hash: None, side }
    }
}

/// A MerkleProof type. Given leaf data and levels (bottom-upwards), the root hash can be computed and validated
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(try_from = "crate::json::MerkleProofJson"),
    serde(into = "crate::json::MerkleProofJson")
)]
#[derive(Clone, Debug)]
pub struct MerkleProof {
    pub(crate) leaf_data: Vec<u8>,
    pub(crate) levels: Vec<LevelNode>,
}

impl MerkleProof {
    /// Creates a new merkle proof with given leaf data and level data (bottom-upwards)
    /// You can verify it against a Blakeb256 root hash by using [`Self::valid()`]
    pub fn new(leaf_data: &[u8], levels: &[LevelNode]) -> Self {
        MerkleProof {
            leaf_data: leaf_data.to_owned(),
            levels: levels.to_owned(),
        }
    }

    /// Validates the Merkle Proof against the expected root hash
    pub fn valid(&self, expected_root: &[u8]) -> bool {
        let leaf_hash = prefixed_hash(0, &self.leaf_data); // Prefix hash with 0 (leaf node)
        let hash = self
            .levels
            .iter()
            .fold(leaf_hash, |prev_hash, node| match node {
                LevelNode {
                    hash: Some(hash),
                    side: NodeSide::Left,
                } => prefixed_hash2(INTERNAL_PREFIX, prev_hash.as_ref(), hash.as_ref()), // Prefix hash with 1 (internal node hash)
                LevelNode {
                    hash: Some(hash),
                    side: NodeSide::Right,
                } => prefixed_hash2(INTERNAL_PREFIX, hash.as_ref(), prev_hash.as_ref()),
                LevelNode { hash: None, .. } => prefixed_hash(INTERNAL_PREFIX, prev_hash.as_ref()),
            });

        hash.as_ref() == expected_root
    }
    #[cfg(feature = "json")]
    /// Validates the MerkleProof against a base16 hash
    pub fn valid_base16(&self, expected_root: &str) -> Result<bool, base16::DecodeError> {
        // The rationale for adding this function is mainly to make using MerkleProof in Swift easier, without resorting to add a new dependency to base16
        let expected_root = base16::decode(expected_root)?;
        Ok(self.valid(&expected_root))
    }

    /// Adds a new node (above the current node)
    pub fn add_node(&mut self, node: LevelNode) {
        self.levels.push(node);
    }

    /// Returns Leaf Data for proof node
    pub fn get_leaf_data(&self) -> &[u8] {
        &self.leaf_data
    }
}

#[cfg(test)]
#[cfg(feature = "json")]
#[allow(clippy::unwrap_used)]
mod test {
    use crate::LevelNode;
    use crate::MerkleProof;
    use crate::NodeSide;
    use ergo_chain_types::Digest32;
    use std::convert::TryFrom;

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
        let proof = MerkleProof::new(
            &tx_id,
            &[LevelNode::new(
                Digest32::try_from(&levels[0..32]).unwrap(),
                side,
            )],
        );
        assert!(proof.valid(tx_root));
    }

    // Proof for block #650787 on Ergo mainnet
    #[test]
    fn block_proof() {
        let json = "{
            \"leafData\": \"563b34b96e65788d767a10b0c2ce4a9ef5dcb9f7f7919781624870d56506dc5b\",
            \"levels\": [
                [\"274d105b42c2da3e03519865470ccef5072d389b153535ca7192fef4abf3b3ed\", 0],
                [\"c1887cee0c42318ac04dfa93b8ef6b40c2b53a83b0e111f91a16b0842166e76e\", 0],
                [\"58be076cd9ef596a739ec551cbb6b467b95044c05a80a66a7f256d4ebafd787f\", 0]]
            }";
        let proof: MerkleProof = serde_json::from_str(json).unwrap();
        let tx_root =
            base16::decode("250063ac1cec3bf56f727f644f49b70515616afa6009857a29b1fe298441e69a")
                .unwrap();

        assert!(proof.valid(&tx_root));
    }

    // Tests block #0 on ergo mainnet, which contains only one transaction
    #[test]
    fn merkle_proof_genesis_block() {
        let json = "{
        \"leafData\" : \"4c6282be413c6e300a530618b37790be5f286ded758accc2aebd41554a1be308\",
        \"levels\" : [[\"\", 0]]}";
        let proof: MerkleProof = serde_json::from_str(json).unwrap();
        let tx_root =
            base16::decode("93fb06aa44413ff57ac878fda9377207d5db0e78833556b331b4d9727b3153ba")
                .unwrap();
        assert!(proof.valid(&tx_root));
    }
}
