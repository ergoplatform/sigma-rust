use crate::batchmerkleproof::{BatchMerkleProof, BatchMerkleProofIndex};
use crate::{prefixed_hash, prefixed_hash2, INTERNAL_PREFIX, LEAF_PREFIX};
use ergo_chain_types::Digest32;
use std::collections::{BTreeSet, HashMap};

/// Node for a Merkle Tree
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub enum MerkleNode {
    #[doc(hidden)]
    Node { hash: Digest32 },
    /// Leaf Node in MerkleTree. Can be created using [`Self::from_bytes`] or [`Self::from`]
    Leaf {
        /// 32 byte Blake2b256 hash for data
        hash: Digest32,
        /// Leaf Data
        data: Vec<u8>,
    },
    #[doc(hidden)]
    EmptyNode,
}

impl MerkleNode {
    /// Creates a new Leaf Node from bytes. The hash is prefixed with a leaf node prefix.
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Self {
        let hash = prefixed_hash(LEAF_PREFIX, bytes.as_ref());
        MerkleNode::Leaf {
            hash,
            data: bytes.as_ref().to_owned(),
        }
    }
    /// Gets hash for the node, returns None if it's an Empty Node
    pub fn get_hash(&self) -> Option<&Digest32> {
        match self {
            MerkleNode::Node { hash } => Some(hash),
            MerkleNode::Leaf { hash, .. } => Some(hash),
            _ => None,
        }
    }
    /// Gets data for the node if it's a leaf node
    pub fn get_leaf_data(&self) -> Option<&Vec<u8>> {
        match self {
            MerkleNode::Leaf { data, .. } => Some(data),
            _ => None,
        }
    }
    pub(crate) fn empty() -> Self {
        Self::EmptyNode
    }
}

// utillity functions for indexing a binary tree
fn get_left(node_index: usize) -> usize {
    2 * node_index + 1
}
fn get_right(node_index: usize) -> usize {
    2 * node_index + 2
}

fn get_parent(index: usize) -> Option<usize> {
    index.checked_sub(1).map(|v| v / 2)
}

fn get_sibling(index: usize) -> Option<usize> {
    let parent_index = get_parent(index)?;
    if get_left(parent_index) != index {
        Some(get_left(parent_index))
    } else {
        Some(get_right(parent_index))
    }
}

// builds a new MerkleProof from tree nodes
fn build_proof(
    nodes: &[MerkleNode],
    mut leaf_index: usize,
    internal_nodes: usize,
) -> Option<crate::MerkleProof> {
    leaf_index += internal_nodes;
    let mut proof_nodes: Vec<crate::LevelNode> = vec![];
    let leaf_data = match nodes.get(leaf_index) {
        Some(MerkleNode::Leaf { data, .. }) => data,
        _ => return None,
    };
    while let Some(sibling) = get_sibling(leaf_index) {
        let side = if sibling == leaf_index + 1 {
            crate::NodeSide::Left // side information is encoded relative to the node we're trying to prove is in the tree. The leaf is on the left of the current node
        } else {
            crate::NodeSide::Right
        };
        match nodes[sibling].get_hash() {
            Some(hash) => proof_nodes.push(crate::LevelNode::new(hash.clone(), side)),
            _ => proof_nodes.push(crate::LevelNode::empty_node(side)),
        }
        leaf_index = get_parent(leaf_index)?;
    }

    Some(crate::MerkleProof::new(leaf_data, &proof_nodes))
}

fn build_multiproof(
    nodes: &[MerkleNode],
    leaf_indices: &[usize],
    internal_nodes: usize,
) -> Option<BatchMerkleProof> {
    let mut multiproof: Vec<crate::LevelNode> = vec![];

    let mut a: BTreeSet<usize> = leaf_indices
        .iter()
        .map(|idx| idx + internal_nodes)
        .collect();
    // while a does not contain the root index (0)
    while !a.contains(&0) {
        let mut b_pruned = BTreeSet::new();
        for node in &a {
            // for each leaf node, insert it and it's neighbor into the set. Since we're inserting into a set, we don't need any deduplication or sorting
            b_pruned.insert(*node);
            b_pruned.insert(get_sibling(*node)?);
        }

        let diff = &b_pruned - &a;
        for node in diff {
            let side = match get_sibling(node) {
                Some(s) if s == node - 1 => crate::NodeSide::Right,
                Some(_) => crate::NodeSide::Left,
                None => unreachable!(),
            };
            let levelnode = match nodes[node].get_hash() {
                Some(hash) => crate::LevelNode::new(hash.clone(), side),
                None => crate::LevelNode::empty_node(side),
            };
            multiproof.push(levelnode);
        }
        a = b_pruned.into_iter().flat_map(get_parent).collect();
    }

    Some(BatchMerkleProof::new(
        leaf_indices
            .iter()
            .flat_map(|idx| {
                Some(BatchMerkleProofIndex {
                    index: *idx,
                    hash: nodes[idx + internal_nodes].get_hash().cloned()?,
                })
            })
            .collect(),
        multiproof,
    ))
}

