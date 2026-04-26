# ErgoScript Rust Compiler — Skill Guide

How to add a new language feature to the `ergoscript-compiler` crate.

## Pipeline Overview

```
source text → Lexer → Parser → AST → HIR → Binder → Type Infer → MIR Lower → ErgoTree
```

Every new feature must be threaded through all stages. The order below is the order you should implement.

## Step-by-Step: Adding a Feature

### 1. Lexer (`src/lexer/token_kind.rs`)

Add new `TokenKind` variants with `#[token("...")]` or `#[regex("...")]` attributes (logos crate).

**Rules:**
- Multi-char tokens must come before single-char (e.g., `==` before `=`, `>=` before `>`)
- Keywords must come before the `Ident` regex (logos priority: exact token > regex)
- Add a `Display` impl arm for each new token
- Add a unit test: `check(">=", TokenKind::GtEq)`

### 2. Syntax (`src/syntax.rs`)

Add matching `SyntaxKind` enum variants and extend the `From<TokenKind>` impl. Also add any new **composite** node kinds here (e.g., `BoolLiteral`, `FuncCall`, `BlockExpr`).

**Gotcha:** `SyntaxKind` uses `FromPrimitive`/`ToPrimitive` derive macros. The enum must have contiguous discriminants — don't skip numbers.

### 3. Parser (`src/parser/grammar/expr.rs`)

The parser uses Pratt parsing with binding powers. Key concepts:
- **`lhs()`** — parses primary expressions (literals, idents, prefix ops, parens, blocks)
- **`expr_binding_power()`** — handles infix operators in a loop

**Binding power table (as of Session 1):**
```
||           : (1, 2)
&&           : (3, 4)
==, !=       : (5, 6)
>, <, >=, <= : (7, 8)
+, -         : (9, 10)
*, /         : (11, 12)
unary -, !   : ((), 13)
postfix call : 15
```

**To add an infix operator:** Add an `else if p.at(TokenKind::Xxx)` branch in the loop, map to a `BinaryOp` variant, and define binding power.

**To add a prefix operator:** Add a branch in `lhs()` with its own function (like `prefix_not`).

**To add postfix syntax:** Check for the token at the top of the loop (before infix ops), use `lhs.precede(p)` to wrap.

**Tests:** Use `expect_test` for CST snapshots. Run `UPDATE_EXPECT=1 cargo test` to auto-generate expected output.

### 4. AST (`src/ast.rs`)

Add a new variant to `enum Expr` and its cast in `Expr::cast()` matching on `SyntaxKind`. Create a struct that wraps `SyntaxNode` and provides typed accessor methods (like `BoolLiteral::value()`, `FuncCall::func_name()`).

**Pattern for accessors:**
- `.children()` to find child nodes
- `.children_with_tokens()` to find tokens (operators, keywords)
- `.first_token()` for leaf nodes

### 5. HIR (`src/hir.rs` + `src/hir/rewrite.rs`)

Add to `ExprKind` enum. Implement lowering in `Expr::lower()` — map from the AST node to HIR, setting `tpe` where known (e.g., `Bool(true)` → `tpe: Some(SType::SBoolean)`).

**Critical:** Also add a match arm in `src/hir/rewrite.rs` so the binder and type inference can recurse into the new node type. Forgetting this causes silent failures where child nodes don't get resolved.

### 6. Binder (`src/binder.rs`)

The binder resolves identifiers to their meanings. Currently handles:
- `HEIGHT` → `GlobalVars::Height`
- Function calls — recursively binds arguments
- Binary expressions — recursively binds lhs/rhs

**To add a new global:** Add a match arm in the `"HEIGHT"` match block (e.g., `"SELF"`, `"INPUTS"`).

**To add a built-in function:** No special binder handling needed — just leave it as `FuncCall` and handle in MIR lowering.

### 7. Type Inference (`src/type_infer.rs`)

Assign types to expressions that don't have them yet. Uses `hir::rewrite` to walk the tree.

