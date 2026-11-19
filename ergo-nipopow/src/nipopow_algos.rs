use ergo_chain_types::{
    autolykos_pow_scheme::{
        decode_compact_bits, order_bigint, AutolykosPowScheme, AutolykosPowSchemeError,
    },
    Header,
};
use num_traits::ToPrimitive;
use std::convert::TryInto;

use crate::{nipopow_proof::PoPowHeader, NipopowProof, NipopowProofError};
use ergo_chain_types::{BlockId, Digest32, ExtensionCandidate};

/// Prefix for Block Interlinks
pub const INTERLINK_VECTOR_PREFIX: u8 = 0x01;
/// A set of utilities for working with NiPoPoW protocol.
///
/// Based on papers:
///
/// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
///
/// [`KLS16`]: http://fc16.ifca.ai/bitcoin/papers/KLS16.pdf
///
/// Please note that for KMZ17 we're using the version published @ Financial Cryptography 2020,
/// which is different from previously published versions on IACR eprint.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct NipopowAlgos {
    /// The proof-of-work scheme
    pub pow_scheme: AutolykosPowScheme,
}

impl NipopowAlgos {
    /// Computes best score of a given chain.
    /// The score value depends on number of µ-superblocks in the given chain.
    ///
    /// see [`KMZ17`], Algorithm 4
    ///
    /// [`KMZ17`]:
    /// "To find the best argument of a proof π given b, best-arg_m collects all the μ
    /// indices which point to superblock levels that contain valid arguments after block b.
    /// Argument validity requires that there are at least m μ-superblocks following block b,
    /// which is captured by the comparison|π↑μ{b:}|≥m. 0 is always considered a valid level,
    /// regardless of how many blocks are present there. These level indices are collected into set M.
    /// For each of these levels, the score of their respective argument is evaluated by weighting the
    /// number of blocks by the level as 2μ|π↑μ{b:}|. The highest possible score across all levels is returned."
    ///
    /// function best-arg_m(π, b)
    /// M←{μ:|π↑μ{b:}|≥m}∪{0}
    /// return max_{μ∈M} {2μ·|π↑μ{b:}|}
    /// end function
    ///
    /// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
    pub fn best_arg(&self, chain: &[&Header], m: u32) -> Result<usize, AutolykosPowSchemeError> {
        // Little helper struct for loop below
        struct Acc {
            level: u32,
            acc: Vec<(u32, usize)>,
        }
        let mut res = Acc {
            level: 1,
            acc: vec![(0, chain.len())],
        };
        let acc = loop {
            let mut args = vec![];
            for h in chain {
                if (self.max_level_of(h)? as u32) >= res.level {
                    args.push(h);
                }
            }
            // The `!args.is_empty()` condition only matters for m == 0, which
            // the reference implementation rejects with `require(m >= 1)` and
            // which NipopowProof's constructors/validity checks reject as
            // well: with no headers at the current level, no higher level can
            // have any either, so the loop must stop instead of counting
            // empty levels forever.
            if args.len() >= (m as usize) && !args.is_empty() {
                res.acc.insert(0, (res.level, args.len()));
                res = Acc {
                    level: res.level + 1,
                    acc: res.acc,
                };
            } else {
                break res.acc;
            }
        };
        #[allow(clippy::unwrap_used)]
        Ok(acc
            .into_iter()
            .map(|(level, size)| {
                // 2^µ * |C↑µ|
                2usize.pow(level) * size
            })
            .max()
            .unwrap())
    }

    /// Computes max level (μ) of the given header, such that μ = log(T) − log(id(B))
    pub fn max_level_of(&self, header: &Header) -> Result<i32, AutolykosPowSchemeError> {
        let genesis_header = header.height == 1;
        if !genesis_header {
            // Order of the secp256k1 elliptic curve
            let order = order_bigint();
            #[allow(clippy::unwrap_used)]
            let required_target = (order / decode_compact_bits(header.n_bits))
                .to_f64()
                .unwrap();
            #[allow(clippy::unwrap_used)]
            let real_target = self.pow_scheme.pow_hit(header)?.to_f64().unwrap();
            let level = required_target.log2() - real_target.log2();
            Ok(level as i32)
        } else {
            Ok(i32::MAX)
        }
    }

