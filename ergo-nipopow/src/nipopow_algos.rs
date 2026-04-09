use ergo_chain_types::{
    autolykos_pow_scheme::{
        decode_compact_bits, order_bigint, AutolykosPowScheme, AutolykosPowSchemeError,
    },
    Header,
};
use num_traits::ToPrimitive;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;

use crate::popow_header_reader::PopowHeaderReader;
use crate::{nipopow_proof::PoPowHeader, NipopowProof, NipopowProofError};
use ergo_chain_types::{BlockId, Digest32, ExtensionCandidate};

/// Prefix for Block Interlinks
pub const INTERLINK_VECTOR_PREFIX: u8 = 0x01;

/// Default value for [`NipopowAlgos::use_last_epochs`] — the number of epochs
/// the difficulty adjustment looks back over. Matches the value used by Ergo
/// mainnet AND testnet (`useLastEpochs = 8` in
/// `ergo-core/src/main/resources/application.conf`).
pub const DEFAULT_USE_LAST_EPOCHS: u32 = 8;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NipopowAlgos {
    /// The proof-of-work scheme
    pub pow_scheme: AutolykosPowScheme,
    /// Number of last epochs the difficulty adjustment looks back over.
    ///
    /// This is the Rust analog of the JVM `chainSettings.useLastEpochs`
    /// field. It is consumed by [`NipopowProof::has_valid_connections`] to
    /// compute the maximum allowed gap between adjacent prefix entries —
    /// JVM-built proofs include continuous-mode difficulty-recalculation
    /// headers and naturally-skipped entries from sparse-superlevel walks,
    /// so the verifier needs a tolerant lookback window to accept them.
    ///
    /// Default is [`DEFAULT_USE_LAST_EPOCHS`] (= 8), matching Ergo mainnet
    /// and testnet `application.conf`. A future port of `ChainSettings` can
    /// replace this single field with the full struct without changing the
    /// connection-check semantics.
    pub use_last_epochs: u32,
}

impl Default for NipopowAlgos {
    fn default() -> Self {
        Self {
            pow_scheme: AutolykosPowScheme::default(),
            use_last_epochs: DEFAULT_USE_LAST_EPOCHS,
        }
    }
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

