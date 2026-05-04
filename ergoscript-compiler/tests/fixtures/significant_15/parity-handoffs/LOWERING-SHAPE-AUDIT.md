# Lowering Shape Parity Audit — WS-E.2

**Status:** audit shipped 2026-05-03 as part of WS-E.2. Follow-up
per-shape fixes are individual sessions driven by Metals confirmation
+ concrete fixture traces, mirroring WS-E.1's per-arm methodology.

## Why this audit

Three spot-fixes during the sig-15 wave-1 each closed one categorical
shape divergence:

| Session | Construct | Old (Rust) | New (parity) |
|---|---|---|---|
| S68 | `groupGenerator` | `Expr::GlobalVars(GroupGenerator)` reference | `Expr::PropertyCall(Global, groupGenerator)` |
| S71 | `g.multiply(h)` | `Expr::MethodCall(g, multiply, [h])` | `Expr::MultiplyGroup(g, h)` |
| S76 | `tree.get(key, proof)` | `Expr::TreeLookup` direct opcode | `Expr::MethodCall(GET_METHOD)` (3B header) |

Each was found by fixture-byte diffing followed by Metals lookup against
Scala's `BuildPredefIR` / `ToErgoTree` / `TreeBuilding`. Pattern: 5–10
more such divergences are *expected* across the 35 predefs and ~46 method
arms our compiler dispatches.

## Dispatch surface (Rust side)

### Predef builtins — `lower(Apply{func: Ident(name), args})` dispatch in [`mir/lower.rs::522-1450`](../../../src/mir/lower.rs#L522)

35 named builtins handled by string-match arm. Source-of-truth for the
list comes from the actual match arms (since the WS-A "44 predefs"
figure includes a few that resolve via Constant / GlobalVars in the
binder rather than this dispatch).

```
allOf, anyOf, atLeast, avlTree, blake2b256, byteArrayToBigInt,
byteArrayToLong, Coll, decodeNbits, decodePoint, deserializeTo,
downcast, encodeNbits, executeFromSelfReg, executeFromSelfRegWithDefault,
executeFromVar, fromBigEndianBytes, getVar, getVarFromInput,
longToByteArray, max, min, none, placeholder, powHit, proveDHTuple,
proveDlog, serialize, sha256, sigmaProp, some, substConstants,
treeLookup, upcast, downcast, xor, xorOf, ZKProof
```

### Method/property dispatch — `lower(FieldAccess)` in [`mir/lower.rs::2181-...`](../../../src/mir/lower.rs#L2181)

~46 method/property names: `value`, `propositionBytes`, `id`,
`creationInfo`, `bytes`, `size`, `tokens`, `dataInputs`, `selfBoxIndex`,
`get`, `getOrElse`, `isDefined`, `propBytes`, `toLong`, `toBigInt`,
`toInt`, `toShort`, `toByte`, `getMany`, `insert`, `remove`, `update`,
`updated`, `updateMany`, `updateDigest`, `updateOperations`, `contains`,
`indexOf`, `startsWith`, `endsWith`, `slice`, `append`, `patch`, `zip`,
`map`, `filter`, `flatMap`, `fold`, `forall`, `exists`, `multiply`,
`exp`, `preHeader`, `allZK`. (List from raw `"..." =>` arms; the
dispatch is layered — some names route through a method registry by
SType.)

### Method registry call sites — additional dispatches at [`mir/lower.rs:1535`](../../../src/mir/lower.rs#L1535) and [`mir/lower.rs:1754`](../../../src/mir/lower.rs#L1754)

Two further `match method { ... }` blocks where individual methods are
re-dispatched after type-driven binding.

### Numeric promotion rules — `numeric_upcast_pair`, `narrow_upcast`

Used in `BinOp` lowering for mixed-type arithmetic. Phoenix S17 / S58
already corrected BigInt promotion shape for `Coll[Byte] + BigInt`-style
mixes. Other type pairs (Int+Long, Long+BigInt, signed/unsigned mixes)
have not been audited against Scala's `sigma.ast` type-coercion rules.

## Approach for follow-up sessions

Per-shape spot-fix sessions follow this pattern (template from S68/S71/S76):

1. Pick a fixture with a non-LOCAL-MATCH delta. Diff our hex vs NODE hex
   to identify the diverging opcode region.
2. Trace the diverging region back to the source-language construct.
3. Use Metals MCP to navigate Scala's lowering for that construct
   (`BuildPredefIR.compile` for predefs, `MethodCall` resolution for
   methods, `TreeBuilding.processAstGraph` for shape choices).
4. Compare the Scala-emitted shape against our Rust-emitted shape.
5. Patch our `mir/lower.rs` arm to match.
6. Multi-run-verify the regressing fixture; full regression suite.

**Do not** attempt to "audit-by-analogy" a different predef just because
its shape *looks* similar. Scala's lowerings have one-off special-cases
that don't generalize (e.g., `groupGenerator` is a `PropertyCall(Global, _)`
even though most globals are `GlobalVars(_)`; `multiply` collapses to
`MultiplyGroup` even though most binary methods stay `MethodCall`).

## Suspect clusters (for future per-fixture sessions)

These are heuristic flags from reading the dispatch — each needs Metals
confirmation before any patch.