**Rules by operator type:**
- Arithmetic (`+`, `-`, `*`, `/`): result type = operand type
- Comparison (`>`, `<`, `==`, etc.): result = `SBoolean`
- Logical (`&&`, `||`): result = `SBoolean`
- Function calls: look up return type by name (e.g., `"sigmaProp"` → `SSigmaProp`)

### 8. MIR Lowering (`src/mir/lower.rs`)

Map HIR nodes to `ergotree_ir::mir::*` types. This is the final compilation step.

**Key mappings:**
- `hir::BinaryOp` → `BinOpKind` (via the `From` impl): arithmetic → `ArithOp`, comparison → `RelationOp`, logical → `LogicalOp`
- `hir::Literal::Bool` → `Constant::from(bool)` → `Expr::Const`
- `hir::FuncCall("sigmaProp")` → `BoolToSigmaProp { input }`
- `hir::GlobalVars::Height` → `GlobalVars::Height`

**Type checking:** After lowering, the function verifies `mir.tpe() == hir_tpe`. If they don't match, something is wrong in type inference.

**Available ergotree_ir types** (in `~/.cargo/registry/src/*/ergotree-ir-0.28.0/src/mir/`):
- `bin_op.rs` — BinOp, ArithOp, RelationOp, LogicalOp
- `bool_to_sigma.rs` — BoolToSigmaProp
- `global_vars.rs` — GlobalVars (Height, SelfBox, Inputs, Outputs, etc.)
- `constant.rs` — Constant (From<bool>, From<i32>, From<i64>, etc.)
- `func_value.rs` — FuncValue (for lambdas, Session 5)
- `method_call.rs` — MethodCall (for `.method()`, Session 3)
- `val_def.rs`, `val_use.rs` — ValDef/ValUse (for val bindings, Session 2)

## Testing Strategy

### Unit tests at each level
Each pipeline file has its own `#[cfg(test)] mod tests` with a `check()` helper that runs the pipeline up to that stage.

### End-to-end test
```rust
#[test]
fn test_my_feature() {
    let result = compile_expr("{ my_source }", ScriptEnv::new());
    assert!(result.is_ok());
}
```

### Byte-for-byte verification against Ergo node
```rust
#[test]
fn test_ergotree_hex() {
    use ergotree_ir::serialization::SigmaSerializable;
    let tree = compile("{ source }", ScriptEnv::new()).unwrap();
    let bytes = tree.sigma_serialize_bytes().unwrap();
    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(hex, "expected_hex_from_node");
}
```

Get expected hex from Ergo node (localhost:9053):
```bash
source ~/.secrets
ADDR=$(curl -s -X POST "http://localhost:9053/script/p2sAddress" \
  -H "Content-Type: application/json" -H "api_key: $API_KEY" \
  -d '{"source": "{ your_source }", "treeVersion": 0}' | jq -r .address)
curl -s "http://localhost:9053/script/addressToTree/$ADDR" -H "api_key: $API_KEY" | jq -r .tree
```

## Build Notes

- **core2 → core3**: The `core2` crate v0.4.0 was yanked. We replaced it with `core3` (drop-in successor) across the workspace.
- **Don't touch `ergotree-ir` or `ergotree-interpreter`** — they're complete and production.
- Run `cargo test -p ergoscript-compiler` — don't run workspace-wide tests unless needed.
- Use `UPDATE_EXPECT=1 cargo test` to auto-update `expect_test` snapshots.

## Session Status

