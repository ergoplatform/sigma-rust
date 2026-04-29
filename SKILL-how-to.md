# ErgoScript Rust Compiler — Skill Guide

How to add a language feature, add a contract test, or modify the CSE pipeline in the `ergoscript-compiler` crate.

**Current state (S60):** 203 lib tests, 45/46 native byte-match, 14/14 ecosystem batch LOCAL MATCH against Ergo node v6.1.2 at `localhost:9053`. Live status lives in [ERGOSCRIPT-COMPILER-STATUS.md](ERGOSCRIPT-COMPILER-STATUS.md). Per-session deep dives live in `SESSION-NN-HANDOFF.md`.

---

## Pipeline Overview

```
source text
  → Lexer (token_kind.rs) → Parser (Pratt) → AST (ast.rs)
  → HIR (hir.rs)
  → Binder (binder.rs)            — resolve identifiers, ValUse linking
  → Type Infer (type_infer.rs)    — assign SType to every node
  → MIR Lower (mir/lower.rs)
  → propagate_val_types           — fix BigInt-from-Long-annotation cases (S17)
  → apply_cse (mir/cse.rs):
       process_ast_graph          — DAG hash-cons + extraction
       apply_cse_within_branches  — recurse into If branches / &&/|| right arms
       flatten_nested_blocks
       disambiguate_val_ids       — globally-unique ids (must come BEFORE next pass — S60)
       dfs_reassign_val_ids       — root-only, body-walk encounter order
       reorder_valdefs            — sort items[] by id
       sequential_renumber        — final ValIds 0..N
  → ErgoTree (ergotree-ir)
```

Every new feature must be threaded through Lexer → … → MIR Lower at minimum. CSE post-passes only kick in for shared sub-expressions — most features ignore them.

## Two Compilation Modes

```rust
use ergoscript_compiler::compiler::{compile, compile_canonical};
use ergoscript_compiler::script_env::ScriptEnv;

// Pure Rust — no network, no node. 45/46 contracts byte-match Scala.
let tree = compile(source, ScriptEnv::new())?;

// Canonical — verifies against Ergo node, falls back to node bytes if local differs.
// Guarantees identical P2S addresses to the Scala toolchain.
let result = compile_canonical(source, ScriptEnv::new(), "http://localhost:9053", &api_key)?;
// result.matched: Some(true) = local match, Some(false) = used node, None = node unavailable
```

Node version pinned to **Ergo v6.1.2** at `localhost:9053`. Use `treeVersion: 0` unless the source explicitly needs v6+ features.

---

## Adding a Language Feature (Step-by-Step)

### 1. Lexer ([src/lexer/token_kind.rs](ergoscript-compiler/src/lexer/token_kind.rs))

Add `TokenKind` variants with `#[token("...")]` or `#[regex("...")]` (logos crate).

- Multi-char tokens before single-char (`==` before `=`, `>=` before `>`).
- Keywords before the `Ident` regex (logos priority: exact > regex).
- Add a `Display` arm and a `check(">=", TokenKind::GtEq)` unit test.

### 2. Syntax ([src/syntax.rs](ergoscript-compiler/src/syntax.rs))

Add matching `SyntaxKind` variants and extend `From<TokenKind>`. Add composite node kinds (`BoolLiteral`, `FuncCall`, `BlockExpr`, …).

`SyntaxKind` uses `FromPrimitive`/`ToPrimitive` derive — discriminants must be contiguous. Don't skip numbers.

### 3. Parser ([src/parser/grammar/expr.rs](ergoscript-compiler/src/parser/grammar/expr.rs))

Pratt parsing. `lhs()` parses primary expressions; `expr_binding_power()` handles infix in a loop.

Binding power table:
```
||           : (1, 2)
&&           : (3, 4)
==, !=       : (5, 6)
>, <, >=, <= : (7, 8)
+, -         : (9, 10)
*, /         : (11, 12)
unary -, !   : ((), 13)
postfix call : 15
dot          : 17
```

- **Infix op:** add a branch in the loop, define binding power.
- **Prefix op:** add a branch in `lhs()`.
- **Postfix:** check at top of loop before infix, use `lhs.precede(p)`.
- **Tests:** `expect_test`. Run `UPDATE_EXPECT=1 cargo test` to regenerate snapshots.

### 4. AST ([src/ast.rs](ergoscript-compiler/src/ast.rs))

New `Expr` variant + cast in `Expr::cast()`. Wrap `SyntaxNode` and add typed accessors (`.children()`, `.children_with_tokens()`, `.first_token()`).