/// Merkle Tree
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub struct MerkleTree {
    nodes: Vec<MerkleNode>,
    elements_hash_index: HashMap<Digest32, usize>,
    internal_nodes: usize,
}

impl MerkleTree {
    /// Creates a new MerkleTree from leaf nodes
    pub fn new(nodes: impl Into<Vec<MerkleNode>>) -> Self {
        #[allow(clippy::unwrap_used)]
        fn build_nodes(nodes: &mut [MerkleNode]) {
            for pair in (1..nodes.len()).step_by(2).rev() {
                let node = match (
                    nodes[pair].get_hash(),
                    nodes[get_sibling(pair).unwrap()].get_hash(), // since we pad nodes with no sibling with an empty node, get_sibling should always return Some
                ) {
                    (Some(left_hash), Some(right_hash)) => MerkleNode::Node {
                        hash: prefixed_hash2(
                            INTERNAL_PREFIX,
                            left_hash.as_ref(),
                            right_hash.as_ref(),
                        ),
                    },
                    (Some(hash), None) => MerkleNode::Node {
                        hash: prefixed_hash(INTERNAL_PREFIX, hash.as_ref()),
                    },
                    (None, None) => MerkleNode::EmptyNode,
                    _ => unreachable!(),
                };
                nodes[get_parent(pair).unwrap()] = node;
            }
        }
        let mut tree_nodes = nodes.into();
        if tree_nodes.len() % 2 == 1 {
            tree_nodes.push(MerkleNode::EmptyNode);
        }
        let elements_hash_index = tree_nodes
            .iter()
            .flat_map(MerkleNode::get_hash)
            .enumerate()
            .map(|(i, node)| (node.clone(), i))
            .collect();
        let leaf_nodes = tree_nodes.len();
        // prepend leaf nodes with empty nodes to build the full tree
        tree_nodes.splice(
            0..0,
            std::iter::repeat(MerkleNode::empty()).take(tree_nodes.len().next_power_of_two() - 1),
        );
        build_nodes(&mut tree_nodes);
        let nodes_len = tree_nodes.len();
        Self {
            nodes: tree_nodes,
            elements_hash_index,
            internal_nodes: nodes_len - leaf_nodes,
        }
    }

    /// Returns the root hash for MerkleTree. If the tree is empty, then returns [0; 32]
    pub fn root_hash(&self) -> Digest32 {
        self.nodes
            .get(0)
            .and_then(MerkleNode::get_hash)
            .cloned()
            .unwrap_or_else(Digest32::zero)
    }
    /// Returns the root hash for MerkleTree. If the tree is empty, then returns a special hash '0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8', which is extension/transaction root hash for genesis block
    /// See: <https://github.com/ergoplatform/ergo/issues/1077>
    pub fn root_hash_special(&self) -> Digest32 {
        self.nodes
            .get(0)
            .and_then(MerkleNode::get_hash)
            .cloned()
            .unwrap_or_else(
                #[allow(clippy::unwrap_used)]
                // unwrap is safe to use here, since digest size is always 32 bytes
                || {
                    use blake2::digest::{Update, VariableOutput};
                    let mut hasher = crate::VarBlake2b::new(32).unwrap();
                    hasher.update(&[]);
                    let hash: Box<[u8; 32]> = hasher.finalize_boxed().try_into().unwrap();
                    Digest32::from(hash)
                },
            )
    }

    /// Returns HashMap of hashes and their index in the tree
    pub fn get_elements_hash_index(&self) -> &HashMap<Digest32, usize> {
        &self.elements_hash_index
    }

    /// Builds a [`crate::MerkleProof`] for leaf_index. Returns None if index does not exist
    pub fn proof_by_index(&self, leaf_index: usize) -> Option<crate::MerkleProof> {
        build_proof(&self.nodes, leaf_index, self.internal_nodes)
    }
    /// Builds a [`crate::MerkleProof`] for given hash. Returns None if hash is not a leaf of tree
    pub fn proof_by_element_hash(&self, hash: &Digest32) -> Option<crate::MerkleProof> {
        let index = *self.elements_hash_index.get(hash)?;
        self.proof_by_index(index)
    }
    /// Builds a [`crate::MerkleProof`] for element, by searching for its hash in the tree
    pub fn proof_by_element(&self, data: &[u8]) -> Option<crate::MerkleProof> {
        let hash = prefixed_hash(LEAF_PREFIX, data);
        self.proof_by_element_hash(&hash)
    }