| Session | Feature | Status | Test Target |
|---------|---------|--------|-------------|
| 1 | Lexer + bool/comparison ops | DONE | `{ sigmaProp(HEIGHT > 0 && HEIGHT < 100) }` |
| 2 | Block expressions + val bindings | DONE | `{ val x: Long = 5L; sigmaProp(x > 0L) }` |
| 3 | Method calls + SELF/INPUTS/OUTPUTS | DONE | `{ sigmaProp(SELF.value > 0L) }` |
| 4 | Index access + collection ops + tuples | DONE | `{ sigmaProp(SELF.tokens.size > 0 && SELF.tokens(0)._2 == 1L) }` |
| 5 | Lambda expressions | DONE | `INPUTS.filter { (b: Box) => ... }` |
| 6 | Built-in functions + string literals | DONE | Governance contract (vault.es) compiles |
| 7 | If/else + generic type annotations | DONE | vault.es + reserve.es compile, if/else byte-matches node |
| 8 | CONTEXT + registers + .get | DONE | vault + reserve + timeValidator compile (3/7) |
| 9 | Tuple types + SigmaProp ops + fold + 20 features | DONE | 7/7 governance + 17 p2p-options contracts (163 tests) |
| 10 | Constant folding + val inlining + negation elim | DONE | 24+ expressions byte-match node (178 tests) |
| 11 | Common Subexpression Elimination (CSE) | DONE | CSE at top-level + inside lambdas (178 tests) |
| 12 | Graph IR CSE + selective hash-consing | DONE | 12/15 batch byte-match (180 tests) |
| 13 | CSE parity: ThunkDef scoping + SigmaAnd/SigmaOr | DONE | 15/15 batch byte-match (192 tests) |
| 14 | Language features + Atleast traversal fix | DONE | 28/31 total byte-match (196 tests) |
| 15 | Multi-pass inlining + RHS dedup + catch-all audit | DONE | 30/31 total byte-match (196 tests) |
| 16 | Crystal Pool ThunkDef scope + Phoenix BigInt auto-upcast | DONE | 31/31 total byte-match (196 tests) |
| 17 | Phoenix HodlERG BigInt type propagation + CSE parity | DONE | 31/31 native byte-match, 0 canonical (196 tests) |
| 18 | Ecosystem corpus + fromBase58 + append + flatMap | DONE | 31/31 core match, 14/15 ecosystem compile, 3/14 match (198 tests) |

### Session 20 handoff: ecosystem byte-match progress

**Result:** BondContractERG now matches (4 ecosystem LOCAL MATCH, up from 3). 198 core tests pass.

**What was done:**

1. **DFS val ID reassignment (`dfs_reassign_val_ids` in cse.rs):** New pass added between `deduplicate_inner_consts` and `reorder_valdefs` in the `apply_cse` pipeline. Walks the result expression in DFS order, collects val IDs in encounter order, and reassigns them so that `reorder_valdefs`' sort-by-ID produces the same ordering as Scala's graph schedule. Uses iterative DFS to avoid stack overflow. Only applies to the top-level BlockValue (inner blocks handled by `reorder_valdefs` independently).

2. **Attempted but reverted:**
   - Removing PropertyCall(ValUse) from scope check → broke dexy-style LP (147 vs 146B)
   - Changing `is_graph_shared` to use `is_input_stable` for box accessors (ExtractAmount etc.) → broke Oracle Pool v2 (over-extraction)
   - Disabling HIR `inline_single_use_vals` entirely → stack overflow on large contracts
   - HIR `has_shared_field_access` check → also broke dexy-style LP

**Root cause analysis for remaining 10 contracts:**

The remaining contracts share a common blocker: **HIR val inlining vs CSE scope interaction**. Our HIR optimizer (`optimize.rs:inline_single_use_vals`) inlines single-use vals before CSE runs. Scala's graph IR processes ALL user vals as graph nodes first, counts usages on the pre-inline DAG, then inlines during tree reconstruction. This means:

- **DuckPools Repayment (1B):** Node extracts `initalPool.tokens` (PropertyCall on val-bound box). In Scala, it appears in two ValDef RHSes (main scope) → DAG count 2 → extracted. After our HIR inlining, both vals are inlined into the `&&` chain (thunk positions) → scope check blocks extraction.

- **EXP_BondContractERG (same size 182B, different bytes):** Node has 10 vals, we have 8. Extra 2 are `repaymentBox.propositionBytes` and `repaymentBox.value`. Same pattern — user vals containing these subexpressions get inlined, moving the subexpressions into thunk positions.

- **Lilium CollectionIssuance (110 vs 113B):** Node has 3 vals, we have 2. Extra 1 is `SigmaPropBytes(ValUse(txOperatorPK))`. Similar CSE extraction gap.

