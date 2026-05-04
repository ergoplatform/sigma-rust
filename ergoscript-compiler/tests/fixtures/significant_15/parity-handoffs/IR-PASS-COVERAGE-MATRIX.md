# IR-Pass Coverage Matrix — WS-E.1

**Status:** matrix shipped 2026-05-03 as part of WS-E.1. No arm additions
in this commit; per-arm fixes are follow-up work driven by concrete
failure traces.

## Methodology

Coverage of each Rust IR-walker pass against the `Expr` enum in
[`ergotree-ir/src/mir/expr.rs`](../../../../ergotree-ir/src/mir/expr.rs)
(68 variants total). For each variant, the table records whether the
pass has an explicit `Expr::Variant(...) => ...` match arm, and whether
that variant is ever **produced** by our `mir/lower.rs` / `hir/optimize.rs`
construction code.

Two walker classes (per WS-E handoff §"Walker enumeration arm coverage"):

- **Counting walkers** (`direct_children`, `count_occurrences`,
  `count_occurrences_no_inner_if`) — bidirectional risk class. A missing
  arm makes the pass treat the node as a leaf, so cross-scope
  occurrences hidden inside it are uncounted. Adding an arm changes
  counts and can cascade into different extraction decisions →
  fixture regressions. **Speculative addition is forbidden** (chaincash
  2026-05-03 false start: adding 5 arms regressed −62 → −77).
- **Completeness walkers** (`replace_all`, `propagate_inner`) —
  monotonic-direction. A missing arm causes silent failure
  (substitution doesn't recurse, type doesn't propagate) but adding an
  arm cannot introduce new behavior, only prevent the failing case.
  Still needs per-arm concrete failure trace per the handoff.

## Coverage matrix

