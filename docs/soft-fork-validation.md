# Soft-fork validation settings

Parsing a validation failure and accepting it under a soft fork are separate steps.
A sized tree can retain an unknown opcode, type, method, root-type failure or
position-limit failure as `ErgoTree::Unparsed`. Hard structural failures, including
the zero type prefix, missing constants and oversized constant counts, reject the
tree. Nonzero tree versions require the size bit.

`IsSoftForkable` classifies a validation failure; it does not authorize accepting
the script. `SigmaValidationSettings` supplies the chain's validation-rule
overrides. Its default has no overrides, so no validation failure is tolerated.
`Changed` matches exact added codes (type/method pairs for method rules), and
`Replaced` follows the corresponding validation-rule replacement. The old V5
primitive/type/method rules do not authorize their V6 replacements. Unknown
primitive IDs (10 and 11), including compact container tags, use primitive rules
1007/1017 rather than general type rules 1008/1018.

The caller must obtain overrides from the applicable chain state. For a direct
interpreter call, set `Context::validation_settings`. For transaction reduction,
signing and multisig, set them on the Rust `ErgoStateContext` with
`with_validation_settings`; the wallet passes them into the interpreter context.
Existing state-context constructors use the conservative default. Bindings that
do not expose the setter continue to use that default.

Version gating and settings-based acceptance run in `reduce_to_crypto`, shared by
the verifier, transaction reduction and multisig paths. Error spans retain their
underlying sources, so wrapping a failure does not change its classification.
Future-version parsed trees retain the existing verifier/prover distinction:
verification can skip a version beyond the interpreter's support after network
activation, while the prover refuses to generate a proof for such a parsed tree.

The standalone byte loader still preserves legacy unsized V0 input that it cannot
parse, for byte-identical box and transaction IDs. Loading those bytes is not
validation: unsized `Unparsed` trees always fail reduction, verification and
proving, even when the failure matches voted overrides. Only sized `Unparsed`
trees may follow those overrides. The box/transaction wire parser has no such
generic V0 loading fallback.