- **Larger contracts (BondContractToken, DuckPools ParentInterest/ProxyBorrow, OpenOrder*):** Combinations of the above issues plus no-segregation fallbacks.

**Key discriminator found but not yet exploitable:**
- DuckPools' `initalPool.tokens` SHOULD be extracted (Scala does). Both occurrences were in main-scope ValDef RHSes before HIR inlining moved them to thunks.
- dexy-style LP's `out.tokens` should NOT be extracted (Scala doesn't). Even though both occurrences were also in main-scope ValDef RHSes before HIR inlining.
- The difference is unclear — may be related to Scala's ThunkDef-scoped hash-consing or the ByIndex input type (constant vs runtime index). Needs Scala IR debug logging to resolve.

**Fix direction for next session:**
The most promising approach is to modify the HIR optimizer to preserve vals whose inlining would move shared subexpressions from main scope to thunk positions. The challenge is distinguishing which vals to keep (DuckPools' `borrow0`/`validInitialPool`) from which to inline (dexy-style LP's `outValid`/`deltaY`). A Scala IR debug trace would clarify the exact extraction criteria.

**Alternative:** Implement scope-aware DAG counting in `process_ast_graph` that counts usages while ignoring ThunkDef boundaries, matching Scala's `flatSchedule` counting. This would avoid needing to change HIR inlining.

**Verification:** `cargo test -p ergoscript-compiler` (198 passed, 0 failed) + `test_ecosystem_batch --ignored` (4 match, 10 fallback).

### Session 19 handoff: ecosystem byte-match parity

**Goal:** Get 11 ecosystem contracts from USED NODE → LOCAL MATCH.

**Root cause (all 11):** CSE val extraction ordering. Our binder assigns ValIds in source order (1, 2, 3... as val definitions appear in the source). Scala's `buildTree`/`processAstGraph` assigns IDs in DFS graph traversal order from the root expression. When `reorder_valdefs` does DFS from result and sorts If-branch refs by val ID, the IDs are already "wrong" — they reflect source order, not graph order.

**Example — BondContractERG (146B, same size but different bytes):**
- Node: `val1=OUTPUTS(0)`, `val5=SELF.R5[SigmaProp]` — DFS from result sees repaymentBox first
- Local: `val1=SELF.R5[SigmaProp]`, `val5=OUTPUTS(0)` — source order has borrowerPK first

**Fix approach:** In `process_ast_graph` (cse.rs), after DAG construction and CSE extraction but BEFORE `sequential_renumber`, re-assign val IDs in DFS traversal order from the result expression. This makes `reorder_valdefs` sort produce the same ordering as Scala. The 31 core contracts already match (their DFS order happens to coincide with source order), so this should be safe if done correctly.

**Specific contracts to target (easiest first):**
1. BondContractERG — 146B same size, pure val reordering
2. DuckPools Repayment — 188 vs 189B, 1 byte diff (likely constant store ordering)
3. Lilium CollectionIssuance — 110 vs 113B, small
4. EXP_BondContractERG — 182B same size
5. BondContractToken — 231 vs 223B
6. DuckPools ParentInterest — 409 vs 412B
7. DuckPools ProxyBorrow — 434 vs 440B
8. SigmaFi OpenOrderERG — 440 vs 471B (BigInt fees, complex)
9. SigmaFi OpenOrderToken — 572 vs 638B (no-segregation fallback)
10. SkyHarbor SigUSDV1 — 459 vs 510B (no-segregation fallback)
11. Lilium SaleLP — 298 vs 317B (flatMap, no-segregation fallback)

**Also pending:**
- DuckPools InterestRate — CSE stack overflow on `(f*x)/D*x/M*x/M*x/M*x/M`. Needs iterative CSE or recursion depth limit.
- 3 contracts use non-segregated fallback — once CSE ordering is fixed, the ValDef scoping issue may resolve too (they may start passing the `ErgoTree::new` roundtrip).

**Verification:** After every change run `cargo test -p ergoscript-compiler` (198 passed, 0 failed) + `test_ecosystem_batch` (--ignored).

