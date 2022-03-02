//! Chain generation with Autolykos v1 and v2 proof of work. To make things tests tractable the
//! difficulty is set to 1.

use std::convert::TryFrom;

use ergo_lib::{
    chain::{
        ergo_box::box_builder::ErgoBoxCandidateBuilder,
        transaction::{prover_result::ProverResult, Input, Transaction, TxIoVec},
    },
    ergo_chain_types::{BlockId, Digest32},
};
use ergo_lib::{
    ergo_chain_types::blake2b256_hash,
    ergotree_ir::{
        chain::{
            ergo_box::{box_value::BoxValue, BoxId},
            header::{AutolykosSolution, Header},
            votes::Votes,
        },
        ergo_tree::ErgoTree,
        serialization::{sigma_byte_writer::SigmaByteWriter, SigmaSerializable},
        sigma_protocol::dlog_group::{order, EcPoint},
    },
};
use ergo_lib::{
    ergo_chain_types::ADDigest,
    ergotree_interpreter::sigma_protocol::{
        private_input::DlogProverInput,
        prover::{ContextExtension, ProofBytes},
    },
};
use num_bigint::{BigInt, Sign};
use rand::{thread_rng, Rng};

use crate::{
    default_miner_secret, pack_interlinks, unpack_interlinks, update_interlinks, ErgoFullBlock,
    ExtensionCandidate, MerkleTreeNode,
};

/// Section of a block which contains transactions.
#[allow(dead_code)]
struct BlockTransactions {
    /// Identifier of a header of a corresponding block
    header_id: BlockId,
    /// Protocol version for the block
    block_version: u8,
    /// Transactions of the block
    txs: Vec<Transaction>,
}

/// Returns an iterator to generate an arbitrary number of simulated Ergo blocks
pub fn block_stream(start_block: Option<ErgoFullBlock>) -> impl Iterator<Item = ErgoFullBlock> {
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
        // Taken from `dex_t2tpool_parse` unit test in `ergotree-ir`.
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
        .and_then(|b| {
            Some(update_interlinks(
                b.header.clone(),
                unpack_interlinks(&b.extension).ok()?,
            ))
        })
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
    let target_b = order.clone() / ergo_nipopow::decode_compact_bits(header.n_bits);

    let x = DlogProverInput::random();
    let x_bigint = BigInt::from_bytes_be(Sign::Plus, &x.to_bytes());

    use byteorder::{BigEndian, WriteBytesExt};
    let mut height_bytes = Vec::with_capacity(4);
    #[allow(clippy::unwrap_used)]
    height_bytes.write_u32::<BigEndian>(header.height).unwrap();
    let popow_algos = ergo_nipopow::NipopowAlgos::default();
    let big_n = popow_algos.pow_scheme.calc_big_n(version, height);

    // Check nonces
    let min_nonce = i64::MIN;
    let max_nonce = i64::MAX;

    let (sk, sk_bigint) = default_miner_secret();
    let p1 = sk.public_image_bytes().unwrap();
    let p2 = x.public_image().h.sigma_serialize_bytes().unwrap();
    for i in min_nonce..max_nonce {
        let nonce = {
            let mut bytes = vec![];
            bytes.write_i64::<BigEndian>(i).unwrap();
            bytes
        };
        let seed_hash = if version == 1 {
            let mut seed = msg.clone();
            seed.extend(&nonce);
            *blake2b256_hash(&seed).0
        } else {
            *popow_algos
                .pow_scheme
                .calc_seed_v2(big_n, &msg, &nonce, &height_bytes)
                .unwrap()
        };

        let sum = popow_algos
            .pow_scheme
            .gen_indexes(&seed_hash, big_n)
            .into_iter()
            .map(|ix| {
                let mut index_bytes = vec![];
                index_bytes.write_u32::<BigEndian>(ix).unwrap();
                generate_element(version, &msg, &p1, &p2, &index_bytes, &height_bytes)
            })
            .fold(BigInt::from(0_u8), |acc, e| acc + e);
        let d = if version == 1 {
            (x_bigint.clone() * sum - sk_bigint.clone()).modpow(&BigInt::from(1_u8), &order.clone())
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
    let popow_algos = ergo_nipopow::NipopowAlgos::default();
    if version == 1 {
        // Autolykos v. 1: H(j|M|pk|m|w) (line 5 from the Algo 2 of the spec)
        let mut concat = vec![];
        concat.extend(index_bytes);
        concat.extend(popow_algos.pow_scheme.calc_big_m());
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
        concat.extend(popow_algos.pow_scheme.calc_big_m());
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

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use ergo_nipopow::{NipopowAlgos, NipopowProof, PoPowHeader};

    fn generate_popowheader_chain(len: usize, start: Option<PoPowHeader>) -> Vec<PoPowHeader> {
        block_stream(start.map(|p| ErgoFullBlock {
            header: p.header,
            extension: ExtensionCandidate {
                fields: pack_interlinks(p.interlinks),
            },
        }))
        .take(len)
        .flat_map(|b| {
            Some(PoPowHeader {
                header: b.header,
                interlinks: unpack_interlinks(&b.extension).ok()?,
            })
        })
        .collect()
    }

    #[test]
    fn test_nipopow_lowest_common_ancestor_diverging_autolykos_v1() {
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
    fn test_nipopow_best_arg_always_greater_for_better_proofs_autolykos_v1() {
        let m = 30;
        let k = 30;
        let popow_algos = NipopowAlgos::default();

        let chain_0 = generate_popowheader_chain(100, None);
        let proof_0 = popow_algos.prove(&chain_0, k, m).unwrap();
        let chain_1 = chain_0[0..70].to_vec();
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
    fn test_nipopow_is_better_than_marginally_longer_chain_better_autolykos_v1() {
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
    fn test_nipopow_is_better_than_disconnected_chain_should_not_win_autolykos_v1() {
        let m = 50;
        let k = 1;
        let size = 100;
        let popow_algos = NipopowAlgos::default();

        let longer_chain = generate_popowheader_chain(size * 2, None);
        let longer_proof = popow_algos.prove(&longer_chain, k, m).unwrap();

        let chain = longer_chain[0..size].to_vec();
        let proof = popow_algos.prove(&chain, k, m).unwrap();

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
    fn test_popow_roundtrip() {
        use sigma_ser::ScorexSerializable;
        let size = 10;
        let chain = generate_popowheader_chain(size, None);

        for header in chain {
            let bytes = header.scorex_serialize_bytes().unwrap();
            assert_eq!(
                PoPowHeader::scorex_parse(&mut std::io::Cursor::new(bytes)).unwrap(),
                header
            );
        }
    }
}