    /// Computes a NiPoPow proof for the chain exposed by `reader`, optionally
    /// rooted at the prefix that contains a specific header (when
    /// `header_id_opt` is `Some`, that header becomes the suffix head).
    ///
    /// This is a direct port of the JVM
    /// `org.ergoplatform.modifiers.history.popow.NipopowProverWithDbAlgs.prove`
    /// method (the db-backed prover used in production for serving NiPoPoW
    /// proofs to peers). Unlike [`NipopowAlgos::prove`], which requires the
    /// caller to materialize the entire chain as `PoPowHeader`s up front,
    /// `prove_with_reader` walks the interlink hierarchy via the
    /// [`PopowHeaderReader`] callback and only materializes the headers it
    /// actually needs — see the trait docs for the asymptotic motivation.
    ///
    /// # Known divergence from the JVM source
    ///
    /// The JVM `prove` accepts a `params.continuous` flag that, when set,
    /// embeds extra epoch-boundary popow headers into the prefix so peers can
    /// validate difficulty for blocks past the suffix without further sync.
    /// This Rust port does NOT implement that mode: sigma-rust's
    /// [`NipopowProof`] currently has no `continuous` field, so adding it
    /// would require coordinated changes to the struct, the serializer, and
    /// the on-wire format — out of scope for this patch. `prove_with_reader`
    /// always produces non-continuous proofs (equivalent to
    /// `params.continuous = false`). Continuous-mode support is tracked as a
    /// follow-up. JVM peers applying non-continuous proofs still succeed:
    /// `applyPopowProof` does not strictly require the flag, the proof
    /// recipient just cannot self-validate difficulty for blocks beyond the
    /// suffix until they sync more headers.
    pub fn prove_with_reader<R: PopowHeaderReader + ?Sized>(
        &self,
        reader: &R,
        header_id_opt: Option<&BlockId>,
        k: u32,
        m: u32,
    ) -> Result<NipopowProof, NipopowProofError> {
        if k == 0 {
            return Err(NipopowProofError::ZeroKParameter);
        }
        if reader.headers_height() < k + m {
            return Err(NipopowProofError::ChainTooShort);
        }

        // Build the suffix: either rooted at an explicit header_id, or the
        // last `k` headers at the chain tip.
        let (suffix_head, suffix_tail): (PoPowHeader, Vec<Header>) = match header_id_opt {
            Some(header_id) => {
                let suffix_head = reader
                    .popow_header_by_id(header_id)
                    .ok_or(NipopowProofError::MissingPopowHeader)?;
                let suffix_tail = reader.best_headers_after(&suffix_head.header, (k - 1) as usize);
                (suffix_head, suffix_tail)
            }
            None => {
                let suffix = reader.last_headers(k as usize);
                let head = suffix
                    .first()
                    .ok_or(NipopowProofError::MissingPopowHeader)?;
                let suffix_head = reader
                    .popow_header_by_id(&head.id)
                    .ok_or(NipopowProofError::MissingPopowHeader)?;
                let suffix_tail: Vec<Header> = suffix.iter().skip(1).cloned().collect();
                (suffix_head, suffix_tail)
            }
        };

        // Mirror the JVM `prefixBuilder` / `storedHeights` accumulators.
        // The genesis popow header is always in the prefix; height 1 is
        // pre-recorded so the dedup loop below skips it.
        const GENESIS_HEIGHT: u32 = 1;
        let mut stored_heights: BTreeSet<u32> = BTreeSet::new();
        let mut prefix_builder: Vec<PoPowHeader> = Vec::new();

        let genesis = reader
            .popow_header_at_height(GENESIS_HEIGHT)
            .ok_or(NipopowProofError::MissingPopowHeader)?;
        prefix_builder.push(genesis);
        stored_heights.insert(GENESIS_HEIGHT);

        // (Continuous mode would inject additional epoch-boundary headers
        // here. See the doc comment above for why this is omitted.)

        let prefix_collected = prove_prefix(reader, GENESIS_HEIGHT, &suffix_head, m)?;
        for ph in prefix_collected {
            if !stored_heights.contains(&ph.header.height) {
                stored_heights.insert(ph.header.height);
                prefix_builder.push(ph);
            }
        }

        prefix_builder.sort_by_key(|p| p.header.height);

        NipopowProof::new(m, k, prefix_builder, suffix_head, suffix_tail)
    }

