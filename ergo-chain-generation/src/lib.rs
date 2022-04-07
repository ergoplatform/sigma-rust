//! Generate Ergo blockchains for simulation and testing

// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::wildcard_enum_match_arm)]
//#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use ergo_lib::{
    ergo_chain_types::Digest32,
    wallet::{ext_secret_key::ExtSecretKey, mnemonic::Mnemonic},
};
use ergo_lib::{
    ergo_chain_types::{blake2b256_hash, BlockId},
    ergotree_ir::chain::header::Header,
};
use ergo_nipopow::NipopowAlgos;
use num_bigint::{BigInt, Sign};

pub mod chain_generation;
mod fake_pow_scheme;

/// Ergo block
#[derive(Clone, Debug)]
pub struct ErgoFullBlock {
    pub(crate) header: Header,
    //block_transactions: BlockTransactions,
    pub(crate) extension: ExtensionCandidate,
    //ad_proofs: ProofBytes,
}

impl std::convert::TryInto<ergo_nipopow::PoPowHeader> for ErgoFullBlock {
    type Error = &'static str;
    fn try_into(self) -> Result<ergo_nipopow::PoPowHeader, &'static str> {
        let interlinks_proof = match self.extension.proof_for_interlink_vector() {
            Some(proof) => proof,
            None => return Err("Unable to generate BatchMerkleProof for interlinks"),
        };
        let interlinks = unpack_interlinks(&self.extension)?;
        Ok(ergo_nipopow::PoPowHeader {
            header: self.header,
            interlinks,
            interlinks_proof,
        })
    }
}

/// Extension section of Ergo block. Contains key-value storage.
#[derive(Clone, Debug)]
pub struct ExtensionCandidate {
    /// Fields as a sequence of key -> value records. A key is 2-bytes long, value is 64 bytes max.
    pub(crate) fields: Vec<([u8; 2], Vec<u8>)>,
}

impl ExtensionCandidate {
    // TODO: memoize merkletree in extensioncandidate fields?
    fn merkletree(&self) -> ergo_merkle_tree::MerkleTree {
        extension_merkletree(&self.fields)
    }

    /// returns a MerkleProof for a single key element
    pub fn proof_for(&self, key: [u8; 2]) -> Option<ergo_merkle_tree::MerkleProof> {
        let tree = self.merkletree();
        let kv = self.fields.iter().find(|(k, _)| *k == key)?;
        tree.proof_by_element(&kv_to_leaf(kv))
    }

    /// Returns a BatchMerkleProof (compact multi-proof) for multiple key elements
    pub fn batch_proof_for(&self, keys: &[[u8; 2]]) -> Option<ergo_merkle_tree::BatchMerkleProof> {
        let tree = self.merkletree();
        let indices: Vec<usize> = keys
            .iter()
            .flat_map(|k| self.fields.iter().find(|(key, _)| key == k))
            .map(kv_to_leaf)
            .map(ergo_merkle_tree::MerkleNode::from)
            .flat_map(|node| node.get_hash().copied())
            .flat_map(|hash| tree.get_elements_hash_index().get(&hash).copied())
            .collect();
        tree.proof_by_indices(&indices)
    }
    pub(crate) fn proof_for_interlink_vector(&self) -> Option<ergo_merkle_tree::BatchMerkleProof> {
        let interlinks: Vec<[u8; 2]> = self
            .fields
            .iter()
            .map(|(key, _)| *key)
            .filter(|key| key[0] == INTERLINK_VECTOR_PREFIX)
            .collect();
        if interlinks.is_empty() {
            dbg!("Making empty tree");
            Some(ergo_merkle_tree::BatchMerkleProof::new(vec![], vec![]))
        } else {
            self.batch_proof_for(&interlinks)
        }
    }
}

static INTERLINK_VECTOR_PREFIX: u8 = 0x01;

