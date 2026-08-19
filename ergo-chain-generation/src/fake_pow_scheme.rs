//! Fake pow scheme from `ergo` node tests. This method generates blocks according to Autolykos v1
//! and the 'd' parameter is generated deterministically such that the level of the block is
//! **always** positive.
//!
//! Tests are all adapted from <https://github.com/ergoplatform/ergo/blob/master/src/test/scala/org/ergoplatform/modifiers/history/PoPowAlgosSpec.scala>

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergo_lib::ergo_chain_types::{blake2b256_hash, ADDigest, BlockId, Digest32};
    use ergo_nipopow::{NipopowAlgos, NipopowProof, NipopowVerifier, INTERLINK_VECTOR_PREFIX};

    use ergo_chain_types::{autolykos_pow_scheme::order_bigint, AutolykosSolution, Header, Votes};
    use ergo_lib::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
    use ergo_lib::ergotree_ir::serialization::sigma_byte_writer::SigmaByteWriter;
    use ergo_nipopow::PoPowHeader;
    use num_bigint::BigUint;
    use rand::{thread_rng, Rng};

    use crate::{default_miner_secret, ErgoFullBlock, ExtensionCandidate};
    use ergo_merkle_tree::{BatchMerkleProof, MerkleNode, MerkleTree};

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

    fn generated_nipopow_proof() -> (BlockId, NipopowProof) {
        let m = 5;
        let k = 5;
        let chain = generate_popowheader_chain(100, None);
        let genesis_id = chain[0].header.id;
        let proof = NipopowAlgos::default().prove(&chain, k, m).unwrap();
        assert_eq!(proof.prefix[0].header.id, genesis_id);
        (genesis_id, proof)
    }

    fn assert_initial_proof_ignored(genesis_id: BlockId, proof: NipopowProof) {
        let mut verifier = NipopowVerifier::new(genesis_id);
        assert!(verifier.process(proof).is_ok());
        assert!(verifier.best_proof().is_none());
        assert!(verifier.best_chain().is_empty());
    }

    fn indexed_block_id(index: usize) -> BlockId {
        let mut bytes = [0u8; 32];
        bytes[..2].copy_from_slice(&u16::try_from(index).unwrap().to_be_bytes());
        BlockId(Digest32::from(bytes))
    }

    fn packed_interlink_field_value(count: u8, block_id: BlockId) -> Vec<u8> {
        let mut value = vec![count];
        let block_id_bytes: Vec<u8> = block_id.0.into();
        value.extend(block_id_bytes);
        value
    }

    fn set_suffix_head_interlinks_with_valid_proof(
        proof: &mut NipopowProof,
        interlinks: Vec<BlockId>,
    ) {
        let extension =
            ExtensionCandidate::new(NipopowAlgos::pack_interlinks(interlinks.clone())).unwrap();
        let interlinks_proof = NipopowAlgos::proof_for_interlink_vector(&extension).unwrap();
        let extension_root = MerkleTree::new(
            extension
                .fields()
                .iter()
                .map(|(key, value)| {
                    std::iter::once(2u8)
                        .chain(key.iter().copied())
                        .chain(value.iter().copied())
                        .collect::<Vec<_>>()
                })
                .map(MerkleNode::from_bytes)
                .collect::<Vec<_>>(),
        )
        .root_hash();
        proof.suffix_head.interlinks = interlinks;
        proof.suffix_head.interlinks_proof = interlinks_proof;
        proof.suffix_head.header.extension_root = extension_root;
    }

    #[test]
    fn test_nipopow_verifier_accepts_valid_initial_proof() {
        let (genesis_id, proof) = generated_nipopow_proof();
        let mut verifier = NipopowVerifier::new(genesis_id);
        assert!(verifier.process(proof.clone()).is_ok());
        assert_eq!(verifier.best_proof(), Some(proof));
    }

    #[test]
    fn test_generated_interlinks_have_canonical_run_structure() {
        for popow_header in generate_popowheader_chain(100, None) {
            assert_eq!(
                popow_header.interlinks.is_empty(),
                popow_header.header.height == 1
            );
            let mut run_ids = Vec::new();
            for interlink in popow_header.interlinks {
                if run_ids.last() != Some(&interlink) {
                    assert!(!run_ids.contains(&interlink));
                    run_ids.push(interlink);
                }
            }
        }
    }

    #[test]
    fn test_nipopow_verifier_ignores_wrong_genesis() {
        let (_, proof) = generated_nipopow_proof();
        let wrong_genesis = BlockId(Digest32::zero());
        assert_ne!(proof.prefix[0].header.id, wrong_genesis);
        assert_initial_proof_ignored(wrong_genesis, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_disconnected_initial_proof() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(!proof.suffix_tail.is_empty());
        proof.suffix_tail[0].parent_id = BlockId(Digest32::zero());
        assert!(!proof.has_valid_connections());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_non_increasing_initial_proof() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(!proof.suffix_tail.is_empty());
        proof.suffix_tail[0].height = proof.suffix_head.header.height;
        assert!(proof.has_valid_connections());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_invalid_interlinks_initial_proof() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.check_interlinks_proof());
        proof.suffix_head.interlinks.push(BlockId(Digest32::zero()));
        assert!(!proof.suffix_head.check_interlinks_proof());
        assert!(proof.has_valid_connections());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_empty_interlinks_with_nonempty_proof() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        let extension =
            ExtensionCandidate::new(NipopowAlgos::pack_interlinks(vec![genesis_id])).unwrap();
        let nonempty_proof = NipopowAlgos::proof_for_interlink_vector(&extension).unwrap();
        assert!(!nonempty_proof.get_indices().is_empty());
        proof.prefix[0].interlinks.clear();
        proof.prefix[0].interlinks_proof = nonempty_proof;
        assert!(proof.has_valid_connections());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_empty_non_genesis_interlinks() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        proof.suffix_head.interlinks.clear();
        proof.suffix_head.interlinks_proof = BatchMerkleProof::new(vec![], vec![]);
        assert!(!proof.suffix_head.check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_nonempty_genesis_interlinks() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert_eq!(proof.prefix[0].header.height, 1);
        let interlinks = vec![genesis_id];
        let extension =
            ExtensionCandidate::new(NipopowAlgos::pack_interlinks(interlinks.clone())).unwrap();
        proof.prefix[0].interlinks = interlinks;
        proof.prefix[0].interlinks_proof =
            NipopowAlgos::proof_for_interlink_vector(&extension).unwrap();
        assert!(!proof.prefix[0].check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_nonempty_genesis_interlinks_with_empty_proof() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert_eq!(proof.prefix[0].header.height, 1);
        assert!(proof.prefix[0].interlinks.is_empty());
        assert!(proof.prefix[0].interlinks_proof.get_indices().is_empty());
        assert!(proof.prefix[0].interlinks_proof.get_proofs().is_empty());
        proof.prefix[0].interlinks = vec![genesis_id];
        assert!(proof.has_valid_connections());
        assert!(!proof.prefix[0].check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_reopened_interlink_run() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        let mut interlinks = proof.suffix_head.interlinks.clone();
        let reopened_id = interlinks[0];
        assert_ne!(interlinks.last(), Some(&reopened_id));
        interlinks.push(reopened_id);
        set_suffix_head_interlinks_with_valid_proof(&mut proof, interlinks);
        assert!(proof.has_valid_connections());
        assert!(proof
            .suffix_head
            .interlinks_proof
            .valid(proof.suffix_head.header.extension_root.as_ref()));
        assert!(!proof.suffix_head.check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_reopened_intermediate_interlink_run() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        let mut interlinks = proof.suffix_head.interlinks.clone();
        let reopened_id = indexed_block_id(1);
        let separating_id = indexed_block_id(2);
        assert!(!interlinks.contains(&reopened_id));
        assert!(!interlinks.contains(&separating_id));
        assert_ne!(reopened_id, separating_id);
        interlinks.extend([reopened_id, separating_id, reopened_id]);
        set_suffix_head_interlinks_with_valid_proof(&mut proof, interlinks);
        assert!(proof.has_valid_connections());
        assert!(proof
            .suffix_head
            .interlinks_proof
            .valid(proof.suffix_head.header.extension_root.as_ref()));
        assert!(!proof.suffix_head.check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_interlink_run_overflow() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        let repeated_id = indexed_block_id(1);
        assert_ne!(genesis_id, repeated_id);
        let interlinks = std::iter::once(genesis_id)
            .chain(std::iter::repeat_n(repeated_id, usize::from(u8::MAX) + 1))
            .collect();
        // This is the exact extension that unchecked release-mode u8
        // arithmetic would produce for the overflowing run: 256 wraps to 0.
        // A proof over these bytes ensures the structural guard, rather than a
        // stale Merkle proof, is what makes the verifier fail closed.
        let extension = ExtensionCandidate::new(vec![
            (
                [INTERLINK_VECTOR_PREFIX, 0],
                packed_interlink_field_value(1, genesis_id),
            ),
            (
                [INTERLINK_VECTOR_PREFIX, 1],
                packed_interlink_field_value(0, repeated_id),
            ),
        ])
        .unwrap();
        proof.suffix_head.interlinks = interlinks;
        proof.suffix_head.interlinks_proof =
            NipopowAlgos::proof_for_interlink_vector(&extension).unwrap();
        assert!(proof.has_valid_connections());
        assert!(!proof.suffix_head.check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_interlink_key_overflow() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        proof.suffix_head.interlinks = (0..(usize::from(u8::MAX) + 2))
            .map(indexed_block_id)
            .collect();
        assert!(proof.has_valid_connections());
        assert!(!proof.suffix_head.check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_rejects_interlink_key_position_overflow() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        let second_run_id = indexed_block_id(1);
        let third_run_id = indexed_block_id(2);
        assert_ne!(genesis_id, second_run_id);
        assert_ne!(genesis_id, third_run_id);
        assert_ne!(second_run_id, third_run_id);
        let mut interlinks = vec![genesis_id; usize::from(u8::MAX)];
        interlinks.extend([second_run_id, third_run_id]);
        let extension = ExtensionCandidate::new(vec![
            (
                [INTERLINK_VECTOR_PREFIX, 0],
                packed_interlink_field_value(u8::MAX, genesis_id),
            ),
            (
                [INTERLINK_VECTOR_PREFIX, 1],
                packed_interlink_field_value(1, second_run_id),
            ),
            (
                [INTERLINK_VECTOR_PREFIX, 2],
                packed_interlink_field_value(1, third_run_id),
            ),
        ])
        .unwrap();
        proof.suffix_head.interlinks = interlinks;
        proof.suffix_head.interlinks_proof =
            NipopowAlgos::proof_for_interlink_vector(&extension).unwrap();
        assert!(proof.has_valid_connections());
        assert!(!proof.suffix_head.check_interlinks_proof());
        assert_initial_proof_ignored(genesis_id, proof);
    }

    #[test]
    fn test_nipopow_verifier_accepts_maximum_interlink_run() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        let repeated_id = indexed_block_id(1);
        assert_ne!(genesis_id, repeated_id);
        let interlinks = std::iter::once(genesis_id)
            .chain(std::iter::repeat_n(repeated_id, usize::from(u8::MAX)))
            .collect();
        set_suffix_head_interlinks_with_valid_proof(&mut proof, interlinks);
        assert!(proof.suffix_head.check_interlinks_proof());
        let mut verifier = NipopowVerifier::new(genesis_id);
        assert!(verifier.process(proof.clone()).is_ok());
        assert_eq!(verifier.best_proof(), Some(proof));
    }

    #[test]
    fn test_nipopow_verifier_accepts_maximum_interlink_key_count() {
        let (genesis_id, mut proof) = generated_nipopow_proof();
        assert!(proof.suffix_head.header.height > 1);
        let interlinks: Vec<_> = std::iter::once(genesis_id)
            .chain((1..=usize::from(u8::MAX)).map(indexed_block_id))
            .collect();
        assert_eq!(interlinks.len(), usize::from(u8::MAX) + 1);
        assert!(!interlinks[1..].contains(&genesis_id));
        set_suffix_head_interlinks_with_valid_proof(&mut proof, interlinks);
        assert!(proof.suffix_head.check_interlinks_proof());
        let mut verifier = NipopowVerifier::new(genesis_id);
        assert!(verifier.process(proof.clone()).is_ok());
        assert_eq!(verifier.best_proof(), Some(proof));
    }

    #[test]
    fn test_nipopow_verifier_accepts_valid_proof_after_invalid_initial_proof() {
        let (genesis_id, valid_proof) = generated_nipopow_proof();
        let mut invalid_proof = valid_proof.clone();
        invalid_proof.suffix_tail[0].parent_id = BlockId(Digest32::zero());
        assert!(!invalid_proof.has_valid_connections());

        let mut verifier = NipopowVerifier::new(genesis_id);
        assert!(verifier.process(invalid_proof).is_ok());
        assert!(verifier.best_proof().is_none());
        assert!(verifier.process(valid_proof.clone()).is_ok());
        assert_eq!(verifier.best_proof(), Some(valid_proof));
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
        assert_eq!(
            disconnected_proof.validate(),
            Err(ergo_nipopow::NipopowValidationError::InvalidProofStructure(
                "connections"
            ))
        );
        assert!(proof.is_better_than(&disconnected_proof).unwrap());
        assert!(!disconnected_proof.is_better_than(&proof).unwrap());
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
}
