use std::convert::{TryFrom, TryInto};

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use ergo_lib::{
    chain::{
        ergo_box::box_builder::ErgoBoxCandidateBuilder,
        transaction::{prover_result::ProverResult, Input, Transaction, TxIoVec},
    },
    wallet::{ext_secret_key::ExtSecretKey, mnemonic::Mnemonic},
};
use ergo_nipopow::{NipopowAlgos, NipopowProof, PoPowHeader};
use ergotree_interpreter::sigma_protocol::{
    private_input::DlogProverInput,
    prover::{ContextExtension, ProofBytes},
};
use ergotree_ir::{
    chain::{
        block_id::BlockId,
        digest32::{blake2b256_hash, ADDigest, Digest32},
        ergo_box::{box_value::BoxValue, BoxId},
        header::{AutolykosSolution, Header},
        votes::Votes,
    },
    ergo_tree::ErgoTree,
    serialization::{sigma_byte_writer::SigmaByteWriter, SigmaSerializable},
    sigma_protocol::dlog_group::{order, EcPoint},
};
use num_bigint::{BigInt, Sign};
use rand::{thread_rng, Rng};

static INTERLINK_VECTOR_PREFIX: u8 = 0x01;

#[derive(Clone, Debug)]
struct ErgoFullBlock {
    header: Header,
    //block_transactions: BlockTransactions,
    extension: ExtensionCandidate,
    //ad_proofs: ProofBytes,
}

/// Extension section of Ergo block. Contains key-value storage.
#[derive(Clone, Debug)]
struct ExtensionCandidate {
    /// Fields as a sequence of key -> value records. A key is 2-bytes long, value is 64 bytes max.
    fields: Vec<([u8; 2], Vec<u8>)>,
}

