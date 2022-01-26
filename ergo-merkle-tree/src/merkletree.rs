use crate::{prefixed_hash, prefixed_hash2};
use crate::{HASH_SIZE, INTERNAL_PREFIX, LEAF_PREFIX};
use std::collections::BTreeSet;

/// Node for a Merkle Tree
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub enum MerkleNode {
    Node {
        hash: [u8; HASH_SIZE],
    },
    Leaf {
        hash: [u8; HASH_SIZE],
        data: [u8; 32],
    },
    EmptyNode,
}

impl MerkleNode {
    /// Creates a new Leaf Node from bytes. The hash is prefixed with a leaf node prefix. Fails if data is not 32 bytes in size
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, std::array::TryFromSliceError> {
        let hash = *prefixed_hash(LEAF_PREFIX, bytes.as_ref());
        Ok(MerkleNode::Leaf {
            hash,
            data: bytes.as_ref().try_into()?,
        })
    }
    /// Gets hash for the node, returns None if it's an Empty Node
    pub fn get_hash(&self) -> Option<&[u8; 32]> {
        match self {
            MerkleNode::Node { hash } => Some(hash),
            MerkleNode::Leaf { hash, .. } => Some(hash),
            _ => None,
        }
    }
    /// Gets data for the node if it's a leaf node
    pub fn get_leaf_data(&self) -> Option<&[u8; 32]> {
        match self {
            MerkleNode::Leaf { data, .. } => Some(data),
            _ => None,
        }
    }
    pub(crate) fn empty() -> Self {
        Self::EmptyNode
    }
}

/// Merkle Tree
#[derive(Debug)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub struct MerkleTree {
    nodes: Vec<MerkleNode>,
    internal_nodes: usize,
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
            Some(hash) => proof_nodes.push(crate::LevelNode::new(*hash, side)),
            _ => proof_nodes.push(crate::LevelNode::empty_node(side)),
        }
        leaf_index = get_parent(leaf_index)?;
    }

    crate::MerkleProof::new(leaf_data, &proof_nodes).ok()
}

fn build_multiproof(
    nodes: &[MerkleNode],
    leaf_indices: &[usize],
    internal_nodes: usize,
) -> Option<crate::batchmerkleproof::BatchMerkleProof> {
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
                Some(hash) => crate::LevelNode::new(*hash, side),
                None => crate::LevelNode::empty_node(side),
            };
            multiproof.push(levelnode);
        }
        a = b_pruned.into_iter().flat_map(get_parent).collect();
    }

    Some(crate::batchmerkleproof::BatchMerkleProof::new(
        leaf_indices
            .iter()
            .flat_map(|idx| Some((*idx, nodes[idx + internal_nodes].get_hash().copied()?)))
            .collect(),
        multiproof,
    ))
}

impl MerkleTree {
    /// Creates a new MerkleTree from leaf hashes in nodes
    pub fn new(nodes: &[MerkleNode]) -> Self {
        #[allow(clippy::unwrap_used)]
        fn build_nodes(nodes: &mut [MerkleNode]) {
            for pair in (1..nodes.len()).step_by(2).rev() {
                let node = match (
                    nodes[pair].get_hash(),
                    nodes[get_sibling(pair).unwrap()].get_hash(), // since we pad nodes with no sibling with an empty node, get_sibling should always return Some
                ) {
                    (Some(left_hash), Some(right_hash)) => MerkleNode::Node {
                        hash: *prefixed_hash2(INTERNAL_PREFIX, &left_hash[..], &right_hash[..]),
                    },
                    (Some(hash), None) => MerkleNode::Node {
                        hash: *prefixed_hash(INTERNAL_PREFIX, hash),
                    },
                    (None, None) => MerkleNode::EmptyNode,
                    _ => unreachable!(),
                };
                nodes[get_parent(pair).unwrap()] = node;
            }
        }
        let mut nodes = nodes.to_owned();
        if nodes.len() % 2 == 1 {
            nodes.push(MerkleNode::EmptyNode);
        }
        let leaf_nodes = nodes.len();
        let mut tree_nodes = vec![MerkleNode::empty(); nodes.len().next_power_of_two() - 1];
        tree_nodes.extend_from_slice(&nodes);
        build_nodes(&mut tree_nodes);
        let nodes_len = tree_nodes.len();
        Self {
            nodes: tree_nodes,
            internal_nodes: nodes_len - leaf_nodes,
        }
    }

