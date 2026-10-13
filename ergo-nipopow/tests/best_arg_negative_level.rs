//! Regression tests: `NipopowAlgos::best_arg` cast the signed level returned
//! by `max_level_of` to `u32` before comparing it against the current level.
//! A header whose PoW hit is far above its target has a *negative* level, and
//! the cast turned that into a value near 2^32, so the header was counted at
//! every level: with at least `m` such headers `best_arg`'s loop incremented
//! the level forever (growing its accumulator as it went) and never returned.
//! The reference implementation (ergo node `NipopowAlgos.bestArg`) filters
//! with the signed comparison `maxLevelOf(_) >= level`, so a negative-level
//! header counts at no level >= 1 (level 0 always counts the whole chain).
//! These tests hang on the unfixed code.
use ergo_chain_types::*;
use ergo_nipopow::NipopowAlgos;
use num_bigint::BigUint;

/// An Autolykos v1 header whose `pow_hit` is exactly its `pow_distance`.
/// With `n_bits = 0x207fffff` the required target `order / difficulty` is
/// ~2, so the level is `1 - log2(pow_distance)`.
fn v1_header(pow_distance: u32, height: u32) -> Header {
    Header {
        version: 1,
        id: BlockId(Digest32::zero()),
        parent_id: BlockId(Digest32::zero()),
        ad_proofs_root: Digest32::zero(),
        state_root: ADDigest::zero(),
        transaction_root: Digest32::zero(),
        timestamp: 0,
        n_bits: 0x207fffff,
        height,
        extension_root: Digest32::zero(),
        autolykos_solution: AutolykosSolution {
            miner_pk: Box::new(ergo_chain_types::ec_point::generator()),
            pow_onetime_pk: None,
            nonce: vec![0; 8],
            pow_distance: Some(BigUint::from(pow_distance)),
        },
        votes: Votes([0; 3]),
        unparsed_bytes: Box::new([]),
    }
}

#[test]
fn max_level_of_negative_level_header() {
    // 1 - log2(1_000_000) ~= -18.9, truncated to -18.
    let algos = NipopowAlgos::default();
    assert_eq!(algos.max_level_of(&v1_header(1_000_000, 2)), Ok(-18));
}

#[test]
fn best_arg_negative_level_header_counts_at_level_zero_only() {
    // Regression test: this call never returned, because the level -18 cast
    // to u32 cleared every level threshold. Only the level-0 entry (the
    // whole chain) may score: 2^0 * 1 = 1.
    let algos = NipopowAlgos::default();
    let h = v1_header(1_000_000, 2);
    assert_eq!(algos.best_arg(&[&h], 1), Ok(1));
}

#[test]
fn best_arg_many_negative_level_headers_terminates() {
    // With m = 2 and three negative-level headers the unfixed loop also
    // diverged (3 >= 2 at every level). Score is level 0 only: 2^0 * 3 = 3.
    let algos = NipopowAlgos::default();
    let chain: Vec<Header> = (0..3).map(|i| v1_header(1_000_000, 2 + i)).collect();
    let refs: Vec<&Header> = chain.iter().collect();
    assert_eq!(algos.best_arg(&refs, 2), Ok(3));
}

#[test]
fn best_arg_mixed_negative_and_positive_levels() {
    // pow_distance 1 gives level 1: counted at levels 0 and 1.
    // The negative-level header counts at level 0 only.
    // Score: max(2^0 * 2, 2^1 * 1) = 2.
    let algos = NipopowAlgos::default();
    let neg = v1_header(1_000_000, 2);
    let pos = v1_header(1, 3);
    assert_eq!(algos.max_level_of(&pos), Ok(1));
    assert_eq!(algos.best_arg(&[&neg, &pos], 1), Ok(2));
}

#[test]
fn best_arg_positive_level_unchanged() {
    // A level-1 header alone scores max(2^0 * 1, 2^1 * 1) = 2, as before.
    let algos = NipopowAlgos::default();
    let h = v1_header(1, 2);
    assert_eq!(algos.best_arg(&[&h], 1), Ok(2));
}
