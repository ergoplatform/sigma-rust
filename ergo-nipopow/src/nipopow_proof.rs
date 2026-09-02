use derive_more::From;
use ergo_chain_types::{autolykos_pow_scheme::AutolykosPowSchemeError, BlockId, Header};
use ergo_merkle_tree::BatchMerkleProof;
use serde::{Deserialize, Serialize};
use sigma_ser::{
    vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
    ScorexParsingError, ScorexSerializable, ScorexSerializeResult,
};

use crate::nipopow_algos::NipopowAlgos;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// A structure representing NiPoPow proof as a persistent modifier.
pub struct NipopowProof {
    /// Algos
    #[serde(skip_serializing, skip_deserializing)]
    pub popow_algos: NipopowAlgos,
    /// Security parameter (min μ-level superchain length)
    #[serde(rename = "m")]
    pub m: u32,
    /// Security parameter (min suffix length, >= 1)
    #[serde(rename = "k")]
    pub k: u32,
    /// Proof prefix headers
    #[serde(rename = "prefix")]
    pub prefix: Vec<PoPowHeader>,
    /// First header of the suffix
    #[serde(rename = "suffixHead")]
    pub suffix_head: PoPowHeader,
    /// Tail of the proof suffix headers
    #[serde(rename = "suffixTail")]
    pub suffix_tail: Vec<Header>,
}

impl NipopowProof {
    /// Create new proof instance
    pub fn new(
        m: u32,
        k: u32,
        prefix: Vec<PoPowHeader>,
        suffix_head: PoPowHeader,
        suffix_tail: Vec<Header>,
    ) -> Result<NipopowProof, NipopowProofError> {
        if k >= 1 {
            Ok(NipopowProof {
                popow_algos: NipopowAlgos::default(),
                m,
                k,
                prefix,
                suffix_head,
                suffix_tail,
            })
        } else {
            Err(NipopowProofError::ZeroKParameter)
        }
    }

    /// Implementation of the ≥ algorithm from [`KMZ17`], see Algorithm 4
    ///
    /// [`KMZ17`]: https://fc20.ifca.ai/preproceedings/74.pdf
    pub fn is_better_than(&self, that: &NipopowProof) -> Result<bool, NipopowProofError> {
        if self.is_valid() && that.is_valid() {
            if let Some(lca) = self.popow_algos.lowest_common_ancestor(
                &self.headers_chain().collect::<Vec<_>>(),
                &that.headers_chain().collect::<Vec<_>>(),
            ) {
                let self_headers = self
                    .headers_chain()
                    .filter(|h| h.height > lca.height)
                    .collect::<Vec<_>>();
                let that_headers = that
                    .headers_chain()
                    .filter(|h| h.height > lca.height)
                    .collect::<Vec<_>>();
                Ok(self.popow_algos.best_arg(&self_headers, self.m)?
                    > self.popow_algos.best_arg(&that_headers, self.m)?)
            } else {
                Ok(false)
            }
        } else {
            Ok(self.is_valid())
        }
    }

    /// Returns whether this proof satisfies the current structural validity checks.
    pub(crate) fn is_valid(&self) -> bool {
        self.has_valid_connections() && self.has_valid_heights() && self.has_valid_proofs()
    }