### 5. HIR ([src/hir.rs](ergoscript-compiler/src/hir.rs) + [src/hir/rewrite.rs](ergoscript-compiler/src/hir/rewrite.rs))

Add to `ExprKind`. Implement lowering in `Expr::lower()`. Set `tpe` where known.

**Critical:** also add a match arm in `hir/rewrite.rs` so the binder and type inference recurse into the new node. Forgetting this causes silent failures where child nodes never get resolved.

### 6. Binder ([src/binder.rs](ergoscript-compiler/src/binder.rs))

- New global (`SELF`, `INPUTS`, `HEIGHT`, …): add to the global resolver.
- New built-in function: leave as `FuncCall` and handle in MIR lowering.

### 7. Type Inference ([src/type_infer.rs](ergoscript-compiler/src/type_infer.rs))

- Arith (`+`, `-`, `*`, `/`): result = operand type
- Comparison: result = `SBoolean`
- Logical: result = `SBoolean`
- Function calls: look up return type by name (`"sigmaProp"` → `SSigmaProp`).

### 8. MIR Lowering ([src/mir/lower.rs](ergoscript-compiler/src/mir/lower.rs))

Map HIR → `ergotree_ir::mir::*`. Final step.

| HIR | MIR |
|---|---|
| `BinaryOp` | `BinOpKind` (arith → `ArithOp`, cmp → `RelationOp`, logical → `LogicalOp`) |
| `Literal::Bool` | `Constant::from(bool)` → `Expr::Const` |
| `FuncCall("sigmaProp")` | `BoolToSigmaProp { input }` |
| `GlobalVars::Height` | `GlobalVars::Height` |

After lowering, the function verifies `mir.tpe() == hir_tpe`. Mismatch → fix type inference.

Available `ergotree_ir` types live in `~/.cargo/registry/src/*/ergotree-ir-0.28.0/src/mir/`: `bin_op.rs`, `bool_to_sigma.rs`, `global_vars.rs`, `constant.rs`, `func_value.rs`, `method_call.rs`, `val_def.rs`, `val_use.rs`, …

### Catch-all audit (mandatory whenever you add a new `Expr` variant)

Both [`mir/cse.rs`](ergoscript-compiler/src/mir/cse.rs) functions `collect_and_assign_ids` and `rewrite_ids` use explicit per-variant arms. **Any new variant must be added to BOTH** or lambda params inside it will get wrong IDs (silently). Currently handled: BlockValue, ValDef, FuncValue, BinOp, BoolToSigmaProp, If, Filter, Exists, ForAll, Map, Fold, Append, PropertyCall, MethodCall, Extract*, SizeOf, ByIndex, SelectField, OptionGet, OptionIsDefined, OptionGetOrElse, Slice, LogicalNot, Negation, SigmaPropBytes, Upcast, Downcast, CalcBlake2b256, CreateProveDlog, SigmaAnd, SigmaOr, Tuple, TreeLookup, Apply, And, Or, Collection, Atleast.

---

## Adding a Contract to the Test Corpus

Three tiers, promote in order. Don't skip levels.

### Tier A — Single-contract canonical test

