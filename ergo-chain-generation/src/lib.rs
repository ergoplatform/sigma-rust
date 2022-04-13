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

use ergo_lib::{ergo_chain_types::BlockId, ergotree_ir::chain::header::Header};
use ergo_lib::{
    ergo_chain_types::Digest32,
    wallet::{ext_secret_key::ExtSecretKey, mnemonic::Mnemonic},
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