    /// Checks the connections of the blocks in the proof.
    ///
    /// Adjacent blocks in the suffix must be linked via parent id (the
    /// suffix is a strict header chain).
    ///
    /// Prefix entries are linked via interlink or parent block id, but the
    /// check is *tolerant*: each entry need not connect to its immediate
    /// predecessor — it may connect to any of the up to
    /// `use_last_epochs + 3` immediately preceding entries. The tolerance
    /// exists because JVM-built proofs include continuous-mode
    /// difficulty-recalculation headers and naturally-skipped entries from
    /// sparse-superlevel walks; these show up as prefix entries that don't
    /// connect to their immediate sorted-by-height neighbour but do connect
    /// to a nearby earlier neighbour.
    ///
    /// Direct port of the JVM
    /// `org.ergoplatform.modifiers.history.popow.NipopowProof.hasValidConnections`
    /// (see `ergo-core/.../popow/NipopowProof.scala`, lines ~128-148):
    ///
    /// ```scala
    /// val maxDiffHeaders = popowAlgos.chainSettings.useLastEpochs + 1
    /// val prefixToCheck = prefix :+ suffixHead
    /// val prefixConnections = (1 until prefixToCheck.length).forall { checkIdx =>
    ///   val next = prefixToCheck(checkIdx)
    ///   (checkIdx - 1).to(Math.max(0, checkIdx - maxDiffHeaders - 1 - 1), -1).exists { prevIdx =>
    ///     val prev = prefixToCheck(prevIdx)
    ///     next.interlinks.contains(prev.id) || next.header.parentId == prev.id
    ///   }
    /// }
    /// ```
    ///
    /// The Scala `(checkIdx - 1).to(max(0, checkIdx - maxDiffHeaders - 2), -1)`
    /// range is **inclusive on both ends**, so the lookback covers indices
    /// `[max(0, checkIdx - maxDiffHeaders - 2), checkIdx - 1]` — a window of
    /// up to `maxDiffHeaders + 2 = use_last_epochs + 3` predecessors.
    pub fn has_valid_connections(&self) -> bool {
        let use_last_epochs = self.popow_algos.use_last_epochs as usize;
        // `maxDiffHeaders = useLastEpochs + 1` in JVM. The full lookback span
        // is `maxDiffHeaders + 2 = use_last_epochs + 3` predecessors.
        let lookback_span = use_last_epochs + 3;

        let prefix_len = self.prefix.len();
        // `prefixToCheck = prefix :+ suffixHead` — virtual concatenation,
        // accessed via the closure below to avoid an unnecessary clone.
        let prefix_to_check_len = prefix_len + 1;
        let get = |idx: usize| -> &PoPowHeader {
            if idx == prefix_len {
                &self.suffix_head
            } else {
                &self.prefix[idx]
            }
        };

        let prefix_connections = (1..prefix_to_check_len).all(|check_idx| {
            let next = get(check_idx);
            // JVM walks `(checkIdx - 1).to(max(0, checkIdx - maxDiffHeaders - 2), -1)`,
            // i.e. indices `[lookback_start, check_idx - 1]` inclusive,
            // descending. The descending order is observable only as a
            // micro-optimization (closer predecessors are likeliest to match);
            // any iteration order is semantically equivalent for `exists`.
            let lookback_start = check_idx.saturating_sub(lookback_span);
            (lookback_start..check_idx).rev().any(|prev_idx| {
                let prev = get(prev_idx);
                // Note that blocks with level 0 do not appear at all within
                // interlinks, which is why we need to check the parent
                // block id as well.
                next.interlinks.contains(&prev.header.id) || next.header.parent_id == prev.header.id
            })
        });

        let suffix_connections = std::iter::once(&self.suffix_head.header)
            .chain(self.suffix_tail.iter())
            .zip(self.suffix_tail.iter())
            .all(|(prev, next)| next.parent_id == prev.id);

        prefix_connections && suffix_connections
    }

    /// Checks if the heights of the header-chain provided are consistent, meaning that for any two
    /// blocks b1 and b2, if b1 precedes b2 then b1's height should be smaller. Return true if the
    /// heights of the header-chain are consistent
    fn has_valid_heights(&self) -> bool {
        self.headers_chain()
            .zip(self.headers_chain().skip(1))
            .all(|(prev, next)| prev.height < next.height)
    }
    /// Checks interlink proofs for each block using `PoPowHeader::check_interlinks_proof`
    fn has_valid_proofs(&self) -> bool {
        std::iter::once(&self.suffix_head)
            .chain(self.prefix.iter())
            .all(PoPowHeader::check_interlinks_proof)
    }