    /// Builds a [`crate::BatchMerkleProof`] for given indices
    pub fn proof_by_indices(
        &self,
        leaf_indices: &[usize],
    ) -> Option<crate::batchmerkleproof::BatchMerkleProof> {
        let mut leaf_indices = leaf_indices.to_owned();
        leaf_indices.sort_unstable();
        leaf_indices.dedup();
        if leaf_indices.is_empty()
            || leaf_indices
                .iter()
                .any(|i| *i > self.nodes.len() - self.internal_nodes)
        {
            return None;
        }

        build_multiproof(&self.nodes, &leaf_indices, self.internal_nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod test {
    use crate::{prefixed_hash, prefixed_hash2, MerkleNode, MerkleTree};

    #[test]
    fn merkle_tree_zero_elements() {
        let tree = MerkleTree::new(&[][..]);
        assert!(tree.root_hash().as_ref() == [0; 32]);
    }
    #[test]
    fn merkle_tree_test_one_element() {
        // Here the merkle tree looks as follows:
        //      I
        //     / \
        //    L   E
        // Where `I`, `L`, and `E` denote the root internal node, leaf node, and empty node,
        // respectively.

        let data = [1; 32];
        let node = MerkleNode::from_bytes(&data);
        let tree = MerkleTree::new(&[node][..]);
        assert_eq!(
            tree.root_hash(),
            prefixed_hash(1, prefixed_hash(0, &data).as_ref())
        );
    }
    #[test]
    fn merkle_tree_test_five_elements() {
        let bytes = [1u8; 32];
        let nodes = vec![MerkleNode::from_bytes(&bytes); 5];
        let tree = MerkleTree::new(&*nodes);
        let h0x = prefixed_hash(0, &bytes);
        let h10 = prefixed_hash2(1, h0x.as_ref(), h0x.as_ref());
        let h11 = &h10;
        let h12 = prefixed_hash(1, h0x.0.as_ref());
        let h20 = prefixed_hash2(1, h10.as_ref(), h11.as_ref());
        let h21 = prefixed_hash(1, h12.as_ref());
        let h30 = prefixed_hash2(1, h20.as_ref(), h21.as_ref());
        assert_eq!(tree.root_hash(), h30);
    }

    #[test]
    fn merkle_tree_test_merkleproof() {
        let nodes = [
            MerkleNode::from_bytes(&[1; 32]),
            MerkleNode::from_bytes(&[2; 32]),
            MerkleNode::from_bytes(&[3; 32]),
            MerkleNode::from_bytes(&[4; 32]),
            MerkleNode::from_bytes(&[5; 32]),
        ];
        let tree = MerkleTree::new(&nodes[..]);
        let tree_root = tree.root_hash();
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(
                tree.proof_by_index(i).unwrap().get_leaf_data(),
                node.get_leaf_data().unwrap()
            );
            assert!(tree.proof_by_index(i).unwrap().valid(tree_root.as_ref()));
            assert!(tree
                .proof_by_element(node.get_leaf_data().unwrap())
                .unwrap()
                .valid(tree_root.as_ref()));
            assert!(tree
                .proof_by_element_hash(node.get_hash().unwrap())
                .unwrap()
                .valid(tree_root.as_ref()));
        }
    }

    #[cfg(feature = "arbitrary")]
    use proptest::array::uniform32;
    #[cfg(feature = "arbitrary")]
    use proptest::collection::vec;
    #[cfg(feature = "arbitrary")]
    use proptest::prelude::*;
    #[cfg(feature = "arbitrary")]
    proptest! {
        #[test]
        fn merkle_tree_test_arbitrary_proof(data in vec(uniform32(0u8..), 0..1000)) {
            let nodes: Vec<MerkleNode> = data.iter().map(MerkleNode::from_bytes).collect();
            let tree = MerkleTree::new(&*nodes);
            for (i, node) in nodes.iter().enumerate() {
                assert_eq!(
                    tree.proof_by_index(i).unwrap().get_leaf_data(),
                    node.get_leaf_data().unwrap()
                );
                assert!(tree.proof_by_index(i).unwrap().valid(tree.root_hash().as_ref()));
            }
        }
        #[test]
        fn merkle_tree_test_arbitrary_batch_proof(data in vec(uniform32(0u8..), 0..1000), indices in vec(0..1000usize, 0..1000)) {
            let nodes: Vec<MerkleNode> = data.iter().map(MerkleNode::from_bytes).collect();
            let tree = MerkleTree::new(&*nodes);

            let valid = indices.iter().all(|i| *i < data.len()) && indices.len() < data.len() && !indices.is_empty(); // TODO, is there any better strategy for proptest that doesn't require us to filter out invalid indices
            if valid {
                assert!(tree.proof_by_indices(&indices).unwrap().valid(tree.root_hash().as_ref()));
            }
            else {
                assert!(tree.proof_by_indices(&indices).is_none());
            }
        }
    }
}
