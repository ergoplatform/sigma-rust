use ergo_nipopow::NipopowProofError;

fn legacy_variant_name(error: NipopowProofError) -> &'static str {
    match error {
        NipopowProofError::AutolykosPowSchemeError(_) => "autolykos",
        NipopowProofError::ZeroKParameter => "zero-k",
        NipopowProofError::NonAnchoredChain => "non-anchored",
        NipopowProofError::ChainTooShort => "chain-too-short",
    }
}

#[test]
fn nipopow_proof_error_keeps_its_legacy_exhaustive_variants() {
    assert_eq!(
        legacy_variant_name(NipopowProofError::ZeroKParameter),
        "zero-k"
    );
}