### Session 18 changes
- **fromBase58:** Compile-time constant fold identical to fromBase16. Decodes Base58 string literal to `Const(Coll[Byte])` via `bs58` crate. Added to `type_infer.rs` and `mir/lower.rs`.
- **append on collections:** Added `Append::new(obj, col2)` lowering in `mir/lower.rs`. CSE already handled `Expr::Append`. Type: `SColl(T).append(SColl(T)) → SColl(T)`.
- **flatMap on collections:** Uses `MethodCall` with `FLATMAP_METHOD` from `ergotree_ir::types::scoll`, specialized via `specialize_for()` for concrete types. Type: `SColl(A).flatMap(A => Coll[B]) → SColl(B)`.
- **.toBigInt on BigInt:** Identity no-op — when input is already `SBigInt`, emit input unchanged instead of `Upcast`. Added `SBigInt` arm to `type_infer.rs`.
- **Constant segregation fallback:** `compile()` now falls back to non-segregated ErgoTree (`v0(false)`) when the `ErgoTree::new` serialize→deserialize roundtrip fails. Root cause: CSE extracts vals into ThunkDef scopes whose ValDefs aren't encountered during linear re-parsing.
- **Ecosystem test corpus:** 15 real-world contracts from SigmaFi (5), SkyHarbor (1), DuckPools (4+1 skipped), Lilium (5). 3 local match, 11 byte-diff (CSE ordering), 1 skipped (CSE stack overflow).