/// Section of a block which contains transactions.
struct BlockTransactions {
    /// Identifier of a header of a corresponding block
    header_id: BlockId,
    /// Protocol version for the block
    block_version: u8,
    /// Transactions of the block
    txs: Vec<Transaction>,
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
    for size in [10, 100, 200] {
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

    let short_chain = generate_popowheader_chain(1000, None);
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

fn generate_popowheader_chain(len: usize, start: Option<PoPowHeader>) -> Vec<PoPowHeader> {
    block_stream(start.map(|p| ErgoFullBlock {
        header: p.header,
        extension: ExtensionCandidate {
            fields: pack_interlinks(p.interlinks),
        },
    }))
    .take(len)
    .map(|b| PoPowHeader {
        header: b.header,
        interlinks: unpack_interlinks(&b.extension),
    })
    .collect()
}

fn block_stream(start_block: Option<ErgoFullBlock>) -> impl Iterator<Item = ErgoFullBlock> {
    let spending_proof = ProverResult {
        proof: ProofBytes::try_from(String::from("7c")).unwrap(),
        extension: ContextExtension::empty(),
    };
    let inputs = vec![Input {
        box_id: BoxId::zero(),
        spending_proof,
    }];

    // Corresponds to `BoxUtils.minimalErgoAmountSimulated` call in `ChainGenerator.scala` in `ergo`.
    let output_candidates = {
        //let ergo_tree = ErgoTree::try_from(Expr::Const(Constant::from(true))).unwrap();
        let base16_str = "19a3030f0400040204020404040404060406058080a0f6f4acdbe01b058080a0f6f4acdbe01b050004d00f0400040005000500d81ad601b2a5730000d602e4c6a70405d603db63087201d604db6308a7d605b27203730100d606b27204730200d607b27203730300d608b27204730400d609b27203730500d60ab27204730600d60b9973078c720602d60c999973088c720502720bd60d8c720802d60e998c720702720dd60f91720e7309d6108c720a02d6117e721006d6127e720e06d613998c7209027210d6147e720d06d615730ad6167e721306d6177e720c06d6187e720b06d6199c72127218d61a9c72167218d1edededededed93c27201c2a793e4c672010405720292c17201c1a793b27203730b00b27204730c00938c7205018c720601ed938c7207018c720801938c7209018c720a019593720c730d95720f929c9c721172127e7202069c7ef07213069a9c72147e7215067e9c720e720206929c9c721472167e7202069c7ef0720e069a9c72117e7215067e9c721372020695ed720f917213730e907217a19d721972149d721a7211ed9272199c7217721492721a9c72177211";
        let tree_bytes = base16::decode(base16_str.as_bytes()).unwrap();
        let ergo_tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();

        let value = BoxValue::try_from(i64::MAX).unwrap();
        let creation_height = i32::MAX as u32;
        let box_candidate = ErgoBoxCandidateBuilder::new(value, ergo_tree, creation_height)
            .build()
            .unwrap();
        vec![box_candidate]
    };

    let txs = vec![Transaction::new(
        TxIoVec::from_vec(inputs).unwrap(),
        None,
        TxIoVec::from_vec(output_candidates).unwrap(),
    )
    .unwrap()];
    let block_version = 1;
    let start = if start_block.is_some() {
        start_block
    } else {
        next_block(
            None,
            txs.clone(),
            ExtensionCandidate { fields: vec![] },
            block_version,
        )
    };
    std::iter::successors(start, move |b| {
        next_block(
            Some(b.clone()),
            txs.clone(),
            ExtensionCandidate { fields: vec![] },
            block_version,
        )
    })
}

fn next_block(
    prev_block: Option<ErgoFullBlock>,
    txs: Vec<Transaction>,
    mut extension: ExtensionCandidate,
    block_version: u8,
) -> Option<ErgoFullBlock> {
    let interlinks = prev_block
        .as_ref()
        .map(|b| update_interlinks(b.header.clone(), unpack_interlinks(&b.extension)))
        .unwrap_or_default();
    if !interlinks.is_empty() {
        // Only non-empty for non-genesis block
        extension.fields.extend(pack_interlinks(interlinks));
    }
    prove_block(
        prev_block.map(|b| b.header),
        block_version,
        txs,
        0,
        extension,
    )
}

fn prove_block(
    parent_header: Option<Header>,
    version: u8,
    transactions: Vec<Transaction>,
    timestamp: u64,
    extension_candidate: ExtensionCandidate,
) -> Option<ErgoFullBlock> {
    // Corresponds to initial difficulty of 1, in line with the ergo test suite.
    let n_bits = 16842752_u64;
    let state_root = ADDigest::zero();
    let votes = Votes([0, 0, 0]);

    // Ergo test suite uses randomly generated value for ad_proofs_root.
    let mut rng = thread_rng();
    let how_many: usize = rng.gen_range(0..5000);
    let mut ad_proofs_bytes: Vec<u8> = std::iter::repeat(0_u8).take(how_many).collect();
    for x in &mut ad_proofs_bytes {
        *x = rng.gen();
    }
    let ad_proofs_root = blake2b256_hash(&ad_proofs_bytes);
    let transaction_root = transactions_root(&transactions, version);

    // Now prove
    let (parent_id, height) = if let Some(parent_header) = parent_header {
        (parent_header.id.clone(), parent_header.height + 1)
    } else {
        (BlockId(Digest32::zero()), 1)
    };

    let extension_root = MerkleTreeNode::new(
        extension_candidate
            .clone()
            .fields
            .into_iter()
            .map(|(key, value)| {
                let mut data = vec![2_u8];
                data.extend(key);
                data.extend(value);
                data
            })
            .collect(),
    )
    .hash()
    .into();

    let dummy_autolykos_solution = AutolykosSolution {
        miner_pk: Box::new(EcPoint::default()),
        pow_onetime_pk: None,
        nonce: vec![],
        pow_distance: Some(BigInt::from(0_u8)),
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
    };
    let msg = blake2b256_hash(&header.serialize_without_pow().unwrap())
        .0
        .to_vec();
    // Order of the secp256k1 elliptic curve
    let order = order();
    let target_b = order.clone() / ergo_nipopow::test::decode_n_bits(header.n_bits);

    let x = DlogProverInput::random();
    let x_bigint = BigInt::from_bytes_be(Sign::Plus, &x.to_bytes());

    use byteorder::{BigEndian, WriteBytesExt};
    let mut height_bytes = Vec::with_capacity(4);
    #[allow(clippy::unwrap_used)]
    height_bytes.write_u32::<BigEndian>(header.height).unwrap();
    let big_n = ergo_nipopow::test::calc_big_n(version, height);

    // Check nonces
    let min_nonce = i64::MIN;
    let max_nonce = i64::MAX;

    let (sk, sk_bigint) = default_miner_secret();
    let p1 = sk.public_image_bytes().unwrap();
    let p2 = x.public_image().h.sigma_serialize_bytes().unwrap();
    for i in min_nonce..max_nonce {
        let nonce: Vec<u8> = std::iter::repeat(0_u8).take(8).collect();
        //{
        //    let mut bytes = vec![];
        //    bytes.write_i64::<BigEndian>(i).unwrap();
        //    bytes
        //};
        let seed_hash = if version == 1 {
            let mut seed = msg.clone();
            seed.extend(&nonce);
            *blake2b256_hash(&seed).0
        } else {
            *ergo_nipopow::test::calc_seed_v2(&msg, big_n, &nonce, &height_bytes)
        };

        let sum = ergo_nipopow::test::gen_indexes(&seed_hash, big_n)
            .into_iter()
            .map(|ix| {
                let mut index_bytes = vec![];
                index_bytes.write_u32::<BigEndian>(ix).unwrap();
                generate_element(version, &msg, &p1, &p2, &index_bytes, &height_bytes)
            })
            .fold(BigInt::from(0_u8), |acc, e| acc + e);
        let d = if version == 1 {
            order.clone() / (height + 1)
            //(x_bigint.clone() * sum - sk_bigint.clone()).modpow(&BigInt::from(1_u8), &order.clone())
        } else {
            BigInt::from_bytes_be(Sign::Plus, &*blake2b256_hash(&sum.to_signed_bytes_be()).0)
        };

        if d <= target_b {
            let autolykos_solution = AutolykosSolution {
                miner_pk: sk.public_key().unwrap().public_key.into(),
                pow_onetime_pk: Some(x.public_image().h),
                nonce,
                pow_distance: Some(d),
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
            break;
            //let popow_algos = NipopowAlgos::default();
            //if popow_algos.max_level_of(&header).unwrap() > 2 {
            //    println!("REAL HIT");
            //    break;
            //}
        }
    }

    Some(ErgoFullBlock {
        header,
        extension: extension_candidate,
    })
}

/// Generate element of Autolykos equation.
fn generate_element(
    version: u8,
    msg: &[u8],
    pk: &[u8],
    w: &[u8],
    index_bytes: &[u8],
    height_bytes: &[u8],
) -> BigInt {
    if version == 1 {
        // Autolykos v. 1: H(j|M|pk|m|w) (line 5 from the Algo 2 of the spec)
        let mut concat = vec![];
        concat.extend(index_bytes);
        concat.extend(ergo_nipopow::test::calc_big_m());
        concat.extend(pk);
        concat.extend(msg);
        concat.extend(w);
        let valid_range = (BigInt::from(2_u8).pow(256) / order()) * order();
        numeric_hash(&concat, valid_range, order())
    } else {
        // Autolykos v. 2: H(j|h|M) (line 5 from the Algo 2 of the spec)
        let mut concat = vec![];
        concat.extend(index_bytes);
        concat.extend(height_bytes);
        concat.extend(ergo_nipopow::test::calc_big_m());
        BigInt::from_bytes_be(Sign::Plus, &blake2b256_hash(&concat).0[1..])
    }
}

/// One way cryptographic hash function that produces numbers in [0,q) range.
/// It calculates Blake2b256 hash of a provided input and checks whether the result is
/// in range from 0 to a maximum number divisible by q without remainder.
/// If yes, it returns the result mod q, otherwise make one more iteration using hash as an input.
/// This is done to ensure uniform distribution of the resulting numbers.
fn numeric_hash(input: &[u8], valid_range: BigInt, order: BigInt) -> BigInt {
    let mut hashed: Vec<u8> = blake2b256_hash(input).into();
    loop {
        let bi = BigInt::from_bytes_be(Sign::Plus, &hashed);
        if bi < valid_range {
            break bi.modpow(&BigInt::from(1_u8), &order);
        } else {
            hashed = blake2b256_hash(&hashed).into();
        }
    }
}

/// Returns the secret key of the miner secret with its `BigInt` representation. Taken from ergo
/// test suite.
fn default_miner_secret() -> (ExtSecretKey, BigInt) {
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

/// Used in the miner when a BlockTransaction instance is not generated yet (because a header is not known)
fn transactions_root(txs: &[Transaction], block_version: u8) -> Digest32 {
    if block_version == 1 {
        let tree = MerkleTreeNode::new(
            txs.iter()
                .map(|tx| {
                    blake2b256_hash(&tx.bytes_to_sign().unwrap())
                        .0
                        .as_ref()
                        .to_vec()
                })
                .collect(),
        );
        tree.hash().into()
    } else {
        let tree = MerkleTreeNode::new(
            txs.iter()
                .map(|tx| {
                    let mut data = blake2b256_hash(&tx.bytes_to_sign().unwrap())
                        .0
                        .as_ref()
                        .to_vec();
                    //  Id of transaction "witness" (taken from Bitcoin jargon, means commitment to
                    //  signatures of a transaction).  Id is 248-bit long, to distinguish
                    //  transaction ids from witness ids in Merkle tree of transactions, where both
                    //  kinds of ids are written into leafs of the tree.
                    let witness: Vec<u8> = tx
                        .inputs
                        .iter()
                        .flat_map(|input| {
                            let i: Vec<u8> = input.spending_proof.proof.clone().into();
                            i
                        })
                        .collect();
                    data.extend(witness);
                    data
                })
                .collect(),
        );
        tree.hash().into()
    }
}

/// Unpacks interlinks from key-value format of block extension.
fn unpack_interlinks(extension: &ExtensionCandidate) -> Vec<BlockId> {
    let mut res = vec![];
    let entries = extension
        .fields
        .iter()
        .filter(|&(key, _)| key[0] == INTERLINK_VECTOR_PREFIX);
    for (_, bytes) in entries {
        // Each interlink is packed as [qty | blockId], which qty is a single-byte value
        // representing the number of duplicates of `blockId`. Every `BlockId` is 32 bytes which
        // implies that `bytes` is 33 bytes.
        assert_eq!(bytes.len(), 33);
        let qty = bytes[0];
        let block_id_bytes: [u8; 32] = bytes[1..].try_into().unwrap();
        let block_id = BlockId(Digest32::from(block_id_bytes));
        res.extend(std::iter::repeat(block_id).take(qty as usize));
    }
    res
}

/// Packs interlinks into key-value format of the block extension.
fn pack_interlinks(interlinks: Vec<BlockId>) -> Vec<([u8; 2], Vec<u8>)> {
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
    let block_id_bytes: Vec<u8> = curr_block_id.clone().0.into();
    let packed_value = std::iter::once(curr_block_id_count)
        .chain(block_id_bytes)
        .collect();
    res.push((
        [INTERLINK_VECTOR_PREFIX, ix_distinct_block_ids],
        packed_value,
    ));
    res
}

/// Computes interlinks vector for a header next to `prevHeader`.
fn update_interlinks(prev_header: Header, prev_interlinks: Vec<BlockId>) -> Vec<BlockId> {
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

#[derive(Clone, PartialEq, Eq, Debug)]
enum MerkleTreeNode {
    Internal {
        left: Box<MerkleTreeNode>,
        right: Box<MerkleTreeNode>,
    },
    Leaf(Vec<u8>),
    Empty,
    EmptyTree,
}

impl MerkleTreeNode {
    fn new(leaves: Vec<Vec<u8>>) -> Self {
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
    fn hash(&self) -> [u8; 32] {
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
pub(crate) fn prefixed_hash(prefix: u8, data: &[u8]) -> Box<[u8; 32]> {
    let mut hasher = VarBlake2b::new(32).unwrap();
    hasher.update(&[prefix]);
    hasher.update(data);
    let hash = hasher.finalize_boxed();
    hash.try_into().unwrap()
}

pub(crate) fn concatenate_hashes(hash_a: &[u8; 32], hash_b: &[u8; 32]) -> [u8; 64] {
    let mut sum = [0; 64];
    sum[0..32].clone_from_slice(hash_a);
    sum[32..].clone_from_slice(hash_b);
    sum
}

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