// converts a key value pair to an array of [key.length, key, val]
fn kv_to_leaf(kv: &([u8; 2], Vec<u8>)) -> Vec<u8> {
    std::iter::once(2)
        .chain(kv.0.into_iter())
        .chain(kv.1.iter().copied())
        .collect()
}
// creates a MerkleTree from a key/value pair of extension section
pub(crate) fn extension_merkletree(kv: &Vec<([u8; 2], Vec<u8>)>) -> ergo_merkle_tree::MerkleTree {
    let leafs = kv
        .iter()
        .map(kv_to_leaf)
        .map(ergo_merkle_tree::MerkleNode::from)
        .collect::<Vec<ergo_merkle_tree::MerkleNode>>();
    ergo_merkle_tree::MerkleTree::new(&leafs)
}

/// Unpacks interlinks from key-value format of block extension.
pub(crate) fn unpack_interlinks(
    extension: &ExtensionCandidate,
) -> Result<Vec<BlockId>, &'static str> {
    let mut res = vec![];
    let entries = extension
        .fields
        .iter()
        .filter(|&(key, _)| key[0] == INTERLINK_VECTOR_PREFIX);
    for (_, bytes) in entries {
        // Each interlink is packed as [qty | blockId], which qty is a single-byte value
        // representing the number of duplicates of `blockId`. Every `BlockId` is 32 bytes which
        // implies that `bytes` is 33 bytes.
        if bytes.len() != 33 {
            return Err("Interlinks must be 33 bytes in size");
        }
        let qty = bytes[0];
        let block_id_bytes: [u8; 32] = bytes[1..].try_into().unwrap();
        let block_id = BlockId(Digest32::from(block_id_bytes));
        res.extend(std::iter::repeat(block_id).take(qty as usize));
    }
    Ok(res)
}

/// Computes interlinks vector for a header next to `prevHeader`.
pub(crate) fn update_interlinks(
    prev_header: Header,
    prev_interlinks: Vec<BlockId>,
) -> Vec<BlockId> {
    let is_genesis = prev_header.height == 1;
    if !is_genesis {
        // Interlinks vector cannot be empty in case of non-genesis header
        assert!(!prev_interlinks.is_empty());
        let genesis = prev_interlinks[0].clone();
        let nipopow_algos = NipopowAlgos::default();
        let prev_level = nipopow_algos.max_level_of(&prev_header).unwrap() as usize;
        if prev_level > 0 {
            // Adapted:
            //   `(genesis +: tail.dropRight(prevLevel)) ++Seq.fill(prevLevel)(prevHeader.id)`
            // from scala
            if prev_interlinks.len() > prev_level {
                std::iter::once(genesis)
                    .chain(
                        prev_interlinks[1..(prev_interlinks.len() - prev_level)]
                            .iter()
                            .cloned(),
                    )
                    .chain(std::iter::repeat(prev_header.id).take(prev_level))
                    .collect()
            } else {
                std::iter::once(genesis)
                    .chain(std::iter::repeat(prev_header.id).take(prev_level))
                    .collect()
            }
        } else {
            prev_interlinks
        }
    } else {
        vec![prev_header.id]
    }
}

/// Returns the secret key of the miner secret with its `BigInt` representation. Taken from ergo
/// test suite.
pub(crate) fn default_miner_secret() -> (ExtSecretKey, BigInt) {
    let test_mnemonic =
        "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic";
    let seed = Mnemonic::to_seed(test_mnemonic, "");
    let default_root_secret = ExtSecretKey::derive_master(seed).unwrap();
    let bytes = default_root_secret.secret_key_bytes();
    (
        default_root_secret,
        BigInt::from_bytes_be(Sign::Plus, &bytes),
    )
}

/// This is currently use to generate the transaction root for a `Header`. Can remove this once the
/// Merkle tree is implemented `ergo-merkle-tree` crate.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum MerkleTreeNode {
    Internal {
        left: Box<MerkleTreeNode>,
        right: Box<MerkleTreeNode>,
    },
    Leaf(Vec<u8>),
    Empty,
    EmptyTree,
}