### Session 17 changes
- **Type propagation pass (→ Phoenix 309B→312B):** Added `propagate_val_types()` in `lower.rs`. After MIR lowering, walks the tree collecting actual ValDef RHS types, updates ValUse types (fixing `val x: Long = BigInt_expr` annotation mismatches), and re-applies `numeric_upcast_pair` on BinOps. Wired into pipeline between `lower()` and `apply_cse()`.
- **CSE If-branch ThunkDef scoping:** Extended `appears_in_main_scope_inner` to treat If true/false branches as ThunkDef scopes. Added `Upcast(ValUse(_) | Const(_))` to `needs_scope_check`. Prevents over-extraction of Upcast nodes that appear only in If branches.
- **Post-CSE single-use val inlining (→ 312B):** Added `inline_single_use_vals()` after CSE extraction. When CSE extracts `Upcast(ValUse(x), BigInt)`, the original val `x` may become single-use — inlining folds it into `Upcast(ExtractAmount(Self), BigInt)` matching Scala's combined form.
- **Inner-block constant dedup (→ 314B same size):** Added `deduplicate_inner_consts()`. After the Upcast scope check prevents extracting `Upcast(Const(1000L), BigInt)` from If branches, duplicate `Const(1000L)` nodes remain. This pass extracts them as vals within inner BlockValues, preventing duplicate ConstantStore entries. Only operates on blocks inside If branches.
- **If-branch val ordering via freeVars sort (→ 314B exact match):** Changed `emit_deps` for If to collect all ValUse references from both branches, sort by val ID (matching Scala's symbol-ID-ordered ThunkDef freeVars), and emit in that order. Made `reorder_valdefs` recursive to also reorder inner blocks via `map_children`.

### Session 16 changes
- **Crystal Pool fix (→ match):** Extended `process_ast_graph` ThunkDef scope check to cover `ByIndex` on globals (e.g., `ByIndex(Outputs, 0)`). The root cause was that `OUTPUTS(0)` appeared only inside separate `&&` right arms (ThunkDef scopes), but the scope check only covered ValUse-dependent expressions. Now `ByIndex(GlobalVars, ...)` is also checked.
- **BigInt auto-upcast:** Added `numeric_upcast_pair` in MIR lowering. When a `BinOp` has one BigInt operand and one smaller numeric type, the smaller is auto-upcast to BigInt. This matches Scala's implicit numeric promotion. Also relaxed the type check to allow BigInt results assigned to Long-annotated vals.

### Session 15 changes
- **Multi-pass val inlining:** Dead-code elimination now cascades — after removing dead vals, use counts are recalculated and previously multi-use vals may become single-use. Fixed SigmaUSD (+8B → match).
- **RHS dedup pass:** Before inlining a single-use val, check if its RHS appears elsewhere in the block. If so, keep the val and replace duplicate occurrences with ValUse. Fixed Off-the-grid.
- **sigmaProp no-op:** `sigmaProp(X)` where X is already SSigmaProp (e.g. SigmaOr) now elides the redundant BoolToSigmaProp wrapper.
- **Catch-all audit:** All non-leaf Expr variants now have explicit arms in `collect_and_assign_ids` and `rewrite_ids` — no more silent `_ => {}` catch-alls that skip lambda param renumbering.

### Remaining byte-match gaps: NONE

All 31 contracts produce bytecode identical to the Scala reference node. Phoenix HodlERG (#25) was the final contract — fixed in Session 17 via type propagation, CSE scope fixes, constant dedup, and val ordering alignment.

### Known missing traversals
The `collect_and_assign_ids` and `rewrite_ids` functions in `cse.rs` use `_ => {}` / `_ => other` catch-alls. Any new expression type must be explicitly added to BOTH functions or lambda params inside that expression type will get wrong IDs. Currently handled: BlockValue, ValDef, FuncValue, BinOp, BoolToSigmaProp, If, Filter, Exists, ForAll, Map, Fold, Append, PropertyCall, MethodCall, Extract*, SizeOf, ByIndex, SelectField, OptionGet, OptionIsDefined, OptionGetOrElse, Slice, LogicalNot, Negation, SigmaPropBytes, Upcast, Downcast, CalcBlake2b256, CreateProveDlog, SigmaAnd, SigmaOr, Tuple, TreeLookup, Apply, And, Or, Collection, Atleast.

## Files Modified in Session 1

| File | Changes |
|------|---------|
| `src/lexer/token_kind.rs` | +18 tokens (comparison, boolean, punctuation, keywords, string literals) |
| `src/syntax.rs` | +18 SyntaxKind variants + 3 composite nodes (BoolLiteral, FuncCall, BlockExpr) |
| `src/parser/grammar/expr.rs` | Comparison/boolean ops in Pratt parser, func call postfix, block expr, bool literal |
| `src/parser/parse_error.rs` | Fixed pre-existing unicode curly quote bug in test expectations |
| `src/ast.rs` | +3 Expr variants (BoolLiteral, FuncCall, Block) with accessor methods |
| `src/hir.rs` | +6 BinaryOp variants, +FuncCall struct, +Block, +Literal::Bool, lowering for all |
| `src/hir/rewrite.rs` | +FuncCall and Block recursive rewrite arms |
| `src/binder.rs` | Recursive binding for FuncCall args and Binary operands |
| `src/type_infer.rs` | Type rules for comparison→SBoolean, logical→SBoolean, sigmaProp→SSigmaProp |
| `src/mir/lower.rs` | RelationOp, LogicalOp mapping, Bool→Constant, sigmaProp→BoolToSigmaProp, Block |
| `src/compiler.rs` | End-to-end tests including ErgoTree hex byte-for-byte match |

## Files Modified in Session 2

| File | Changes |
|------|---------|
| `src/lexer/token_kind.rs` | +Semicolon token |
| `src/syntax.rs` | +Semicolon SyntaxKind |
| `src/parser/grammar/stmt.rs` | Optional `: Type` annotation in `variable_def()` |
| `src/parser/grammar/expr.rs` | Semicolon consumption in block_expr |
| `src/ast.rs` | +VariableDef Expr variant with name(), type_annotation(), value() accessors |
| `src/hir.rs` | +ValDef(name, id, tpe, rhs), +ValUse(id, tpe), +parse_type_name() helper |
| `src/hir/rewrite.rs` | +ValDef and ValUse match arms |
| `src/binder.rs` | Complete rewrite: Scope struct with define/lookup, recursive bind_expr, ident→ValUse resolution |
| `src/type_infer.rs` | +ValDef type from annotation or RHS, +ValUse type passthrough, +Block type = last item |
| `src/mir/lower.rs` | +ValDef→ergotree_ir::ValDef, +ValUse→ergotree_ir::ValUse, Block→BlockValue |
| `src/compiler.rs` | Session 2 end-to-end tests + serialization roundtrip verification |

### Session 2 Notes
- **No constant folding**: The Scala compiler constant-folds `val x = 5L; x > 0L` → `true`. Our compiler produces the unoptimized `BlockValue { ValDef, BoolToSigmaProp(BinOp(Gt, ValUse, Const)) }`. This is semantically correct but won't byte-match the node for trivially-foldable expressions.
- **ValId assignment**: The binder assigns sequential `ValId(0)`, `ValId(1)`, etc. within each block scope.
- **Type inference from RHS**: If no type annotation, the type is inferred from the RHS literal type (e.g., `val x = 5L` → `SLong`).

## Files Modified in Session 3

| File | Changes |
|------|---------|
| `src/syntax.rs` | +FieldAccess SyntaxKind |
| `src/parser/grammar/expr.rs` | Dot postfix in Pratt parser (binding power 17), produces FieldAccess node |
| `src/ast.rs` | +FieldAccess Expr variant with object()/field_name() accessors |
| `src/hir.rs` | +FieldAccessExpr, +GlobalVars::SelfBox/Inputs/Outputs with SBox/SColl(SBox) types |
| `src/hir/rewrite.rs` | +FieldAccess match arm (recurses into object) |
| `src/binder.rs` | +SELF/INPUTS/OUTPUTS global resolution, FieldAccess recursive binding |
| `src/type_infer.rs` | Box property type lookup (.value→SLong, .id→SColl(SByte), etc.), SColl .size→SInt |
| `src/mir/lower.rs` | +ExtractAmount/ExtractScriptBytes/ExtractId/ExtractCreationInfo/ExtractBytes/SizeOf, GlobalVars mapping |
| `src/compiler.rs` | Session 3 tests + ErgoTree hex byte-for-byte match |

### Session 3 Notes
- **Box field access** maps to dedicated Extract* IR nodes, NOT PropertyCall. Each has its own opcode for efficient serialization.
- **Byte-for-byte match**: `{ sigmaProp(SELF.value > 0L) }` → `10010500d191c1a77300` matches the Ergo node exactly.
- **Dot binding power 17** is highest, ensuring `SELF.value > 0L` parses as `(SELF.value) > 0L`.

## Files Modified in Session 4

| File | Changes |
|------|---------|
| `src/lexer/token_kind.rs` | Ident regex updated to `[A-Za-z_][A-Za-z0-9_]*` (supports `_1`, `_2`) |
| `src/error.rs` | Fixed overflow bug: `saturating_sub(1)` instead of `- 1` for span start=0 |
| `src/ast.rs` | FuncCall.func_expr() returns Expr instead of just name string |
| `src/hir.rs` | Replaced FuncCall with Apply { func: Box<Expr>, args } for generalized application |
| `src/hir/rewrite.rs` | Apply match arm (recurses into func + args) |
| `src/binder.rs` | Apply handling with recursive func/args binding |
| `src/type_infer.rs` | Apply type: Ident("sigmaProp")→SSigmaProp, SColl→elem type. STuple field `_N` type lookup |
| `src/mir/lower.rs` | Apply→BoolToSigmaProp or ByIndex. .tokens→PropertyCall(TOKENS_METHOD). `_N`→SelectField |
| `src/compiler.rs` | Session 4 end-to-end + roundtrip tests |

### Session 4 Notes
- **FuncCall→Apply refactor**: `ExprKind::FuncCall { name, args }` replaced by `ExprKind::Apply { func, args }` where func is any Expr. MIR lowering checks func type: Ident→built-in function, SColl→ByIndex.
- **CSE gap**: Scala compiler does common subexpression elimination (e.g., `SELF.tokens` evaluated once via val binding). Our compiler evaluates it each time — semantically correct but different bytes.
- **PropertyCall for .tokens**: Uses `ergotree_ir::types::sbox::TOKENS_METHOD` lazy_static.
- **SelectField for ._N**: Uses 1-based `TupleFieldIndex` from `select_field.rs`.