    /// Returns an iterator representing a chain of `Headers` from `self.prefix`, to
    /// `self.suffix_head` and `self.suffix_tail`.
    pub(crate) fn headers_chain(&self) -> impl Iterator<Item = &Header> {
        self.prefix
            .iter()
            .map(|p| &p.header)
            .chain(std::iter::once(&self.suffix_head.header).chain(self.suffix_tail.iter()))
    }
}

impl ScorexSerializable for NipopowProof {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        w.put_u32(self.m)?;
        w.put_u32(self.k)?;
        w.put_u32(self.prefix.len() as u32)?;
        for p in &self.prefix {
            let prefix_num_bytes = p.scorex_serialize_bytes()?.len();
            w.put_u32(prefix_num_bytes as u32)?;
            p.scorex_serialize(w)?;
        }
        let suffix_head_num_bytes = self.suffix_head.scorex_serialize_bytes()?.len();
        w.put_u32(suffix_head_num_bytes as u32)?;
        self.suffix_head.scorex_serialize(w)?;
        w.put_u32(self.suffix_tail.len() as u32)?;
        for h in &self.suffix_tail {
            let header_num_bytes = h.scorex_serialize_bytes()?.len();
            w.put_u32(header_num_bytes as u32)?;
            h.scorex_serialize(w)?;
        }
        Ok(())
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let m = r.get_u32()?;
        let k = r.get_u32()?;
        let num_prefixes = r.get_u32()? as usize;
        let mut prefix = Vec::with_capacity(num_prefixes);
        for _ in 0..num_prefixes {
            let _size = r.get_u32()?;
            prefix.push(PoPowHeader::scorex_parse(r)?);
        }
        let _suffix_head_size = r.get_u32()?;
        let suffix_head = PoPowHeader::scorex_parse(r)?;
        let num_suffix_tail = r.get_u32()? as usize;
        let mut suffix_tail = Vec::with_capacity(num_suffix_tail);
        for _ in 0..num_suffix_tail {
            let _size = r.get_u32();
            suffix_tail.push(Header::scorex_parse(r)?);
        }
        Ok(NipopowProof {
            popow_algos: NipopowAlgos::default(),
            m,
            k,
            prefix,
            suffix_head,
            suffix_tail,
        })
    }
}

/// `NipopowProof` errors
#[derive(PartialEq, Eq, Debug, Clone, From, thiserror::Error)]
pub enum NipopowProofError {
    /// Errors from `AutolykosPowScheme`
    #[error("{0:?}")]
    AutolykosPowSchemeError(AutolykosPowSchemeError),
    /// `k` parameter == 0. Must be >= 1.
    #[error("k parameter == 0. Must be >= 1")]
    ZeroKParameter,
    /// Can not prove non-anchored (first block is non-Genesis) chain
    #[error("Can not prove non-anchored (first block is non-Genesis) chain")]
    NonAnchoredChain,
    /// Chain must be of length `>= k + m`
    #[error("Chain must be of length `>= k + m`")]
    ChainTooShort,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// PoPowHeader structure. Represents the block header and unpacked interlinks
pub struct PoPowHeader {
    /// The block header
    pub header: Header,
    /// Interlinks are stored in reverse order: first element is always genesis header, then level
    /// of lowest target met etc
    pub interlinks: Vec<BlockId>,
    /// BatchMerkleProof for interlinks in extension field
    #[serde(rename = "interlinksProof")]
    pub interlinks_proof: BatchMerkleProof,
}

impl PoPowHeader {
    fn has_packable_interlinks(&self) -> bool {
        let Some(mut current) = self.interlinks.first() else {
            return false;
        };
        let max_run_length = usize::from(u8::MAX);
        let max_key_position = usize::from(u8::MAX);
        let mut run_length = 1usize;

        for (position, interlink) in self.interlinks.iter().enumerate().skip(1) {
            if interlink == current {
                if run_length == max_run_length {
                    return false;
                }
                run_length += 1;
            } else {
                if position > max_key_position {
                    return false;
                }
                current = interlink;
                run_length = 1;
            }
        }
        true
    }

