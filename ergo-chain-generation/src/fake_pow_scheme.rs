//! Fake pow scheme from `ergo` node tests. This method generates blocks according to Autolykos v1
//! and the 'd' parameter is generated deterministically such that the level of the block is
//! **always** positive.
//!
//! Tests are all adapted from <https://github.com/ergoplatform/ergo/blob/master/src/test/scala/org/ergoplatform/modifiers/history/PoPowAlgosSpec.scala>

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergo_lib::ergo_chain_types::{blake2b256_hash, ADDigest, BlockId, Digest32};
    use ergo_nipopow::{NipopowAlgos, NipopowProof, PopowHeaderReader};

    use ergo_chain_types::{autolykos_pow_scheme::order_bigint, AutolykosSolution, Header, Votes};
    use ergo_lib::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use ergo_lib::ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
    use ergo_nipopow::PoPowHeader;
    use num_bigint::BigUint;
    use rand::{thread_rng, Rng};
    use std::collections::HashMap;

    use crate::{default_miner_secret, ErgoFullBlock, ExtensionCandidate};
    use ergo_merkle_tree::{MerkleNode, MerkleTree};

    fn generate_popowheader_chain(len: usize, start: Option<PoPowHeader>) -> Vec<PoPowHeader> {
        block_stream(start.map(|p| ErgoFullBlock {
            header: p.header,
            extension:
                ExtensionCandidate::new(NipopowAlgos::pack_interlinks(p.interlinks)).unwrap(),
        }))
        .take(len)
        .map(ErgoFullBlock::try_into)
        .flat_map(Result::ok)
        .collect()
    }

    fn block_stream(start_block: Option<ErgoFullBlock>) -> impl Iterator<Item = ErgoFullBlock> {
        let block_version = 1;
        let start = if start_block.is_some() {
            start_block
        } else {
            next_block(None, ExtensionCandidate::default(), block_version)
        };
        std::iter::successors(start, move |b| {
            next_block(
                Some(b.clone()),
                ExtensionCandidate::default(),
                block_version,
            )
        })
    }

    fn next_block(
        prev_block: Option<ErgoFullBlock>,
        mut extension: ExtensionCandidate,
        block_version: u8,
    ) -> Option<ErgoFullBlock> {
        let interlinks = prev_block
            .as_ref()
            .and_then(|b| {
                NipopowAlgos::update_interlinks(
                    b.header.clone(),
                    NipopowAlgos::unpack_interlinks(&b.extension).ok()?,
                )
                .ok()
            })
            .unwrap_or_default();
        if !interlinks.is_empty() {
            // Only non-empty for non-genesis block
            extension
                .fields_mut()
                .extend(NipopowAlgos::pack_interlinks(interlinks));
        }
        prove_block(prev_block.map(|b| b.header), block_version, 0, extension)
    }

    fn prove_block(
        parent_header: Option<Header>,
        version: u8,
        timestamp: u64,
        extension_candidate: ExtensionCandidate,
    ) -> Option<ErgoFullBlock> {
        // Corresponds to initial difficulty of 1, in line with the ergo test suite.
        let n_bits = 16842752_u32;
        let state_root = ADDigest::zero();
        let votes = Votes([0, 0, 0]);

        // Ergo test suite uses randomly generated value for ad_proofs_root.
        let mut rng = thread_rng();
        let how_many: usize = rng.gen_range(0..5000);
        let mut ad_proofs_bytes: Vec<u8> = vec![0; how_many];
        for x in &mut ad_proofs_bytes {
            *x = rng.gen();
        }
        let ad_proofs_root = blake2b256_hash(&ad_proofs_bytes);

        // Use dummy transaction root.
        let transaction_root = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]
        .into();

        // Now prove
        let (parent_id, height) = if let Some(parent_header) = parent_header {
            (parent_header.id, parent_header.height + 1)
        } else {
            (BlockId(Digest32::zero()), 1)
        };

        let extension_root = MerkleTree::new(
            extension_candidate
                .fields()
                .iter()
                .map(|(key, value)| {
                    let mut data = vec![2_u8];
                    data.extend(key);
                    data.extend(value);
                    data
                })
                .map(MerkleNode::from_bytes)
                .collect::<Vec<MerkleNode>>(),
        )
        .root_hash_special();

        let dummy_autolykos_solution = AutolykosSolution {
            miner_pk: Box::<ergo_chain_types::EcPoint>::default(),
            pow_onetime_pk: None,
            nonce: vec![],
            pow_distance: Some(BigUint::from(0_u8)),
        };

        let mut header = Header {
            version,
            id: BlockId(Digest32::zero()),
            parent_id,
            ad_proofs_root,
            state_root,
            transaction_root,
            timestamp,
            n_bits,
            height,
            extension_root,
            autolykos_solution: dummy_autolykos_solution,
            votes,
            unparsed_bytes: Box::new([]),
        };

        let x = DlogProverInput::random();

        let (sk, _) = default_miner_secret();
        let nonce: Vec<u8> = vec![0; 8];
        let d = order_bigint() / (height + 1);
        let autolykos_solution = AutolykosSolution {
            miner_pk: sk.public_key().unwrap().public_key.into(),
            pow_onetime_pk: Some(x.public_image().h),
            nonce,
            pow_distance: Some(d.to_biguint().unwrap()),
        };

        // Compute header ID
        let mut id_bytes = header.serialize_without_pow().unwrap();
        let mut data = Vec::new();
        let mut w = SigmaByteWriter::new(&mut data, None);
        autolykos_solution.serialize_bytes(version, &mut w).unwrap();
        id_bytes.extend(data);
        let id = BlockId(blake2b256_hash(&id_bytes));
        header.id = id;
        header.autolykos_solution = autolykos_solution;

        Some(ErgoFullBlock {
            header,
            extension: extension_candidate,
        })
    }
    #[test]
    fn test_nipopow_level_0_valid() {
        let popow_algos = NipopowAlgos::default();
        for b in block_stream(None).take(10) {
            assert!(popow_algos.max_level_of(&b.header).unwrap() >= 0);
        }
    }

    #[test]
    fn test_nipopow_lowest_common_ancestor_diverging() {
        let popow_algos = NipopowAlgos::default();
        for size in [10, 50, 100] {
            let stream = block_stream(None);
            let chain_0: Vec<_> = stream.take(size).collect();
            let branch_point = chain_0[size / 2].clone();
            let mut chain_1 = chain_0[..(size / 2)].to_vec();
            chain_1.extend(block_stream(Some(branch_point.clone())).take(size / 2));
            let chain_0_headers: Vec<_> = chain_0.iter().map(|b| &b.header).collect();
            let chain_1_headers: Vec<_> = chain_1.iter().map(|b| &b.header).collect();
            assert_eq!(
                popow_algos.lowest_common_ancestor(&chain_0_headers, &chain_1_headers),
                Some(branch_point.header.clone())
            );
        }
    }

    #[test]
    fn test_nipopow_best_arg_always_equal_for_equal_proofs() {
        let m = 30;
        let k = 30;
        let popow_algos = NipopowAlgos::default();

        let chain_0 = generate_popowheader_chain(100, None);
        let proof_0 = popow_algos.prove(&chain_0, k, m).unwrap();
        let chain_1 = generate_popowheader_chain(100, None);
        let proof_1 = popow_algos.prove(&chain_1, k, m).unwrap();

        assert!(proof_0.has_valid_connections());
        assert!(proof_1.has_valid_connections());
        assert_eq!(proof_0.prefix.len(), proof_1.prefix.len());

        let chain_0_headers: Vec<_> = chain_0.iter().map(|p| &p.header).collect();
        let chain_1_headers: Vec<_> = chain_1.iter().map(|p| &p.header).collect();

        assert_eq!(
            popow_algos.best_arg(&chain_0_headers, m).unwrap(),
            popow_algos.best_arg(&chain_1_headers, m).unwrap()
        );
    }

    #[test]
    fn test_nipopow_best_arg_always_greater_for_better_proofs() {
        let m = 30;
        let k = 30;
        let popow_algos = NipopowAlgos::default();

        let chain_0 = generate_popowheader_chain(100, None);
        let proof_0 = popow_algos.prove(&chain_0, k, m).unwrap();
        let chain_1 = generate_popowheader_chain(70, None);
        let proof_1 = popow_algos.prove(&chain_1, k, m).unwrap();

        assert!(proof_0.has_valid_connections());
        assert!(proof_1.has_valid_connections());
        assert!(proof_0.prefix.len() > proof_1.prefix.len());

        let chain_0_headers: Vec<_> = chain_0.iter().map(|p| &p.header).collect();
        let chain_1_headers: Vec<_> = chain_1.iter().map(|p| &p.header).collect();

        assert!(
            popow_algos.best_arg(&chain_0_headers, m).unwrap()
                > popow_algos.best_arg(&chain_1_headers, m).unwrap()
        );
    }

    #[test]
    fn test_nipopow_is_better_than_marginally_longer_chain_better() {
        let m = 30;
        let k = 30;
        let popow_algos = NipopowAlgos::default();

        let short_chain = generate_popowheader_chain(100, None);
        let branch_point = short_chain[short_chain.len() - 1].clone();
        let mut long_chain = short_chain.clone();
        long_chain.extend(std::iter::once(
            generate_popowheader_chain(2, Some(branch_point))[1].clone(),
        ));
        let short_proof = popow_algos.prove(&short_chain, k, m).unwrap();
        let long_proof = popow_algos.prove(&long_chain, k, m).unwrap();
        assert!(!short_proof.is_better_than(&long_proof).unwrap());
    }

    #[test]
    fn test_nipopow_is_better_than_disconnected_chain_should_not_win() {
        let m = 50;
        let k = 1;
        let size = 100;
        let popow_algos = NipopowAlgos::default();
        let chain = generate_popowheader_chain(size, None);
        let proof = popow_algos.prove(&chain, k, m).unwrap();

        let longer_chain = generate_popowheader_chain(size * 2, None);
        let longer_proof = popow_algos.prove(&longer_chain, k, m).unwrap();

        let disconnected_proof_prefix: Vec<_> = proof
            .prefix
            .clone()
            .into_iter()
            .take(proof.prefix.len() / 2)
            .chain(longer_proof.prefix)
            .collect();
        let disconnected_proof = NipopowProof {
            popow_algos,
            m,
            k,
            prefix: disconnected_proof_prefix,
            suffix_head: proof.suffix_head.clone(),
            suffix_tail: proof.suffix_tail.clone(),
        };
        assert!(proof.is_better_than(&disconnected_proof).unwrap());
    }

    #[test]
    fn test_nipopow_has_valid_connections_ensure_connected_prefix_chain() {
        let m = 5;
        let k = 5;
        for size in [100, 200] {
            let popow_algos = NipopowAlgos::default();
            let chain = generate_popowheader_chain(size, None);
            let proof = popow_algos.prove(&chain, k, m).unwrap();
            let random_block = generate_popowheader_chain(1, None);
            let mut disconnected_proof_prefix = proof.prefix.clone();
            disconnected_proof_prefix[proof.prefix.len() / 2] = random_block[0].clone();
            let disconnected_proof = NipopowProof {
                popow_algos,
                m,
                k,
                prefix: disconnected_proof_prefix,
                suffix_head: proof.suffix_head.clone(),
                suffix_tail: proof.suffix_tail.clone(),
            };
            assert!(proof.has_valid_connections());
            assert!(!disconnected_proof.has_valid_connections());
        }
    }

    #[test]
    fn test_nipopow_has_valid_connections_ensure_connected_suffix_chain() {
        let m = 5;
        let k = 5;
        for size in [100, 200] {
            let popow_algos = NipopowAlgos::default();
            let chain = generate_popowheader_chain(size, None);
            let proof = popow_algos.prove(&chain, k, m).unwrap();
            let random_block = generate_popowheader_chain(1, None);
            let mut disconnected_proof_suffix_tail = proof.suffix_tail.clone();
            disconnected_proof_suffix_tail[proof.suffix_tail.len() / 2] =
                random_block[0].header.clone();
            let disconnected_proof = NipopowProof {
                popow_algos,
                m,
                k,
                prefix: proof.prefix.clone(),
                suffix_head: proof.suffix_head.clone(),
                suffix_tail: disconnected_proof_suffix_tail,
            };
            assert!(proof.has_valid_connections());
            assert!(!disconnected_proof.has_valid_connections());
        }
    }

    #[test]
    fn test_nipopow_has_valid_connections_ensure_prefix_last_and_suffix_head_linked() {
        let prefix = generate_popowheader_chain(1, None);
        let suffix = generate_popowheader_chain(1, None);
        let proof = NipopowProof {
            popow_algos: NipopowAlgos::default(),
            m: 0,
            k: 0,
            prefix,
            suffix_head: suffix[0].clone(),
            suffix_tail: vec![suffix[0].header.clone()],
        };
        assert!(!proof.has_valid_connections());
    }

    /// In-memory `PopowHeaderReader` backed by a synthetic chain. Lives in
    /// the test module because `ergo-nipopow` cannot depend on
    /// `ergo-chain-generation` (circular dep).
    struct MockReader {
        by_id: HashMap<BlockId, PoPowHeader>,
        by_height: Vec<PoPowHeader>, // index 0 = height 1 (genesis)
    }

    impl MockReader {
        fn from_chain(chain: &[PoPowHeader]) -> Self {
            let mut by_id = HashMap::with_capacity(chain.len());
            let mut by_height: Vec<PoPowHeader> = Vec::with_capacity(chain.len());
            for ph in chain {
                by_id.insert(ph.header.id, ph.clone());
                by_height.push(ph.clone());
            }
            for (i, ph) in by_height.iter().enumerate() {
                assert_eq!(ph.header.height as usize, i + 1);
            }
            Self { by_id, by_height }
        }
    }

    impl PopowHeaderReader for MockReader {
        fn headers_height(&self) -> u32 {
            self.by_height.len() as u32
        }

        fn popow_header_by_id(&self, id: &BlockId) -> Option<PoPowHeader> {
            self.by_id.get(id).cloned()
        }

        fn popow_header_at_height(&self, height: u32) -> Option<PoPowHeader> {
            if height == 0 {
                return None;
            }
            self.by_height.get((height - 1) as usize).cloned()
        }

        fn last_headers(&self, k: usize) -> Vec<Header> {
            let len = self.by_height.len();
            let start = len.saturating_sub(k);
            self.by_height[start..]
                .iter()
                .map(|ph| ph.header.clone())
                .collect()
        }

        fn best_headers_after(&self, header: &Header, n: usize) -> Vec<Header> {
            // Heights are 1-indexed; the slot immediately after `header` is
            // at vec index `header.height` (because by_height[0] is height 1).
            let start = header.height as usize;
            let end = (start + n).min(self.by_height.len());
            if start >= end {
                return Vec::new();
            }
            self.by_height[start..end]
                .iter()
                .map(|ph| ph.header.clone())
                .collect()
        }
    }

    /// Asserts byte-for-byte equivalence between `prove(&chain)` and
    /// `prove_with_reader(&reader, None, k, m)` on a 100-block synthetic
    /// chain at the JVM P2P defaults `m=6, k=10`. This is the Rust
    /// counterpart to `PoPowAlgosWithDBSpec` "proof(chain) is equivalent to
    /// proof(histReader)" in the JVM ergo node.
    ///
    /// The test must use the fake-pow-scheme chain in this module (where
    /// `d = order / (height + 1)` forces `max_level_of >= 1` for every
    /// non-genesis block). On real-Autolykos chains the in-memory algorithm
    /// adds level-0-only blocks at its level-0 iteration that the db-backed
    /// interlink walk never visits, so byte-for-byte equivalence does not
    /// hold there. The JVM equivalent test (`PoPowAlgosWithDBSpec`)
    /// likewise relies on `DefaultFakePowScheme`, which mints blocks with
    /// `d = q / (height + 10)` for the same reason.
    #[test]
    fn test_nipopow_prove_with_reader_matches_in_memory() {
        use sigma_ser::ScorexSerializable;
        let m = 6;
        let k = 10;
        let popow_algos = NipopowAlgos::default();
        let chain = generate_popowheader_chain(100, None);

        // Sanity: every block must have positive level for the equivalence
        // to hold (see test doc).
        for ph in &chain {
            assert!(popow_algos.max_level_of(&ph.header).unwrap() >= 1);
        }

        let in_memory_proof = popow_algos.prove(&chain, k, m).unwrap();
        let reader = MockReader::from_chain(&chain);
        let db_backed_proof = popow_algos
            .prove_with_reader(&reader, None, k, m)
            .unwrap();

        let in_memory_bytes = in_memory_proof.scorex_serialize_bytes().unwrap();
        let db_backed_bytes = db_backed_proof.scorex_serialize_bytes().unwrap();

        assert_eq!(
            in_memory_bytes, db_backed_bytes,
            "prove and prove_with_reader produced different proofs:\n\
             in-memory prefix len: {}, db-backed prefix len: {}",
            in_memory_proof.prefix.len(),
            db_backed_proof.prefix.len()
        );
    }
}
