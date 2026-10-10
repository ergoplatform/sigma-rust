//! Regression tests for `NipopowAlgos::update_interlinks` with headers whose
//! max level is not positive.
//!
//! These are integration tests (rather than unit tests in
//! `nipopow_algos.rs`) so the regression coverage is independent of the
//! other functions in that file.
//!
//! The headers are Autolykos v1, where `pow_hit` is the `pow_distance`
//! field itself, so the level computed by `max_level_of` is fully
//! determined by `n_bits` and `pow_distance`:
//! `level = log2(order / decode_compact_bits(n_bits)) - log2(pow_distance)`,
//! truncated to `i32`. With `n_bits = 0x207fffff` the required target is
//! exactly 2, so `pow_distance` 1 / 3 / 1_000_000 give levels 1 / 0 / -18.

#![allow(clippy::unwrap_used)]

use ergo_chain_types::{ADDigest, AutolykosSolution, BlockId, Digest32, EcPoint, Header, Votes};
use ergo_nipopow::NipopowAlgos;
use num_bigint::BigUint;

fn block_id(byte: u8) -> BlockId {
    BlockId(Digest32::from([byte; 32]))
}

fn v1_header(height: u32, n_bits: u32, pow_distance: u64) -> Header {
    Header {
        version: 1,
        id: block_id(9),
        parent_id: block_id(8),
        ad_proofs_root: Digest32::zero(),
        state_root: ADDigest::zero(),
        transaction_root: Digest32::zero(),
        timestamp: 0,
        n_bits,
        height,
        extension_root: Digest32::zero(),
        autolykos_solution: AutolykosSolution {
            miner_pk: Box::<EcPoint>::default(),
            pow_onetime_pk: None,
            nonce: vec![],
            pow_distance: Some(BigUint::from(pow_distance)),
        },
        votes: Votes([0, 0, 0]),
        unparsed_bytes: Box::new([]),
    }
}

fn prev_interlinks() -> Vec<BlockId> {
    vec![block_id(1), block_id(2), block_id(3)]
}

#[test]
fn negative_level_returns_prev_interlinks_unchanged() {
    // A header whose PoW hit exceeds its target (an invalid-PoW header)
    // has a negative max level. The reference implementation keeps the
    // level a signed `Int` and returns `prevInterlinks` unchanged when it
    // is not positive. Casting the negative level to `usize` first made
    // it a huge positive number (2^64 - 18 for the level below), and
    // `update_interlinks` then tried to build an interlinks vector with
    // that many copies of the header id, panicking with
    // `capacity overflow` instead of returning a result.
    let header = v1_header(2, 0x207f_ffff, 1_000_000);
    assert_eq!(NipopowAlgos::default().max_level_of(&header).unwrap(), -18);
    let prev = prev_interlinks();
    assert_eq!(
        NipopowAlgos::update_interlinks(header, prev.clone()).unwrap(),
        prev
    );
}

#[test]
fn level_truncating_to_zero_returns_prev_interlinks_unchanged() {
    // -1 < level < 1 truncates to 0, which is not positive either, so the
    // interlinks are returned unchanged (same as the reference's
    // `level.toInt` + `prevLevel > 0` check).
    let header = v1_header(2, 0x207f_ffff, 3);
    assert_eq!(NipopowAlgos::default().max_level_of(&header).unwrap(), 0);
    let prev = prev_interlinks();
    assert_eq!(
        NipopowAlgos::update_interlinks(header, prev.clone()).unwrap(),
        prev
    );
}

#[test]
fn positive_level_one_replaces_last_interlink() {
    // Level 1: genesis + tail with its last entry dropped + one copy of
    // the previous header's id.
    let header = v1_header(2, 0x207f_ffff, 1);
    assert_eq!(NipopowAlgos::default().max_level_of(&header).unwrap(), 1);
    let updated = NipopowAlgos::update_interlinks(header.clone(), prev_interlinks()).unwrap();
    assert_eq!(updated, vec![block_id(1), block_id(2), header.id]);
}

#[test]
fn positive_level_above_interlinks_len_fills_from_genesis() {
    // Level 8 with only 3 interlinks: the whole tail is dropped and the
    // result is the genesis id followed by `level` copies of the previous
    // header's id (the reference's `dropRight` saturates the same way).
    let header = v1_header(2, 0x2000_ffff, 1);
    assert_eq!(NipopowAlgos::default().max_level_of(&header).unwrap(), 8);
    let updated = NipopowAlgos::update_interlinks(header.clone(), prev_interlinks()).unwrap();
    let mut expected = vec![block_id(1)];
    expected.extend(std::iter::repeat_n(header.id, 8));
    assert_eq!(updated, expected);
}

#[test]
fn genesis_header_starts_interlinks_with_own_id() {
    let header = v1_header(1, 0x207f_ffff, 1_000_000);
    assert_eq!(
        NipopowAlgos::update_interlinks(header.clone(), vec![]).unwrap(),
        vec![header.id]
    );
}