    /// Validates interlinks merkle root against provided proof
    pub fn check_interlinks_proof(&self) -> bool {
        let proof_is_empty = self.interlinks_proof.get_indices().is_empty()
            && self.interlinks_proof.get_proofs().is_empty();
        if self.interlinks.is_empty() {
            return proof_is_empty;
        }
        if !self.has_packable_interlinks() {
            return false;
        }

        let fields: Vec<ergo_merkle_tree::MerkleNode> =
            NipopowAlgos::pack_interlinks(self.interlinks.clone())
                .into_iter()
                .map(|(k, v)| -> Vec<u8> {
                    std::iter::once(2u8)
                        .chain(k.iter().copied())
                        .chain(v)
                        .collect()
                })
                .map(ergo_merkle_tree::MerkleNode::from_bytes)
                .collect();
        let tree = ergo_merkle_tree::MerkleTree::new(fields);
        self.interlinks_proof.valid(tree.root_hash().as_ref())
    }
}

impl ScorexSerializable for PoPowHeader {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        let bytes = self.header.scorex_serialize_bytes()?;
        w.put_u32(bytes.len() as u32)?;
        w.write_all(&bytes)?;
        w.put_u32(self.interlinks.len() as u32)?;
        for interlink in self.interlinks.iter() {
            w.write_all(&interlink.0 .0)?;
        }
        let proof_bytes = self.interlinks_proof.scorex_serialize_bytes()?;
        w.put_u32(proof_bytes.len() as u32)?;
        w.write_all(&proof_bytes)?;

        Ok(())
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let header_size = r.get_u32()?;
        let mut buf = vec![0; header_size as usize];
        r.read_exact(&mut buf)?;
        let header = Header::scorex_parse(&mut std::io::Cursor::new(buf))?;

        let interlinks_size = r.get_u32()?;

        let interlinks: Result<Vec<BlockId>, ScorexParsingError> = (0..interlinks_size)
            .map(|_| {
                let mut buf = [0; 32];
                r.read_exact(&mut buf)?;
                Ok(BlockId(buf.into()))
            })
            .collect();

        let proof_bytes = r.get_u32()? as usize;
        let mut proof_buf = vec![0u8; proof_bytes];
        r.read_exact(&mut proof_buf)?;
        let interlinks_proof = BatchMerkleProof::scorex_parse_bytes(&proof_buf);

        Ok(Self {
            header,
            interlinks: interlinks?,
            interlinks_proof: interlinks_proof?,
        })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {
    use super::*;
    use ergo_chain_types::Digest32;
    use ergo_chain_types::ExtensionCandidate;
    use proptest::prelude::*;
    use proptest::{arbitrary::Arbitrary, collection::vec};

    impl Arbitrary for PoPowHeader {
        type Parameters = ();
        type Strategy = BoxedStrategy<PoPowHeader>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Box<Header>>(), vec(any::<Digest32>(), 1..10))
                .prop_map(|(header, digests)| PoPowHeader {
                    header: *header,
                    interlinks: digests.iter().cloned().map(BlockId).collect(),
                    interlinks_proof: NipopowAlgos::proof_for_interlink_vector(
                        &ExtensionCandidate::new(NipopowAlgos::pack_interlinks(
                            digests.into_iter().map(BlockId).collect(),
                        ))
                        .unwrap(),
                    )
                    .unwrap(),
                })
                .boxed()
        }
    }

    impl Arbitrary for NipopowProof {
        type Parameters = ();
        type Strategy = BoxedStrategy<NipopowProof>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<u32>(),
                any::<u32>(),
                vec(any::<PoPowHeader>(), 1..10),
                any::<PoPowHeader>(),
                vec(any::<Header>(), 1..10),
            )
                .prop_map(|(m, k, prefix, suffix_head, suffix_tail)| NipopowProof {
                    popow_algos: NipopowAlgos::default(),
                    m,
                    k,
                    prefix,
                    suffix_head,
                    suffix_tail,
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::panic)]
pub mod tests {
    use super::*;
    use ergo_chain_types::Digest32;
    use ergo_merkle_tree::BatchMerkleProof;
    use proptest::prelude::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    use sigma_ser::scorex_serialize_roundtrip;
    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn nipopowproof_roundtrip(v in any::<NipopowProof>()) {
            prop_assert_eq![scorex_serialize_roundtrip(&v), v];
        }


    }