Legend: `✓` = arm exists, `·` = no arm (catch-all), `L` = leaf, `int` = intentional.
Production column: `lower` = produced by `mir/lower.rs`, `hir` = produced by
`hir/optimize.rs`, `–` = never produced by our pipeline (deserialization-only or
source-language feature we don't compile from).

| Variant | dc | co | conii | ra | pi | Produced? | Notes |
|---|---|---|---|---|---|---|---|
| Append | ✓ | · | · | ✓ | · | lower | replace_all arm added S76 (chaincash) — canonical completeness-walker fix |
| Atleast | · | · | · | · | ✓ | – (lower=0) | dead in our pipeline |
| BinOp | ✓ | ✓ | ✓ | ✓ | ✓ | lower,hir | |
| BitInversion | · | · | · | · | · | – | source `~x` lowered to `bitInversion(x)`? not verified — appears unused |
| BlockValue | ✓ | ✓ | ✓ | ✓ | ✓ | lower,hir | |
| BoolToSigmaProp | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ByIndex | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ByteArrayToBigInt | ✓ | ✓ | ✓ | ✓ | · | lower | propagate_inner gap; trace = unverified |
| ByteArrayToLong | ✓ | · | · | · | · | lower (rare) | counting+completeness gaps; possible trace via SLong-byte lowerings |
| CalcBlake2b256 | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| CalcSha256 | · | · | · | · | · | – | not lowered from source |
| Collection | ✓ | ✓ | ✓ | ✓ | ✓ | lower,hir | only `Collection::Exprs` arm; other variants pass-through |
| Const | L | L | L | L | L | lower,hir | leaf |
| ConstPlaceholder | L | L | L | L | L | lower (CSE) | leaf |
| Context | L | L | L | L | L | lower | leaf |
| CreateAvlTree | · | · | · | · | · | lower | **produced** but no walker arms — see project_avl_priority memo. Counting-walker arm needs Metals. Completeness-walker arm needs trace. |
| CreateProveDhTuple | ✓ | · | · | · | · | lower | counting+completeness gaps; closed direct_children S68; co/conii/ra/pi pending traces |
| CreateProveDlog | ✓ | · | · | · | ✓ | lower | counting+completeness gaps |
| DecodePoint | ✓ | · | · | · | · | lower | counting+completeness gaps |
| DeserializeContext | · | · | · | · | · | – | runtime-only; never in a compiled tree |
| DeserializeRegister | · | · | · | · | · | – | runtime-only |
| Downcast | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| Exists | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| Exponentiate | ✓ | · | · | ✓ | · | lower | counting + propagate gaps |
| ExtractAmount | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ExtractBytes | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ExtractBytesWithNoRef | · | · | · | · | · | – | not lowered |
| ExtractCreationInfo | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ExtractId | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ExtractRegisterAs | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ExtractScriptBytes | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| Filter | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| Fold | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ForAll | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| FuncValue | ✓ | int | int | · | ✓ | lower | co/conii deliberately skip lambda bodies; replace_all skips by catch-all (callers ensure target ValUses don't escape into lambdas — re-verify if a fixture surfaces a lambda-body bug) |
| GetVar | · | · | · | · | · | – | not lowered from source |
| Global | L | L | L | L | L | lower | leaf (added as part of `groupGenerator` Global.PropertyCall, S68) |
| GlobalVars | L | L | L | L | L | lower | leaf |
| If | ✓ | ✓ | special | ✓ | ✓ | lower | conii uses inner-if cutoff (S40 — special semantic, not a gap) |
| LogicalNot | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| LongToByteArray | ✓ | · | · | · | ✓ | lower | counting+completeness gap on co/conii/ra |
| Map | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| MethodCall | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| MultiplyGroup | ✓ | · | · | ✓ | · | lower | counting + propagate gaps |
| Negation | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| OptionGet | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| OptionGetOrElse | ✓ | ✓ | ✓ | ✓ | · | lower | propagate_inner gap |
| OptionIsDefined | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| Or | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| PropertyCall | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| SelectField | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| SigmaAnd | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| SigmaOr | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| SigmaPropBytes | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| SigmaPropIsProven | · | · | · | · | · | – | not lowered |
| SizeOf | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| Slice | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| SubstConstants | · | · | · | · | · | – | not lowered |
| TreeLookup | ✓ | ✓ | ✓ | ✓ | ✓ | lower | (legacy lowering — replaced by MethodCall(GET_METHOD) in S76; may go dead) |
| Tuple | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| Upcast | ✓ | ✓ | ✓ | ✓ | ✓ | lower,hir | |
| ValDef | ✓ | ✓ | ✓ | ✓ | ✓ | lower | |
| ValUse | L | L | L | special | ✓ | lower (CSE) | replace_all matches whole expression; propagate_inner overrides tpe |
| Xor | · | · | · | · | · | – | not lowered (source `^` is BitOp::Xor on numerics, not this collection-byte-Xor) |
| XorOf | · | · | · | · | · | – | not lowered |
| ZkProofBlock | · | · | · | · | · | – | serialization fails NotSupported — never reaches CSE |

(Columns: dc=`direct_children`, co=`count_occurrences`, conii=`count_occurrences_no_inner_if`, ra=`replace_all`, pi=`propagate_inner`.)

## Summary by category

### Variants never produced by our pipeline (catch-all is safe)

`Atleast`, `BitInversion`, `CalcSha256`, `DeserializeContext`,
`DeserializeRegister`, `ExtractBytesWithNoRef`, `GetVar`,
`SigmaPropIsProven`, `SubstConstants`, `Xor`, `XorOf`, `ZkProofBlock`.

12 variants. The catch-all in each pass silently passes them through,
which is correct because they never appear in trees we walk. **No
action required.** If any of these become productive later (e.g. a
new lexer/parser gives access to `xor` collection-byte op), the
walker passes will need explicit arms — but that's a downstream
concern, not an E.1 gap.

### Variants produced, with walker gaps to close (per-fixture follow-up)

| Variant | Counting gaps (need Metals + trace) | Completeness gaps (need trace) |
|---|---|---|
| `ByteArrayToLong` | `co`, `conii` | `ra`, `pi` |
| `ByteArrayToBigInt` | – | `pi` |
| `CreateAvlTree` | `dc`, `co`, `conii` | `ra`, `pi` ← **AVL backlog driver** (Lithos/Etcha/Machina) |
| `CreateProveDhTuple` | `co`, `conii` | `ra`, `pi` |
| `CreateProveDlog` | `co`, `conii` | `ra` |
| `DecodePoint` | `co`, `conii` | `ra`, `pi` |
| `Exponentiate` | `co`, `conii` | `pi` |
| `LongToByteArray` | `co`, `conii` | `ra` |
| `MultiplyGroup` | `co`, `conii` | `pi` |
| `OptionGetOrElse` | – | `pi` |

### Counting walker gaps — DO NOT speculatively add

Mandatory rule from WS-E handoff §Pitfalls: "before adding any arm,
Metals goto-definition through Scala's traversal for that node and
confirm Scala counts/visits it the way the candidate arm would.
Additionally, the missing arm must be tied to a concrete failure trace
from a specific fixture." Audit-by-analogy is forbidden. Each row
above is a *follow-up session candidate*, not an action item for this
commit.

The chaincash 2026-05-03 false start is the warning flare:
speculative additions of `Exponentiate`/`MultiplyGroup`/`DecodePoint`/
`LongToByteArray`/`ByteArrayToLong` to `direct_children` regressed
chaincash by 15B. Some are deliberate omissions because Scala's
`processAstGraph` traversal stops at certain hash-cons boundaries.

### Completeness walker gaps — add when a fixture surfaces

`replace_all` and `propagate_inner` arms are monotonic-direction:
adding cannot regress, only fix silent-failure cases. The chaincash
S76 `Append` arm fix is the canonical pattern — one diagnostic trace
(dangling `ValUse(13, SColl(SByte))` inside `Append` chain), one
arm, narrow fix. Add arms here as fixtures expose them, one per
session unless multiple traces converge.

## Catch-all hardening (separate from this commit)

Each walker ends with a silent catch-all (`_ => vec![]`, `_ => {}`,
`other => other`). This silently drops any new `Expr` variant added
to `ergotree-ir`. Hardening — replacing the catch-all with explicit
"intentionally-omitted" arms for the 12 unproduced variants and an
`unreachable!` for produced-but-uncovered variants — would catch
walker gaps at compile time rather than at fixture-diff time. **Out
of scope for E.1**; tracked as future work alongside the
`compiler.rs:91` silent-fallback hardening already noted in the WS-E
handoff §"Confirmed-pattern wins" follow-up.

## References

- [WORKSTREAM-E-HANDOFF.md](WORKSTREAM-E-HANDOFF.md) — methodology
- [`ergotree-ir/src/mir/expr.rs`](../../../../ergotree-ir/src/mir/expr.rs) L104–L250 — `Expr` enum
- [`ergoscript-compiler/src/mir/cse.rs`](../../../src/mir/cse.rs) — `direct_children`, `count_occurrences`, `count_occurrences_no_inner_if`, `replace_all`
- [`ergoscript-compiler/src/mir/lower.rs`](../../../src/mir/lower.rs) — `propagate_inner`
- Chaincash S76 / WS-E handoff §"Confirmed-pattern wins" — `Append` arm canonical fix
- Chaincash 2026-05-03 false start / WS-E handoff §"Negative results" — speculative-add warning