### Group-element ops

`groupGenerator` (PropertyCall, fixed S68), `multiply` (MultiplyGroup,
fixed S71), `exp` (current shape unverified — likely `Exponentiate`
node, but Metals cross-check needed). Heuristic: the chaincash and
ergomixer scripts both exercise group-element arithmetic; if a future
fixture diff isolates a residual GroupElement byte gap, `exp` is the
prime suspect.

### AVL-tree ops

`avlTree` predef has a documented IR-level discrepancy
([`mir/lower.rs::1326`](../../../src/mir/lower.rs#L1326)): Scala's
`CreateAvlTree::valueLengthOpt` is a runtime Option-typed `Expr`; ours
is a compile-time `Option<Box<Expr>>` resolved from `none[Int]()` /
`some(n)` literals only. This is the AVL backlog driver
(see `project_avl_priority` memory: blocks Lithos / Etcha / Machina
Finance). Fix requires changing
`CreateAvlTree::value_length` shape in `ergotree-ir` first, then
revisiting the predef lowering. Out of scope for spot-fix sessions —
needs an IR-level change.

`treeLookup` predef vs `tree.get` method: S76 fixed `tree.get` to
`MethodCall(GET_METHOD)`. Open question: should the predef-style
`treeLookup(tree, key, proof)` *also* lower to that MethodCall shape, or
does Scala still emit the `TreeLookup` direct opcode (`0xb7`) for the
predef form? Cross-check `BuildPredefIR.compile("treeLookup", ...)` vs
the MethodCall path. If they agree, our `Expr::TreeLookup` may be dead
code post-S76.

### Numeric coercion

`upcast`, `downcast` predefs lower to direct `Upcast`/`Downcast` nodes.
`numeric_upcast_pair` (auto-promotion) handles mixed-type binops.
Phoenix S17/S58 fixed BigInt promotion. Open: signed↔unsigned, BigInt
narrowing, and the placement of explicit `Upcast` *inside* `ByIndex`
arguments (multiple sessions have spot-fixed
`Upcast(ByIndex, SBigInt)` placement; verify the rule is general).

### Sigma-prop conjunctions

`allOf` / `anyOf` lower to `And`/`Or` for `Coll[Boolean]`; `&&`/`||`
binops lower to `BinOp(LogicalAnd/Or)`; `SigmaAnd`/`SigmaOr` constructors
lower from collection-of-SigmaProp. The 4-way mapping has subtle parity
issues (Scala collapses some patterns; our code may not). Heuristic
target if a `gluon_box_guard` / `oracle_refresh` byte-gap diagnosis ever
surfaces a sigma-prop shape divergence.

### Box / Context predefs

`getVar`, `getVarFromInput`, `executeFromVar`, `executeFromSelfReg`,
`executeFromSelfRegWithDefault`, `serialize`, `deserializeTo`. Several
lower to `MethodCall` on Box / Context with hard-coded method ids; if
the method ids drift between Scala and Rust (or the registry rev
changes), these break silently. Worth a Metals cross-check of every
hard-coded method id in this dispatch.

### Crypto / encoding

`encodeNbits`, `decodeNbits`, `powHit`, `decodePoint`,
`fromBigEndianBytes`, `substConstants`. Newer additions in v6.x; less
likely to be exercised by sig-15 fixtures but worth flagging for any
future Lithos / Machina / sigmao byte-gap diagnosis.

## Lowering shapes already verified parity (don't re-check)

- `groupGenerator` → `PropertyCall(Global, groupGenerator)` — S68
- `g.multiply(h)` → `MultiplyGroup(g, h)` — S71
- `tree.get(key, proof)` → `MethodCall(GET_METHOD)` (3B header
  `0xdc 0x64 0x0a`, replacing `TreeLookup` opcode `0xb7`) — S76
- BigInt promotion of `Coll[Byte]` indexing → `Upcast(ByIndex(...), SBigInt)`
  in correct position — Phoenix S17/S58
- `ByteArrayToBigInt` arms in CSE walkers — S66b ergoraffle
- `CreateProveDhTuple` arm in `direct_children` — S68 ergomixer
- `emit_deps` recursion arms for chaincash signature scope — S76
- AvlTree.get method id matches Scala — S76

## What this commit ships

Documentation only:

- `LOWERING-SHAPE-AUDIT.md` (this file): the audit driver for follow-up
  per-fixture / per-construct sessions.

No `mir/lower.rs` arm changes. Following the WS-E.1 commit's
methodology (matrix as deliverable; per-arm changes are follow-ups
driven by concrete failure traces), each shape divergence is its own
focused session — same pattern as S68 / S71 / S76 — not a batch
refactor.

## References

- [WORKSTREAM-E-HANDOFF.md](WORKSTREAM-E-HANDOFF.md) §E.2
- [IR-PASS-COVERAGE-MATRIX.md](IR-PASS-COVERAGE-MATRIX.md) — sibling WS-E.1 deliverable
- [`ergoscript-compiler/src/mir/lower.rs`](../../../src/mir/lower.rs) — predef + method dispatches
- [06c-ergoraffle-inner-block-HANDOFF.md](06c-ergoraffle-inner-block-HANDOFF.md) §"Reference: Scala-side semantics" — Metals MCP usage pattern