    /// Build a `BlockId` filled with the given byte. Used by the
    /// `has_valid_connections` regression tests below to mint distinct,
    /// human-traceable block ids without paying the cost of real hashing.
    fn id_from_byte(byte: u8) -> BlockId {
        BlockId(Digest32::from([byte; 32]))
    }

    /// Generate one valid base `Header` via the proptest `Arbitrary` impl,
    /// then return a customizing closure. The closure rewrites the
    /// `id`, `parent_id`, and `height` fields without rebuilding the rest
    /// of the (irrelevant-to-this-test) header content. We only need the
    /// three fields above because `has_valid_connections` only inspects
    /// `header.id`, `header.parent_id`, and `PoPowHeader::interlinks`.
    fn header_factory() -> impl Fn(BlockId, BlockId, u32) -> Header {
        let mut runner = TestRunner::default();
        let base = any::<Box<Header>>()
            .new_tree(&mut runner)
            .unwrap()
            .current();
        move |id, parent_id, height| {
            let mut h = (*base).clone();
            h.id = id;
            h.parent_id = parent_id;
            h.height = height;
            h
        }
    }

    /// Build a `PoPowHeader` from a header plus an interlink vector. The
    /// `interlinks_proof` is a no-op empty proof — `has_valid_connections`
    /// never invokes `check_interlinks_proof`, so this is sufficient for
    /// connection-check tests.
    fn pop_header(header: Header, interlinks: Vec<BlockId>) -> PoPowHeader {
        PoPowHeader {
            header,
            interlinks,
            interlinks_proof: BatchMerkleProof::new(vec![], vec![]),
        }
    }

    /// Constructs a deliberately-skipped prefix and asserts the JVM-tolerant
    /// `has_valid_connections` accepts it.
    ///
    /// Layout: `h0 -> h1 -> h2 -> h3 -> suffix_head`. `h2.parent_id` is set
    /// to an unrelated id (NOT `h1.id`) and `h1.id` is **not** in
    /// `h2.interlinks`, so the strict (pre-fix) verifier would have rejected
    /// at index 2. The tolerant verifier MUST accept because `h0.id` is in
    /// `h2.interlinks` and `h0` is within the lookback window
    /// (`use_last_epochs + 3 = 11` predecessors by default).
    #[test]
    fn has_valid_connections_accepts_skipped_prefix_entry() {
        let mk_header = header_factory();

        let h0_id = id_from_byte(1);
        let h1_id = id_from_byte(2);
        let h2_id = id_from_byte(3);
        let h3_id = id_from_byte(4);
        let suffix_head_id = id_from_byte(5);
        let suffix_tail_id = id_from_byte(6);
        let unrelated_parent = id_from_byte(0xff);

        // h0: genesis. parent_id is irrelevant for index 0.
        let h0 = pop_header(mk_header(h0_id, id_from_byte(0), 1), vec![h0_id]);
        // h1: connects via parent_id == h0.id.
        let h1 = pop_header(mk_header(h1_id, h0_id, 10), vec![h0_id]);
        // h2: parent_id is UNRELATED (not h1) and h1.id is NOT in interlinks,
        // but h0.id IS in interlinks → tolerant verifier connects via h0
        // through the lookback window.
        let h2 = pop_header(mk_header(h2_id, unrelated_parent, 20), vec![h0_id]);
        // h3: connects via parent_id == h2.id.
        let h3 = pop_header(mk_header(h3_id, h2_id, 30), vec![h0_id, h2_id]);
        // suffix_head: connects via parent_id == h3.id.
        let suffix_head = pop_header(
            mk_header(suffix_head_id, h3_id, 40),
            vec![h0_id, h2_id, h3_id],
        );
        // suffix_tail must be a strict parent_id chain.
        let suffix_tail = vec![mk_header(suffix_tail_id, suffix_head_id, 41)];

        let proof =
            NipopowProof::new(6, 2, vec![h0, h1, h2, h3], suffix_head, suffix_tail).unwrap();

        assert!(
            proof.has_valid_connections(),
            "tolerant verifier must accept a prefix where an entry skips its \
             immediate predecessor but still connects to a header within the \
             lookback window"
        );
    }

