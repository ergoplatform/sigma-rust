//! Merkle Proof verification
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// A MerkleProof type. Given leaf data and levels (bottom-upwards), the root hash can be computed and validated
pub struct MerkleProof(ergo_merkle_tree::MerkleProof);

#[wasm_bindgen]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
/// The side the merkle node is on in the tree
pub enum NodeSide {
    /// Node is on the left side of current level
    Left = 0u8,
    /// Node is on the right side of current level
    Right = 1u8,
}

impl Into<ergo_merkle_tree::NodeSide> for NodeSide {
    fn into(self) -> ergo_merkle_tree::NodeSide {
        match self {
            NodeSide::Left => ergo_merkle_tree::NodeSide::Left,
            NodeSide::Right => ergo_merkle_tree::NodeSide::Right,
        }
    }
}
impl From<ergo_merkle_tree::NodeSide> for NodeSide {
    fn from(side: ergo_merkle_tree::NodeSide) -> NodeSide {
        match side {
            ergo_merkle_tree::NodeSide::Left => NodeSide::Left,
            ergo_merkle_tree::NodeSide::Right => NodeSide::Right,
        }
    }
}

/// A level node in a merkle proof
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct LevelNode(ergo_merkle_tree::LevelNode);

#[wasm_bindgen]
impl LevelNode {
    /// Creates a new LevelNode from a 32 byte hash and side that the node belongs on in the tree. Fails if the digest is not 32 bytes
    pub fn new(hash: &[u8], side: NodeSide) -> Result<LevelNode, JsValue> {
        Ok(Self(ergo_merkle_tree::LevelNode::new(
            (*hash)
                .try_into()
                .map_err(|_| "Digest is not 32 bytes in size")?,
            side.into(),
        )))
    }
    /// Returns the associated digest (hash) with this node
    #[wasm_bindgen(getter)]
    pub fn digest(&self) -> Vec<u8> {
        (&self.0 .0[0..]).to_owned()
    }
    /// Returns the associated side with this node
    #[wasm_bindgen(getter)]
    pub fn side(&self) -> NodeSide {
        self.0 .1.into()
    }
}

#[wasm_bindgen]
impl MerkleProof {
    /// Creates a new merkle proof with given leaf data and level data (bottom-upwards)
    /// You can verify it against a Blakeb256 root hash by using [`Self::valid()`]
    /// Add a node by using [`Self::add_node()`]
    /// Each digest on the level must be exactly 32 bytes
    pub fn new(leaf_data: &[u8]) -> Self {
        Self(ergo_merkle_tree::MerkleProof::new(leaf_data, &[])) // There are issues with wasm when trying to pass an array of structs, so it's better to use add_node instead
    }

    /// Adds a new node to the MerkleProof above the current nodes
    pub fn add_node(&mut self, level: LevelNode) {
        self.0.add_node(level.0);
    }

    /// Validates the Merkle proof against the root hash
    pub fn valid(&self, expected_root: &[u8]) -> bool {
        self.0.valid(expected_root)
    }
}

#[wasm_bindgen]
/// Decodes a base16 string into an array of bytes
pub fn base16_decode(data: &str) -> Result<Vec<u8>, JsValue> {
    return base16::decode(&data).map_err(|_| "Failed to decode base16 input".into());
}