    /// Packs interlinks into key-value format of the block extension.
    pub fn pack_interlinks(interlinks: Vec<BlockId>) -> Vec<([u8; 2], Vec<u8>)> {
        let mut res = vec![];
        let mut ix_distinct_block_ids = 0;
        let mut curr_block_id_count = 1;
        let mut curr_block_id = interlinks[0];
        for id in interlinks.into_iter().skip(1) {
            if id == curr_block_id {
                curr_block_id_count += 1;
            } else {
                let block_id_bytes: Vec<u8> = curr_block_id.0.into();
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

// Helpers for `NipopowAlgos::prove_with_reader`. Direct ports of the
// nested helpers in JVM `NipopowProverWithDbAlgs.prove`.

/// Port of JVM `linksWithIndexes(header) = header.interlinks.tail.reverse.zipWithIndex`.
///
/// Given `interlinks = [genesis, X_max, ..., X_2, X_1]` (genesis at index 0,
/// highest superlevel pointer at index 1, lowest at the last index), this
/// returns `[(X_1, 0), (X_2, 1), ..., (X_max, max-1)]`. Index 0 maps to the
/// LOWEST superlevel pointer (i.e. `interlinks[len-1]`), and the highest
/// index maps to the HIGHEST superlevel pointer (i.e. `interlinks[1]`).
fn links_with_indexes(header: &PoPowHeader) -> Vec<(BlockId, usize)> {
    if header.interlinks.len() < 2 {
        return Vec::new();
    }
    header
        .interlinks
        .iter()
        .skip(1)
        .rev()
        .copied()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect()
}

/// Port of JVM `previousHeaderIdAtLevel(level, currentHeader)` —
/// `linksWithIndexes(currentHeader).find(_._2 == level).map(_._1)`.
///
/// Looks up the interlink pointer for the given linksWithIndexes index.
/// Returns `None` if `level` is out of range for this header's interlinks.
fn previous_header_id_at_level(level: usize, header: &PoPowHeader) -> Option<BlockId> {
    let n = header.interlinks.len();
    if n < 2 {
        return None;
    }
    // tail length = n - 1, so valid indices are 0..=n-2.
    if level > n - 2 {
        return None;
    }
    Some(header.interlinks[n - 1 - level])
}

/// Port of JVM `collectLevel(prevHeaderId, level, anchoringHeight, acc)`,
/// translated from a `@tailrec` recursion to an explicit loop.
///
/// Walks backward through the interlink at `level` starting at
/// `start_id`, accumulating `PoPowHeader`s until either the walk passes
/// below `anchoring_height` or a header with no interlink at this level is
/// reached. The returned `Vec` is in ascending-height order, matching the
/// JVM `prevHeader +: acc` prepend semantics.
fn collect_level<R: PopowHeaderReader + ?Sized>(
    reader: &R,
    start_id: BlockId,
    level: usize,
    anchoring_height: u32,
) -> Result<Vec<PoPowHeader>, NipopowProofError> {
    // We push to the back during the walk (descending-height order) and
    // reverse once at the end, to avoid the O(n^2) cost of `insert(0, ..)`.
    let mut walked: Vec<PoPowHeader> = Vec::new();
    let mut current_id = start_id;
    loop {
        let prev_header = reader
            .popow_header_by_id(&current_id)
            .ok_or(NipopowProofError::MissingPopowHeader)?;
        if prev_header.header.height < anchoring_height {
            walked.reverse();
            return Ok(walked);
        }
        let next_link = previous_header_id_at_level(level, &prev_header);
        walked.push(prev_header);
        match next_link {
            Some(next_id) => current_id = next_id,
            None => {
                walked.reverse();
                return Ok(walked);
            }
        }
    }
}

/// Port of JVM `provePrefix(initAnchoringHeight, lastHeader)`.
///
/// Iterates over `linksWithIndexes(last_header)` from the highest level
/// down to the lowest (matching Scala `foldRight`), running `collect_level`
/// at each level and updating the running anchoring height as the JVM
/// version does. Returns the deduplicated set of collected headers (sorted
/// by `BlockId` because we use a `BTreeMap`, mirroring Scala
/// `mutable.TreeMap[ModifierId, PoPowHeader]`).
fn prove_prefix<R: PopowHeaderReader + ?Sized>(
    reader: &R,
    init_anchoring_height: u32,
    last_header: &PoPowHeader,
    m: u32,
) -> Result<Vec<PoPowHeader>, NipopowProofError> {
    let mut collected: BTreeMap<BlockId, PoPowHeader> = BTreeMap::new();
    let levels = links_with_indexes(last_header);

    // `levels.foldRight(initAnchoringHeight)` in Scala visits elements from
    // last to first, i.e. highest superlevel first.
    let mut anchoring_height = init_anchoring_height;
    for (prev_header_id, level_idx) in levels.into_iter().rev() {
        let level_headers = collect_level(reader, prev_header_id, level_idx, anchoring_height)?;
        for ph in &level_headers {
            collected.insert(ph.header.id, ph.clone());
        }
        if (m as usize) < level_headers.len() {
            anchoring_height = level_headers[level_headers.len() - (m as usize)]
                .header
                .height;
        }
        // else: anchoring_height unchanged, matching the JVM else branch.
    }

    Ok(collected.into_values().collect())
}