    /// Constructs a prefix with a gap LARGER than the lookback window and
    /// asserts the verifier still rejects. This proves the fix is not a
    /// blanket accept-all.
    ///
    /// We squeeze the lookback by setting `use_last_epochs = 0`, which gives
    /// `lookback_span = 3`. The chain is then designed so that the bad entry
    /// (suffix_head, at index 4) only connects backward to index 0, which is
    /// outside the `[1, 3]` lookback range.
    #[test]
    fn has_valid_connections_rejects_too_far_skip() {
        let mk_header = header_factory();

        let h0_id = id_from_byte(1);
        let h1_id = id_from_byte(2);
        let h2_id = id_from_byte(3);
        let h3_id = id_from_byte(4);
        let suffix_head_id = id_from_byte(5);
        let suffix_tail_id = id_from_byte(6);
        let unrelated_parent = id_from_byte(0xff);

        let h0 = pop_header(mk_header(h0_id, id_from_byte(0), 1), vec![h0_id]);
        let h1 = pop_header(mk_header(h1_id, h0_id, 10), vec![h0_id]);
        let h2 = pop_header(mk_header(h2_id, h1_id, 20), vec![h0_id, h1_id]);
        let h3 = pop_header(mk_header(h3_id, h2_id, 30), vec![h0_id, h2_id]);
        // suffix_head's parent is unrelated, h0 is its only interlink, and
        // h1/h2/h3 ids are NOT among its interlinks. With lookback span 3,
        // the lookback window for index 4 covers indices [1, 3] only —
        // h0 (index 0) is excluded → no valid predecessor → REJECT.
        let suffix_head = pop_header(mk_header(suffix_head_id, unrelated_parent, 40), vec![h0_id]);
        let suffix_tail = vec![mk_header(suffix_tail_id, suffix_head_id, 41)];

        let mut proof =
            NipopowProof::new(6, 2, vec![h0, h1, h2, h3], suffix_head, suffix_tail).unwrap();

        // Squeeze the lookback window to size 3 (= use_last_epochs + 3)
        // so a small synthetic chain can demonstrate the boundary.
        proof.popow_algos.use_last_epochs = 0;

        assert!(
            !proof.has_valid_connections(),
            "verifier must reject when the only valid backward connection is \
             outside the lookback window — the fix is tolerant, not blanket"
        );
    }

    /// Sanity check: a proof whose suffix tail is broken (parent_id chain
    /// violated) must still be rejected. Ensures we didn't accidentally
    /// loosen the suffix-side check while loosening the prefix-side check.
    #[test]
    fn has_valid_connections_rejects_broken_suffix_tail() {
        let mk_header = header_factory();

        let h0_id = id_from_byte(1);
        let suffix_head_id = id_from_byte(2);
        let bad_parent = id_from_byte(0xee);
        let suffix_tail_id = id_from_byte(3);

        let h0 = pop_header(mk_header(h0_id, id_from_byte(0), 1), vec![h0_id]);
        let suffix_head = pop_header(mk_header(suffix_head_id, h0_id, 10), vec![h0_id]);
        // suffix_tail header's parent_id is unrelated to suffix_head.id
        // → suffix-side check must fail.
        let suffix_tail = vec![mk_header(suffix_tail_id, bad_parent, 11)];

        let proof = NipopowProof::new(6, 2, vec![h0], suffix_head, suffix_tail).unwrap();

        assert!(
            !proof.has_valid_connections(),
            "broken suffix tail (parent_id chain violation) must still be \
             rejected after the prefix-tolerance fix"
        );
    }
}