impl MerkleTreeNode {
    pub(crate) fn new(leaves: Vec<Vec<u8>>) -> Self {
        if leaves.is_empty() {
            return MerkleTreeNode::EmptyTree;
        }
        let leaves: Vec<_> = leaves.into_iter().map(MerkleTreeNode::Leaf).collect();
        let mut stack = vec![leaves];

        while let Some(nodes) = stack.pop() {
            let mut next_level = vec![];
            for chunks in nodes.chunks(2) {
                if chunks.len() == 2 {
                    next_level.push(MerkleTreeNode::Internal {
                        left: Box::new(chunks[0].clone()),
                        right: Box::new(chunks[1].clone()),
                    });
                } else {
                    next_level.push(MerkleTreeNode::Internal {
                        left: Box::new(chunks[0].clone()),
                        right: Box::new(MerkleTreeNode::Empty),
                    });
                }
            }
            if next_level.len() == 1 {
                let root = next_level.pop().unwrap();
                return root;
            } else {
                stack.push(next_level);
            }
        }
        unreachable!()
    }

    /// Computes the hash of the merkle tree with `self` as root.
    pub(crate) fn hash(&self) -> [u8; 32] {
        let mut computed_children_nodes = vec![];
        enum StackPushType {
            /// Children of the node not yet pushed onto `stack`
            First,
            /// Second time on the stack
            ChildrenAlreadyPushed,
        }

        let mut stack = vec![(self.clone(), StackPushType::First)];
        while let Some((n, push_type)) = stack.pop() {
            match n {
                MerkleTreeNode::Internal {
                    ref left,
                    ref right,
                } => {
                    match push_type {
                        StackPushType::First => {
                            stack.push((n.clone(), StackPushType::ChildrenAlreadyPushed));
                            stack.push((*left.clone(), StackPushType::First));
                            stack.push((*right.clone(), StackPushType::First));
                        }
                        StackPushType::ChildrenAlreadyPushed => {
                            // Note we pop off left child first
                            let computed_left_child = computed_children_nodes.pop().unwrap();
                            if let Some(computed_right_child) = computed_children_nodes.pop() {
                                let concat =
                                    concatenate_hashes(&computed_left_child, &computed_right_child);
                                let hash = prefixed_hash(NodePrefix::Internal as u8, &concat);
                                if stack.is_empty() {
                                    return *hash;
                                } else {
                                    computed_children_nodes.push(*hash);
                                }
                            } else {
                                let internal_hash =
                                    prefixed_hash(NodePrefix::Internal as u8, &computed_left_child);
                                if stack.is_empty() {
                                    return *internal_hash;
                                } else {
                                    computed_children_nodes.push(*internal_hash);
                                }
                            }
                        }
                    }
                }
                MerkleTreeNode::Leaf(data) => match push_type {
                    StackPushType::First => {
                        let hash = prefixed_hash(NodePrefix::Leaf as u8, &data);
                        computed_children_nodes.push(*hash);
                    }
                    StackPushType::ChildrenAlreadyPushed => {
                        unreachable!()
                    }
                },
                MerkleTreeNode::Empty => (),
                MerkleTreeNode::EmptyTree => {
                    return *blake2b256_hash(&[]).0;
                }
            }
        }
        unreachable!()
    }
}

/// Each node in the merkle tree contains a 'prefixed' hash. This is done to give 'second preimage
/// resistance'. More details can be found here <https://en.bitcoinwiki.org/wiki/Merkle_tree>
enum NodePrefix {
    /// Leaf nodes of the tree are hashed as `hash(0 ++ data)` where `++` denotes concatencation and
    /// `data` is a byte array of the data.
    Leaf = 0,
    /// Internal nodes of the tree are hashed as `hash(1 ++ data_left_child ++ data_right_child)`.
    Internal = 1,
}

