//! Regression tests: `NipopowAlgos::max_level_of` used to panic with
//! "attempt to divide by zero" on a non-genesis header whose `n_bits`
//! decodes to a zero difficulty (`order / decode_compact_bits(n_bits)`).
//! `Header::check_pow` already guards this case and returns
//! `AutolykosPowSchemeError::OutOfBounds`; `max_level_of` now does the same,
//! so a parsed NiPoPoW proof carrying such a header is rejected with an
//! error instead of crashing the verifier.
use ergo_chain_types::autolykos_pow_scheme::AutolykosPowSchemeError;
use ergo_chain_types::*;
use ergo_nipopow::NipopowAlgos;

fn header(n_bits: u32, height: u32) -> Header {
    Header {
        version: 2,
        id: BlockId(Digest32::zero()),
        parent_id: BlockId(Digest32::zero()),
        ad_proofs_root: Digest32::zero(),
        state_root: ADDigest::zero(),
        transaction_root: Digest32::zero(),
        timestamp: 0,
        n_bits,
        height,
        extension_root: Digest32::zero(),
        autolykos_solution: AutolykosSolution {
            miner_pk: Box::new(ergo_chain_types::ec_point::generator()),
            pow_onetime_pk: None,
            nonce: vec![0; 8],
            pow_distance: None,
        },
        votes: Votes([0; 3]),
        unparsed_bytes: Box::new([]),
    }
}

#[test]
fn max_level_of_zero_difficulty_returns_error() {
    let algos = NipopowAlgos::default();
    assert_eq!(
        algos.max_level_of(&header(0, 2)),
        Err(AutolykosPowSchemeError::OutOfBounds)
    );
}

#[test]
fn max_level_of_truncated_zero_difficulty_returns_error() {
    // 0x01003456 decodes to 0 (exponent 1 keeps only the top mantissa byte,
    // which is 0x00); `Header::check_pow` treats it as zero difficulty too.
    let algos = NipopowAlgos::default();
    assert_eq!(
        algos.max_level_of(&header(0x01003456, 2)),
        Err(AutolykosPowSchemeError::OutOfBounds)
    );
}

#[test]
fn max_level_of_signed_zero_difficulty_returns_error() {
    // 0x03800000 has the sign bit set with a zero mantissa: -0 == 0.
    let algos = NipopowAlgos::default();
    assert_eq!(
        algos.max_level_of(&header(0x03800000, 2)),
        Err(AutolykosPowSchemeError::OutOfBounds)
    );
}

#[test]
fn best_arg_zero_difficulty_returns_error() {
    // The proof-comparison path: `is_better_than` -> `best_arg` ->
    // `max_level_of`. A zero-difficulty header in the chain must surface
    // as an error, not a panic.
    let algos = NipopowAlgos::default();
    let h = header(0, 2);
    assert_eq!(
        algos.best_arg(&[&h], 1),
        Err(AutolykosPowSchemeError::OutOfBounds)
    );
}

#[test]
fn max_level_of_genesis_header_unchanged() {
    // Genesis headers short-circuit before the difficulty division, so a
    // zero `n_bits` there still yields the genesis level.
    let algos = NipopowAlgos::default();
    assert_eq!(algos.max_level_of(&header(0, 1)), Ok(i32::MAX));
}

#[test]
fn max_level_of_nonzero_difficulty_unchanged() {
    let algos = NipopowAlgos::default();
    assert_eq!(algos.max_level_of(&header(0x207fffff, 2)), Ok(-254));
}