Smallest unit. One `#[test]` calling `compile_canonical(...)` against the live node. Pattern: see [`test_ecosystem_phoenix_hodlerg_bank`](ergoscript-compiler/src/compiler.rs#L3469), [`test_ecosystem_off_the_grid`](ergoscript-compiler/src/compiler.rs#L3526), [`test_ecosystem_crystal_pool_buy`](ergoscript-compiler/src/compiler.rs#L3550).

Run: `source ~/.secrets && cargo test -p ergoscript-compiler test_ecosystem_<name> -- --ignored --nocapture`. Reports `matched: Some(true|false)`.

### Tier B — Ecosystem batch

Append the contract to the inline `contracts: Vec<(&str, &str)>` in [`test_ecosystem_batch`](ergoscript-compiler/src/compiler.rs#L3645). The batch driver runs `compile_canonical` for each entry and prints `LOCAL MATCH` / `USED NODE`. **Goal: every entry reaches LOCAL MATCH.**

If LOCAL MATCH fails, root-cause via `DUMP_TREES=1` / `CONST_DUMP=1` / `CSE_DEBUG=1` (see *CSE diagnostics* below).

### Tier C — Offline native batch

Once Tier B is stable LOCAL MATCH and bytes don't drift across runs, freeze the expected hex into `test_batch_node_byte_match`. CI no longer needs a node for this contract.

### Promotion gate per contract

Tier A green → add to Tier B → reach LOCAL MATCH → freeze into Tier C. Run the **full** suite after every CSE-touching change (`cargo test -p ergoscript-compiler --lib && --lib -- --ignored && test_ecosystem_batch -- --ignored`). CSE tweaks silently break other contracts.

### Fixtures layout

Contracts beyond the original 14 inline strings live in [`ergoscript-compiler/tests/fixtures/significant_15/`](ergoscript-compiler/tests/fixtures/significant_15/). One `.es` per contract + a manifest entry pinning source repo, commit hash, and `ScriptEnv` substitutions. The Tier-B driver loads them from disk.

For sources that use Scala-side string interpolation (`$placeholderConst`):
- **Canonical mode:** leave placeholders as `ScriptEnv` entries, pass an identical `ScriptEnv` to node + local.
- **Tier-C offline mode:** hand-substitute placeholders to fixed dummy values (`fromBase16("...")`, fixed sigma props) and store both source + populated env.

---

## CSE — What You Must Know Before Touching It

CSE is the single highest-risk file in the compiler. Distilled from S43–S60 root causes:

### Triple gate (S55)

Every extraction candidate passes three predicates: `is_collectible` (worth hash-consing), `is_extractable` (legal at this scope), `needs_check` (needs scope safety check). Failing any one drops the candidate. When a contract under-extracts, walk the trio in that order.

### Counting semantics

- `count_dag_usages_scope` counts **parent sets**, not raw occurrences (S55). A node referenced by N parents counts as N regardless of how many times each parent uses it.
- `count_occurrences` (used in §2b S59 seeding) counts raw structural occurrences in the tree.
- Conflate them and you'll over- or under-extract.

### Scope boundaries

- `&&` and `||` lower to `Or.applyLazy(l, Thunk(eval(r)))` — the **right arm is a ThunkDef** (verified in [GraphBuilding.scala:869](https://github.com/ScorexFoundation/sigmastate-interpreter)).
- `If` true/false branches are also Thunk-scoped.
- `direct_children_scope` and `collect_subexprs_scope` stop at these boundaries; `direct_children` (no `_scope`) does not.
- `branch_local_ids` tracks ValDefs **inside If branches only — NOT inside `&&`/`||` right arms** (S59 pitfall).

### Pre-CSE pipeline order (S60 §3a)

```rust
let flattened    = flatten_nested_blocks(deduped);
let disambiguated = disambiguate_val_ids(flattened);   // FIRST — globally unique ids
let reassigned   = dfs_reassign_val_ids(disambiguated); // body-walk encounter order
let reordered    = reorder_valdefs(reassigned);
sequential_renumber(reordered)
```

Disambiguate **before** `dfs_reassign_val_ids` so outer-scope and inner-scope ValDef ids can't collide. Pair with the outer-scope filter in `dfs_collect_val_order` (S60 §3b): `if val_rhs.contains_key(&id)`. The two changes are paired — without disambig-first, the filter misclassifies inner ValUses whose ids alias outer ones.

### Cross-condition-branch SelectField seeding (S59 §2b)

`process_ast_graph_branch` includes a `collect_cond_branch_shared` pass that seeds sub-exprs spanning `If.condition ∩ (true ∪ false)` into `dag_usages` + `schedule`. Targeted: only spans condition+branch. Generalizing to "all sub-exprs inside any thunk" would over-extract.

### Load-bearing micro-rules (don't remove without a regression test for the named contract)

| Rule | Required for | Don't remove unless |
|---|---|---|
| S47 const-RHS partition in `emit_deps`' If-branch handler | SigUSDV1 | You have a SigUSDV1 byte-match test green |
| S55 `is_bare_const` Root-mode scope check | Phoenix HodlERG, others | Full eco battery green |
| `body_walk_children` skips inner BlockValue items[] | id_map coverage invariant (S52) | You re-derive an alternative coverage proof |
| `dfs_reassign_val_ids` is root-only | Avoiding inner-scope blow-ups | Inner-corpus need is empirically proven |

### The DuckPools InterestRate skip (S55+)

`(f * x) / D * x / M * x / M * x / M * x / M` causes recursive CSE stack overflow. Skipped (#39). Fixing requires iterative CSE or a recursion depth limit. Not critical for the 15-significant-contracts list.

### CSE diagnostics

```bash
# All tree dumps end up under /tmp/tree_<NAME>_{local,node}.txt
source ~/.secrets && DUMP_TREES=1 CONST_DUMP=1 ECO_FILTER="<name>" \
    cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture

# Trace which candidates were considered/extracted
CSE_TRACE_EXTRACT=1 ECO_FILTER="<name>" \
    cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture

# Dump branch-level dag_usages_scope and schedule
CSE_DEBUG=1 ECO_FILTER="<name>" \
    cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture
```

Env vars: `ECO_FILTER`, `CONST_DUMP`, `DUMP_TREES`, `PRINT_TREES`, `HEX_TRUNC=N`, `CSE_TRACE_EXTRACT`, `CSE_DEBUG`.

---

## Metals MCP — When and How

Use Metals MCP to read `sigmastate-interpreter` Scala source. **Hard rule: do NOT grep `~/working-files/sigmastate-interpreter/` directly from Bash.** Use MCP tools instead.

### When to reach for it

- A CSE bug needs Scala-side ground truth (`processAstGraph`, `flatSchedule`, `ThunkDef`, `buildTree`).
- A new opcode lowering must match Scala IR shape exactly.
- Byte-diff root cause requires reading Scala extraction order rather than guessing.

### When to skip it

S60 closed without MCP — the diagnosis fell out of side-by-side tree dumps + reading `dfs_collect_val_order` and `disambiguate_val_ids`. Try local diagnostics first; reach for MCP when the local reading runs out of leads.

### Tool cheat sheet

- `mcp__metals__glob-search` — symbol lookup. Disambiguates fast (e.g. `"BinOp"` → 14 candidates).
- `mcp__metals__get-source` with `detailed: true` — full source. **Without `detailed`, method bodies render as `???`**.
- `mcp__metals__inspect` — list class/object members.
- `mcp__metals__list-modules` — modules use `scJVM`, not `sc`, for `sbt testOnly`.

### Canonical entry points in sigmastate-interpreter

| Question | Start at |
|---|---|
| How is `&&`/`||` lowered? | `GraphBuilding.BinAnd`/`BinOr` (`Or.applyLazy(lV, Thunk(eval(r)))`) |
| When does Scala extract? | `IRContext.processAstGraph`, `Costing.buildTree` |
| ThunkDef scope walk | `Thunks.ThunkScope.findDef` parent walk |
| Schedule semantics | `AstGraphs.flatSchedule`, `subG.schedule`, `hasManyUsagesGlobal` |
| Pre-v3 round-trip transforms | `ValueSerializer.serializable`, `TransformingSigmaBuilder.applyUpcast` |

### Schedule dump test (S57)

For ValDef-shape questions on a specific contract:

```bash
cd ~/working-files/sigmastate-interpreter
sbt -Dsbt.io.implicit.relative.glob.conversion=allow \
    "scJVM/testOnly sigmastate.lang.OpenOrderScheduleDumpTest" \
    > /tmp/dump-trees.out 2>&1
grep -A 30 "ValDef RHS shapes" /tmp/dump-trees.out
```

Walker only descends into `ValDef`, `BlockValue`, `FuncValue`, `If`. Expand `collectValDefs` if deeper traversal is needed.

---

## Testing Strategy

### Unit tests at each pipeline stage
Each pipeline file has its own `#[cfg(test)] mod tests` with a `check()` helper that runs the pipeline up to that stage.

### End-to-end test
```rust
#[test]
fn test_my_feature() {
    let result = compile_expr("{ my_source }", ScriptEnv::new());
    assert!(result.is_ok());
}
```

### Byte-for-byte verification
```rust
use ergotree_ir::serialization::SigmaSerializable;
let tree = compile("{ source }", ScriptEnv::new()).unwrap();
let bytes = tree.sigma_serialize_bytes().unwrap();
let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
assert_eq!(hex, "expected_hex_from_node");
```

Get expected hex from the live node:
```bash
source ~/.secrets
ADDR=$(curl -s -X POST "http://localhost:9053/script/p2sAddress" \
  -H "Content-Type: application/json" -H "api_key: $API_KEY" \
  -d '{"source": "{ your_source }", "treeVersion": 0}' | jq -r .address)
curl -s "http://localhost:9053/script/addressToTree/$ADDR" -H "api_key: $API_KEY" | jq -r .tree
```

### Run-everything baseline (after every CSE-touching change)

```bash
cargo test -p ergoscript-compiler --lib                                # 203/203
cargo test -p ergoscript-compiler --lib -- --ignored                   # 3/3
cargo test -p ergoscript-compiler test_batch_node_byte_match           # 1/1
source ~/.secrets && cargo test -p ergoscript-compiler \
    test_ecosystem_batch -- --ignored --nocapture                      # 14/14 LOCAL MATCH
```

---

## Build Notes

- `core2 → core3`: `core2` v0.4.0 was yanked. Use `core3` (drop-in successor) across the workspace.
- **Don't touch `ergotree-ir` or `ergotree-interpreter`** unless you have evidence of a serialization bug at the IR layer (S58 was such a case — pre-v3 Upcast(Const) round-trip).
- Run `cargo test -p ergoscript-compiler` — don't run workspace-wide tests unless needed.
- `UPDATE_EXPECT=1 cargo test` to auto-update `expect_test` snapshots.
- There is a pre-existing `unused_imports` deny on `ergotree-ir/src/chain/ergo_box/register.rs:327` that breaks `cargo test -p ergotree-ir --lib` on baseline. `cargo build -p ergotree-ir` works. Not caused by recent sessions; fix separately if it gets in your way.

---

## Session Status

Sessions 1–18 (foundational language work, fine-grained):

| Session | Feature | Test Target |
|---|---|---|
| 1 | Lexer + bool/comparison ops | `{ sigmaProp(HEIGHT > 0 && HEIGHT < 100) }` |
| 2 | Block expressions + val bindings | `{ val x: Long = 5L; sigmaProp(x > 0L) }` |
| 3 | Method calls + SELF/INPUTS/OUTPUTS | `{ sigmaProp(SELF.value > 0L) }` |
| 4 | Index access + collection ops + tuples | `SELF.tokens.size > 0 && SELF.tokens(0)._2 == 1L` |
| 5 | Lambda expressions | `INPUTS.filter { (b: Box) => ... }` |
| 6 | Built-in functions + string literals | Governance vault.es compiles |
| 7 | If/else + generic type annotations | vault + reserve compile, byte-match |
| 8 | CONTEXT + registers + .get | 3/7 governance contracts |
| 9 | Tuple types + SigmaProp ops + fold + 20 features | 7/7 governance + 17 p2p-options |
| 10 | Constant folding + val inlining + negation elim | 24+ expressions byte-match |
| 11 | Common Subexpression Elimination | CSE at top-level + lambdas |
| 12 | Graph IR CSE + selective hash-consing | 12/15 batch byte-match |
| 13 | CSE parity: ThunkDef scoping + SigmaAnd/SigmaOr | 15/15 batch byte-match |
| 14 | Language features + Atleast traversal fix | 28/31 total byte-match |
| 15 | Multi-pass inlining + RHS dedup + catch-all audit | 30/31 |
| 16 | Crystal Pool ThunkDef scope + Phoenix BigInt auto-upcast | 31/31 |
| 17 | Phoenix HodlERG BigInt type propagation + CSE parity | 31/31 native byte-match |
| 18 | Ecosystem corpus + fromBase58 + append + flatMap | 31/31 core, 3/14 ecosystem |

Sessions 19–60 (ecosystem byte-match parity push, milestone-grouped):

| Range | Milestone | Result |
|---|---|---|
| S19–S20 | DFS val-id reassignment landed | 4/14 ecosystem |
| S21–S40 | Graph-IR DAG-counting parity, scope semantics | 8–11/14 |
| S43–S52 | Triple gate (`is_collectible` / `is_extractable` / `needs_check`); `body_walk_children` invariant; const-RHS partition (S47); 46-contract corpus | 45/46 native |
| S55 | `is_bare_const` Root-mode scope check | 12/14 ecosystem |
| S57–S58 | ergotree-ir narrowing for pre-v3 `Upcast(Const, SBigInt)` round-trip (`expr.rs`, `bin_op.rs`, writer `tree_version`) | 12/14 |
| S59 | Cross-condition-branch SelectField seeding (§2b) — OpenOrderERG flips | 13/14 |
| S60 | Disambig-first + outer-scope ValUse filter — OpenOrderToken flips | **14/14 ecosystem, 45/46 native** |

For per-session deep dives, file lists, and exact diffs, see `SESSION-NN-HANDOFF.md` and `git log`.

### Remaining gaps

- `#39 DuckPools InterestRate` — skipped (CSE recursion / stack overflow on deep BigInt polynomial). Needs iterative CSE or recursion depth limit.
- Live status (including any new contracts added under the significant-15 expansion): see [ERGOSCRIPT-COMPILER-STATUS.md](ERGOSCRIPT-COMPILER-STATUS.md).

---

## File-Modification Tables for S1–S4

Removed from this guide; reconstruct from `git log --oneline ergoscript-compiler/` if needed. The Step-by-Step section above describes the same surface area in pipeline order.