    /// Finds the last common header (branching point) between `left_chain` and `right_chain`.
    pub fn lowest_common_ancestor(
        &self,
        left_chain: &[&Header],
        right_chain: &[&Header],
    ) -> Option<Header> {
        if let Some(head_left) = left_chain.first() {
            if let Some(head_right) = right_chain.first() {
                if *head_left != *head_right {
                    return None;
                }
            }
        }
        let mut common = vec![];
        let mut right_ix_start = 0;
        for left_header in left_chain {
            let start_ix = right_ix_start;
            for (i, right_header) in right_chain.iter().enumerate().skip(start_ix) {
                if **left_header == **right_header {
                    right_ix_start = i + 1;
                    common.push(*left_header);
                }
            }
        }
        common.last().cloned().cloned()
    }

    /// Computes NiPoPow proof for the given `chain` according to given `params`.
    pub fn prove(
        &self,
        chain: &[PoPowHeader],
        k: u32,
        m: u32,
    ) -> Result<NipopowProof, NipopowProofError> {
        if k == 0 {
            return Err(NipopowProofError::ZeroKParameter);
        }
        if m == 0 {
            return Err(NipopowProofError::ZeroMParameter);
        }
        if chain.len() < ((k + m) as usize) {
            return Err(NipopowProofError::ChainTooShort);
        }
        if chain[0].header.height != 1 {
            return Err(NipopowProofError::NonAnchoredChain);
        }

        let suffix = chain[(chain.len() - (k as usize))..].to_vec();
        let suffix_head = suffix[0].clone();
        let suffix_tail: Vec<Header> = suffix[1..].iter().map(|p| p.header.clone()).collect();
        #[allow(clippy::unwrap_used)]
        let max_level: i32 = if chain.len() > (k as usize) {
            (chain[..(chain.len() - (k as usize))]
                .last()
                .unwrap()
                .interlinks
                .len()
                - 1) as i32
        } else {
            return Err(NipopowProofError::ChainTooShort);
        };

        // Here is non-recursive implementation of the scala `provePrefix` function
        let mut prefix = vec![];
        let mut stack = vec![(chain[0].clone(), max_level)];
        while let Some((anchoring_point, level)) = stack.pop() {
            if level >= 0 {
                // C[:−k]{B:}↑µ
                let mut sub_chain = vec![];

                for p in &chain[..(chain.len() - (k as usize))] {
                    let max_level = self.max_level_of(&p.header)?;
                    if max_level >= level && p.header.height >= anchoring_point.header.height {
                        sub_chain.push(p.clone());
                    }
                }

                if (m as usize) < sub_chain.len() {
                    stack.push((sub_chain[sub_chain.len() - (m as usize)].clone(), level - 1));
                } else {
                    stack.push((anchoring_point, level - 1));
                }
                for pph in sub_chain {
                    if !prefix.contains(&pph) {
                        prefix.push(pph);
                    }
                }
            }
        }
        prefix.sort_by(|a, b| a.header.height.cmp(&b.header.height));
        NipopowProof::new(m, k, prefix, suffix_head, suffix_tail)
    }
    /// Packs interlinks into key-value format of the block extension.
    ///
    /// Each run of equal block ids is stored under a key whose second byte is
    /// the index of the run's first occurrence in the vector, mirroring the
    /// reference implementation (`NipopowAlgos.packInterlinks` in the Ergo
    /// node, which keys each entry by the `zipWithIndex` position of the run's
    /// head). The packed value is the run length followed by the block id.
    pub fn pack_interlinks(interlinks: Vec<BlockId>) -> Vec<([u8; 2], Vec<u8>)> {
        let mut res = vec![];
        let mut run_start_ix = 0;
        while run_start_ix < interlinks.len() {
            let curr_block_id = interlinks[run_start_ix];
            // The count is packed into a single byte; wrapping matches the
            // reference implementation's `duplicatesQty.toByte`.
            let mut curr_block_id_count: u8 = 1;
            let mut next_ix = run_start_ix + 1;
            while next_ix < interlinks.len() && interlinks[next_ix] == curr_block_id {
                curr_block_id_count = curr_block_id_count.wrapping_add(1);
                next_ix += 1;
            }
            let block_id_bytes: Vec<u8> = curr_block_id.0.into();
            let packed_value = std::iter::once(curr_block_id_count)
                .chain(block_id_bytes)
                .collect();
            res.push(([INTERLINK_VECTOR_PREFIX, run_start_ix as u8], packed_value));
            run_start_ix = next_ix;
        }
        res
    }
    /// Unpacks interlinks from key-value format of block extension.
    pub fn unpack_interlinks(extension: &ExtensionCandidate) -> Result<Vec<BlockId>, &'static str> {
        let mut res = vec![];
        let entries = extension
            .fields()
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
            let block_id_bytes: [u8; 32] = bytes[1..]
                .try_into()
                .map_err(|_| "Expected 32 byte BlockId")?;
            let block_id = BlockId(Digest32::from(block_id_bytes));
            res.extend(std::iter::repeat_n(block_id, qty as usize));
        }
        Ok(res)
    }

    /// Computes interlinks vector for a header next to `prevHeader`.
    pub fn update_interlinks(
        prev_header: Header,
        prev_interlinks: Vec<BlockId>,
    ) -> Result<Vec<BlockId>, AutolykosPowSchemeError> {
        let is_genesis = prev_header.height == 1;
        if !is_genesis {
            // Interlinks vector cannot be empty in case of non-genesis header
            assert!(!prev_interlinks.is_empty());
            let genesis = prev_interlinks[0];
            let nipopow_algos = NipopowAlgos::default();
            let prev_level = nipopow_algos.max_level_of(&prev_header)? as usize;
            if prev_level > 0 {
                // Adapted:
                //   `(genesis +: tail.dropRight(prevLevel)) ++Seq.fill(prevLevel)(prevHeader.id)`
                // from scala
                if prev_interlinks.len() > prev_level {
                    Ok(std::iter::once(genesis)
                        .chain(
                            prev_interlinks[1..(prev_interlinks.len() - prev_level)]
                                .iter()
                                .cloned(),
                        )
                        .chain(std::iter::repeat_n(prev_header.id, prev_level))
                        .collect())
                } else {
                    Ok(std::iter::once(genesis)
                        .chain(std::iter::repeat_n(prev_header.id, prev_level))
                        .collect())
                }
            } else {
                Ok(prev_interlinks)
            }
        } else {
            Ok(vec![prev_header.id])
        }
    }
    /// Returns [`ergo_merkle_tree::BatchMerkleProof`] for block interlinks
    pub fn proof_for_interlink_vector(
        ext: &ExtensionCandidate,
    ) -> Option<ergo_merkle_tree::BatchMerkleProof> {
        let interlinks: Vec<[u8; 2]> = ext
            .fields()
            .iter()
            .map(|(key, _)| *key)
            .filter(|key| key[0] == INTERLINK_VECTOR_PREFIX)
            .collect();
        if interlinks.is_empty() {
            Some(ergo_merkle_tree::BatchMerkleProof::new(vec![], vec![]))
        } else {
            NipopowAlgos::extension_batch_proof_for(ext, &interlinks)
        }
    }
    /// returns a MerkleProof for a single key element of [`ExtensionCandidate`]
    pub fn extension_proof_for(
        ext: &ExtensionCandidate,
        key: [u8; 2],
    ) -> Option<ergo_merkle_tree::MerkleProof> {
        let tree = extension_merkletree(ext.fields());
        let kv = ext.fields().iter().find(|(k, _)| *k == key)?;
        tree.proof_by_element(&kv_to_leaf(kv))
    }
    /// Returns a [`ergo_merkle_tree::BatchMerkleProof`] (compact multi-proof) for multiple key elements of [`ExtensionCandidate`]
    pub fn extension_batch_proof_for(
        ext: &ExtensionCandidate,
        keys: &[[u8; 2]],
    ) -> Option<ergo_merkle_tree::BatchMerkleProof> {
        let tree = extension_merkletree(ext.fields());
        let indices: Vec<usize> = keys
            .iter()
            .flat_map(|k| ext.fields().iter().find(|(key, _)| key == k))
            .map(kv_to_leaf)
            .map(ergo_merkle_tree::MerkleNode::from_bytes)
            .flat_map(|node| node.get_hash().cloned())
            .flat_map(|hash| tree.get_elements_hash_index().get(&hash).copied())
            .collect();
        tree.proof_by_indices(&indices)
    }
}