    pub fn get_root_hash(&self) -> Option<&[u8; 32]> {
        self.nodes.get(0).and_then(MerkleNode::get_hash)
    }

    pub fn proof_by_index(&self, leaf_index: usize) -> Option<crate::MerkleProof> {
        build_proof(&self.nodes, leaf_index, self.internal_nodes)
    }

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
    use crate::{concatenate_hashes, prefixed_hash, MerkleNode, MerkleTree};

    // TODO: comparing against scala implementation, it creates a root hash of 0's instead of a non-existent hash
    #[test]
    fn merkle_tree_zero_elements() {
        let tree = MerkleTree::new(&[]);
        assert!(tree.get_root_hash().is_none());
    }
    #[test]
    fn merkle_tree_test_one_element() {
        let data = [1; 32];
        let node = MerkleNode::from_bytes(&data).unwrap();
        let tree = MerkleTree::new(&[node]);
        assert_eq!(
            tree.get_root_hash().unwrap(),
            &*prefixed_hash(1, &*prefixed_hash(0, &data))
        );
    }
    #[test]
    fn merkle_tree_test_five_elements() {
        let bytes = [1u8; 32];
        let nodes = [MerkleNode::from_bytes(&bytes).unwrap(); 5];
        let tree = MerkleTree::new(&nodes);
        let h0x = prefixed_hash(0, &bytes);
        let h10 = prefixed_hash(1, &concatenate_hashes(&*h0x, &*h0x));
        let h11 = &h10;
        let h12 = prefixed_hash(1, &*h0x);
        let h20 = prefixed_hash(1, &concatenate_hashes(&*h10, h11));
        let h21 = prefixed_hash(1, &*h12);
        let h30 = prefixed_hash(1, &concatenate_hashes(&*h20, &*h21));
        assert_eq!(tree.get_root_hash().unwrap(), &*h30);
    }

    #[test]
    fn merkle_tree_test_merkleproof() {
        let nodes = [
            MerkleNode::from_bytes(&[1; 32]).unwrap(),
            MerkleNode::from_bytes(&[2; 32]).unwrap(),
            MerkleNode::from_bytes(&[3; 32]).unwrap(),
            MerkleNode::from_bytes(&[4; 32]).unwrap(),
            MerkleNode::from_bytes(&[5; 32]).unwrap(),
        ];
        let tree = MerkleTree::new(&nodes);
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(
                tree.proof_by_index(i).unwrap().get_leaf_data(),
                node.get_leaf_data().unwrap()
            );
            assert!(tree
                .proof_by_index(i)
                .unwrap()
                .valid(tree.get_root_hash().unwrap()));
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
            let nodes: Vec<MerkleNode> = data.iter().map(MerkleNode::from_bytes).map(Result::unwrap).collect();
            let tree = MerkleTree::new(&nodes);
            for (i, node) in nodes.iter().enumerate() {
                assert_eq!(
                    tree.proof_by_index(i).unwrap().get_leaf_data(),
                    node.get_leaf_data().unwrap()
                );
                assert!(tree.proof_by_index(i).unwrap().valid(tree.get_root_hash().unwrap()));
            }
        }
        #[test]
        fn merkle_tree_test_arbitrary_batch_proof(data in vec(uniform32(0u8..), 0..1000), indices in vec(0..1000usize, 0..1000)) {
            let nodes: Vec<MerkleNode> = data.iter().map(MerkleNode::from_bytes).map(Result::unwrap).collect();
            let tree = MerkleTree::new(&nodes);

            let valid = indices.iter().all(|i| *i < data.len()) && indices.len() < data.len(); // TODO, is there any better strategy for proptest that doesn't require us to filter out invalid indices
            if valid {
                assert!(tree.proof_by_indices(&indices).unwrap().valid(tree.get_root_hash().unwrap()));
            }
            else {
                assert!(tree.proof_by_indices(&indices).is_none());
            }
        }
    }
}