// Generates a hash of data prefixed with `prefix`
fn prefixed_hash(prefix: u8, data: &[u8]) -> Box<[u8; 32]> {
    let mut hasher = VarBlake2b::new(32).unwrap();
    hasher.update(&[prefix]);
    hasher.update(data);
    let hash = hasher.finalize_boxed();
    hash.try_into().unwrap()
}

fn concatenate_hashes(hash_a: &[u8; 32], hash_b: &[u8; 32]) -> [u8; 64] {
    let mut sum = [0; 64];
    sum[0..32].clone_from_slice(hash_a);
    sum[32..].clone_from_slice(hash_b);
    sum
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::{concatenate_hashes, prefixed_hash, MerkleTreeNode, NodePrefix};

    #[test]
    fn test_single_leaf_merkle_tree() {
        // Here the merkle tree looks as follows:
        //      I
        //     / \
        //    L   E
        // Where `I`, `L`, and `E` denote the root internal node, leaf node, and empty node,
        // respectively.
        let leaf_data: Vec<u8> = std::iter::repeat(0).take(32).collect();
        let leaf_hash = prefixed_hash(NodePrefix::Leaf as u8, &leaf_data);
        let root_hash = prefixed_hash(NodePrefix::Internal as u8, &*leaf_hash);

        let merkle_root_node = MerkleTreeNode::new(vec![leaf_data]);

        assert_eq!(merkle_root_node.hash(), *root_hash);
    }

    #[test]
    fn test_2_leaf_merkle_tree() {
        // Here the merkle tree looks as follows:
        //      I
        //     / \
        //   L0   L1
        // Where `I`, `L0`, and `1` denote the root internal node, first leaf node, and 2nd leaf node,
        // respectively.
        let leaf0_data: Vec<u8> = std::iter::repeat(0).take(32).collect();
        let leaf1_data: Vec<u8> = std::iter::repeat(1).take(32).collect();
        let leaf0_hash = prefixed_hash(NodePrefix::Leaf as u8, &leaf0_data);
        let leaf1_hash = prefixed_hash(NodePrefix::Leaf as u8, &leaf1_data);
        let root_hash = prefixed_hash(
            NodePrefix::Internal as u8,
            &concatenate_hashes(&*leaf0_hash, &*leaf1_hash),
        );

        let merkle_root_node = MerkleTreeNode::new(vec![leaf0_data, leaf1_data]);

        assert_eq!(merkle_root_node.hash(), *root_hash);
    }

    #[test]
    fn test_3_leaf_merkle_tree() {
        // Here the merkle tree looks as follows:
        //          I
        //         / \
        //        /   \
        //       /     \
        //      I0      I1
        //     / \     / \
        //   L0   L1  L2  E
        let leaf0_data: Vec<u8> = std::iter::repeat(0).take(32).collect();
        let leaf1_data: Vec<u8> = std::iter::repeat(1).take(32).collect();
        let leaf2_data: Vec<u8> = std::iter::repeat(2).take(32).collect();
        let leaf0_hash = prefixed_hash(NodePrefix::Leaf as u8, &leaf0_data);
        let leaf1_hash = prefixed_hash(NodePrefix::Leaf as u8, &leaf1_data);
        let leaf2_hash = prefixed_hash(NodePrefix::Leaf as u8, &leaf2_data);

        // I0
        let i0_hash = prefixed_hash(
            NodePrefix::Internal as u8,
            &concatenate_hashes(&*leaf0_hash, &*leaf1_hash),
        );

        // I1
        let i1_hash = prefixed_hash(NodePrefix::Internal as u8, &*leaf2_hash);

        let root_hash = prefixed_hash(
            NodePrefix::Internal as u8,
            &concatenate_hashes(&*i0_hash, &*i1_hash),
        );

        let merkle_root_node = MerkleTreeNode::new(vec![leaf0_data, leaf1_data, leaf2_data]);

        assert_eq!(merkle_root_node.hash(), *root_hash);
    }
}