// converts a key value pair to an array of [key.length, key, val]
fn kv_to_leaf(kv: &([u8; 2], Vec<u8>)) -> Vec<u8> {
    std::iter::once(2u8)
        .chain(kv.0.iter().copied())
        .chain(kv.1.iter().copied())
        .collect()
}
// creates a MerkleTree from a key/value pair of extension section
fn extension_merkletree(kv: &[([u8; 2], Vec<u8>)]) -> ergo_merkle_tree::MerkleTree {
    let leafs = kv
        .iter()
        .map(kv_to_leaf)
        .map(ergo_merkle_tree::MerkleNode::from_bytes)
        .collect::<Vec<ergo_merkle_tree::MerkleNode>>();
    ergo_merkle_tree::MerkleTree::new(leafs)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ergo_chain_types::ExtensionCandidate;

    fn block_id(byte: u8) -> BlockId {
        BlockId(Digest32::from([byte; 32]))
    }

    fn packed_value(count: u8, byte: u8) -> Vec<u8> {
        std::iter::once(count)
            .chain(vec![byte; 32])
            .collect::<Vec<u8>>()
    }

    #[test]
    fn pack_interlinks_empty() {
        // The reference implementation returns an empty sequence for an
        // empty interlinks vector; this used to panic with an
        // index-out-of-bounds on `interlinks[0]`.
        assert!(NipopowAlgos::pack_interlinks(vec![]).is_empty());
    }

    #[test]
    fn pack_interlinks_distinct_ids_keyed_by_position() {
        let packed = NipopowAlgos::pack_interlinks(vec![block_id(1), block_id(2), block_id(3)]);
        assert_eq!(
            packed,
            vec![
                ([INTERLINK_VECTOR_PREFIX, 0], packed_value(1, 1)),
                ([INTERLINK_VECTOR_PREFIX, 1], packed_value(1, 2)),
                ([INTERLINK_VECTOR_PREFIX, 2], packed_value(1, 3)),
            ]
        );
    }

    #[test]
    fn pack_interlinks_duplicate_run_keyed_by_first_occurrence() {
        // Reference form for [A, A, B]: the B entry is keyed by 2, the index
        // of its first (and only) occurrence, not by 1, its run ordinal.
        let packed = NipopowAlgos::pack_interlinks(vec![block_id(7), block_id(7), block_id(9)]);
        assert_eq!(
            packed,
            vec![
                ([INTERLINK_VECTOR_PREFIX, 0], packed_value(2, 7)),
                ([INTERLINK_VECTOR_PREFIX, 2], packed_value(1, 9)),
            ]
        );
    }

    #[test]
    fn pack_interlinks_single_run() {
        let packed = NipopowAlgos::pack_interlinks(vec![block_id(5), block_id(5), block_id(5)]);
        assert_eq!(
            packed,
            vec![([INTERLINK_VECTOR_PREFIX, 0], packed_value(3, 5))]
        );
    }

    #[test]
    fn pack_unpack_interlinks_roundtrip_with_duplicate_runs() {
        let interlinks = vec![
            block_id(1),
            block_id(1),
            block_id(2),
            block_id(3),
            block_id(3),
            block_id(3),
        ];
        let extension =
            ExtensionCandidate::new(NipopowAlgos::pack_interlinks(interlinks.clone())).unwrap();
        assert_eq!(
            NipopowAlgos::unpack_interlinks(&extension).unwrap(),
            interlinks
        );
    }
}
