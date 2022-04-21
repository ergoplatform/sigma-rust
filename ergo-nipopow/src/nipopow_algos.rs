use ergotree_ir::{chain::header::Header, sigma_protocol::dlog_group::order};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::convert::TryInto;

use crate::{
    autolykos_pow_scheme::{AutolykosPowScheme, AutolykosPowSchemeError},
    nipopow_proof::PoPowHeader,
    NipopowProof, NipopowProofError,
};
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
            if args.len() >= (m as usize) {
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
            let order = order();
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
    pub fn pack_interlinks(interlinks: Vec<BlockId>) -> Vec<([u8; 2], Vec<u8>)> {
        let mut res = vec![];
        let mut ix_distinct_block_ids = 0;
        let mut curr_block_id_count = 1;
        let mut curr_block_id = interlinks[0].clone();
        for id in interlinks.into_iter().skip(1) {
            if id == curr_block_id {
                curr_block_id_count += 1;
            } else {
                let block_id_bytes: Vec<u8> = curr_block_id.clone().0.into();
                let packed_value = std::iter::once(curr_block_id_count)
                    .chain(block_id_bytes)
                    .collect();
                res.push((
                    [INTERLINK_VECTOR_PREFIX, ix_distinct_block_ids],
                    packed_value,
                ));
                curr_block_id = id;
                curr_block_id_count = 1;
                ix_distinct_block_ids += 1;
            }
        }
        let block_id_bytes: Vec<u8> = curr_block_id.0.into();
        let packed_value = std::iter::once(curr_block_id_count)
            .chain(block_id_bytes)
            .collect();
        res.push((
            [INTERLINK_VECTOR_PREFIX, ix_distinct_block_ids],
            packed_value,
        ));
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
            res.extend(std::iter::repeat(block_id).take(qty as usize));
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
            let genesis = prev_interlinks[0].clone();
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
                        .chain(std::iter::repeat(prev_header.id).take(prev_level))
                        .collect())
                } else {
                    Ok(std::iter::once(genesis)
                        .chain(std::iter::repeat(prev_header.id).take(prev_level))
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
            .map(ergo_merkle_tree::MerkleNode::from)
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
        .map(ergo_merkle_tree::MerkleNode::from)
        .collect::<Vec<ergo_merkle_tree::MerkleNode>>();
    ergo_merkle_tree::MerkleTree::new(&leafs)
}

/// The "compact" format is an encoding of a whole number `N` using an unsigned 32 bit number.
/// This number encodes a base-256 scientific notation representation of `N` (similar to a floating
/// point format):
///  - The most significant 8 bits represent the number of bytes necessary to represent `N` in
///    two's-complement form; denote it by `exponent`.
///  - The lower 23 bits are the mantissa(significand).
///  - Bit number 24 (0x800000) represents the sign of N.
///
/// There are 2 cases to consider:
///  - If `exponent >= 3` then `N` is represented by
///      `(-1^sign) * mantissa * 256^(exponent-3)`
///    E.g. suppose the compact form is given in hex-format by `0x04123456`. Mantissa is `0x123456`
///    and `exponent == 4`. So `N == 0x123456 * 265^1`. Now note that we need exactly 4 bytes to
///    represent `N`; 3 bytes for the mantissa and 1 byte for the rest. In base-256:
///      `N == B(0x12)B(0x34)B(0x56)0`
///    where `B(y)` denotes the base-256 representation of a hex-number `y` (note how each base-256
///    digit is represented by a single-byte).
///  - If `exponent < 3` then `N` is represented by the `exponent`-most-significant-bytes of the
///    mantissa. E.g. suppose the compact form is given in hex-format by `0x01003456`. Noting that
///    each hex-digit is represented by 4-bits, our `exponent == 0x01` which is `1` base-10.  The
///    mantissa is represented by `0x003456` and it's most signficant byte is `0x00`. Therefore
///    `N == 0`.
///
/// Satoshi's original implementation used BN_bn2mpi() and BN_mpi2bn(). MPI uses the most
/// significant bit of the first byte as sign. Thus 0x1234560000 is compact 0x05123456 and
/// 0xc0de000000 is compact 0x0600c0de. Compact 0x05c0de00 would be -0x40de000000.
///
/// Bitcoin only uses this "compact" format for encoding difficulty targets, which are unsigned
/// 256bit quantities.  Thus, all the complexities of the sign bit and using base 256 are probably
/// an implementation accident.
pub fn decode_compact_bits(n_bits: u64) -> BigInt {
    let compact = n_bits as i64;
    let size = ((compact >> 24) as i32) & 0xFF;
    if size == 0 {
        return BigInt::from(0);
    }
    let mut buf: Vec<i8> = std::iter::repeat(0).take(size as usize).collect();
    if size >= 1 {
        // Store the first byte of the mantissa
        buf[0] = (((compact >> 16) as i32) & 0xFF) as i8;
    }
    if size >= 2 {
        buf[1] = (((compact >> 8) as i32) & 0xFF) as i8;
    }
    if size >= 3 {
        buf[2] = ((compact as i32) & 0xFF) as i8;
    }

    let is_negative = (buf[0] as i32) & 0x80 == 0x80;
    if is_negative {
        buf[0] &= 0x7f;
        let buf: Vec<_> = buf.into_iter().map(|x| x as u8).collect();
        -BigInt::from_signed_bytes_be(&buf)
    } else {
        let buf: Vec<_> = buf.into_iter().map(|x| x as u8).collect();
        BigInt::from_signed_bytes_be(&buf)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigInt;

    #[test]
    fn test_decode_n_bits() {
        // Following example taken from https://btcinformation.org/en/developer-reference#target-nbits
        let n_bits = 0x181bc330;
        assert_eq!(
            decode_compact_bits(n_bits),
            BigInt::parse_bytes(b"1bc330000000000000000000000000000000000000000000", 16).unwrap()
        );

        let n_bits = 0x01003456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x00).unwrap()
        );

        let n_bits = 0x01123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x12).unwrap()
        );

        let n_bits = 0x04923456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&-0x12345600).unwrap()
        );

        let n_bits = 0x04123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x12345600).unwrap()
        );

        let n_bits = 0x05123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x1234560000i64).unwrap()
        );

        let n_bits = 16842752;
        assert_eq!(decode_compact_bits(n_bits), BigInt::from(1_u8));
    }
}
