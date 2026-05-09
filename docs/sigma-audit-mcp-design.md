# `sigma-audit-mcp` — Design Spec (Session A deliverable)

**Status:** spec, no code yet.
**Owner:** CannonQ
**Created:** 2026-05-08
**Type:** MCP server design — implementation is a separate focused session.
**Related:** [AUDIT-FRAMEWORK-PLANNING-HANDOFF.md](AUDIT-FRAMEWORK-PLANNING-HANDOFF.md) §4c row 3, §10d.

---

## 1. Scope (per §10d revision)

**Drop Scala-side primitives.** Metals MCP subsumes them:
- ~`p2s_probe(es_source)`~ — replaced by metals `get-source` + Scala-side compilation if needed via metals build target invocation.
- ~`cse_trace(fixture, candidate)`~ — Scala side covered by `get-source(AstGraphs, detailed=true)`.
- ~Anything that asks "what does Scala do"~ — metals workhorse.

**Keep only Rust-side primitives** that wrap existing `debug_<fixture>` + cargo invocations into structured JSON output. Replaces grep-walking text dumps with semantic queries.

## 2. MVP tool surface (3 tools)

### 2a. `dump_ir(fixture: string)`

Wraps existing `debug_<fixture>` helper. Returns structured IR comparison.

**Input:**
```json
{ "fixture": "sigmao_option" }
```

**Output:**
```json
{
  "fixture": "sigmao_option",
  "head": "52ca2c14",
  "local": {
    "bytes": 1112,
    "header": "0x10",
    "constants_pool_size": 47,
    "constants_multiset": [{"tpe": "SLong", "count": 5}, ...],
    "outer_valdefs": [
      {"id": 32, "tpe": "SBox", "rhs_kind": "ByIndex"},
      ...
    ],
    "outer_valdef_count": 46,
    "segregation_status": "OK"
  },
  "node": {
    "bytes": 1144,
    "header": "0x10",
    "constants_pool_size": 47,
    "constants_multiset": [...],
    "outer_valdef_count": 46,
    "segregation_status": "OK"
  },
  "delta_bytes": -32,
  "first_diff_offset": 412,
  "diff_summary": "First body-byte divergence at offset 412; outer ValDef count parity; multiset parity; tail divergence likely body-walker-order class."
}
```

**Implementation:** thin Rust wrapper test that calls existing `debug_<fixture>` helpers + serializes via `serde_json`. Keep the wrapper parameterized; no per-fixture hardcoding (introspect via existing `test_significant_15` registry).

### 2b. `compare_constant_pools(fixture: string)`

Detail view of constants-pool divergence. Used when class #1 (const-fold) or class #5 (placement) is suspected.

**Input:** `{ "fixture": "sigmausd_bank" }`

**Output:**
```json
{
  "fixture": "sigmausd_bank",
  "local_only": [{"index": 33, "tpe": "SInt", "value": "-1"}],
  "node_only":  [{"index": 33, "tpe": "SLong", "value": "-1"}],
  "shared": [...],
  "diagnosis_hint": "LOCAL has SInt(-1) where NODE has SLong(-1) — likely numeric upcast fold gap; check propagateBinOp arms in DefRewriting."
}
```

**Implementation:** parses both constant pools, computes set-difference + set-intersection. The `diagnosis_hint` is a string template based on detected divergence shape — keep it simple, surface known patterns, don't over-engineer.

### 2c. `run_corpus(corpus: string, expected_match: number = -1)`

Wraps `cargo test --release ... -- --ignored` for the given corpus + parses MATCH/DIFF/RUST_FAIL/SCALA_FAIL counts.

**Input:**
```json
{ "corpus": "F.2", "expected_match": 563 }
```

**Output:**
```json
{
  "corpus": "F.2",
  "match": 563,
  "diff": 10,
  "rust_fail": 0,
  "scala_fail": 2,
  "total": 575,
  "match_pct": 97.9,
  "expected_match": 563,
  "passed_expected": true,
  "regressions": [],
  "improvements": []
}
```

For sig-15 corpus, output additionally includes per-fixture `local_match` / `used_node` rows.

**Implementation:** subprocess wrapper around `cargo test --release` with output parsing. Must be deterministic enough that the test count parses cleanly across `cargo` versions; consider a JSON-output flag if cargo supports one.

## 3. Tools intentionally NOT in MVP

| Dropped | Reason |
|---|---|
| `p2s_probe` | Metals `get-source` covers Scala-side semantics; for Rust p2sAddress probe, `cargo test` + manual is fine |
| `cse_trace` | Existing `CSE_TRACE` env-flag instrumentation is good enough; structured output isn't needed at session frequency |
| `bisect_baseline` | One-off bisection (per sigmao S3) is rare enough that ad-hoc `git bisect` is fine |
| `apply_fix_and_verify` | Removes the human review checkpoint; explicitly NOT in scope per §4b |
| Per-fixture knobs | Generic via fixture-name parameter; no per-fixture handlers |

**Rule:** add a 4th tool ONLY when a session actually needs the missing one (per §6 — "Don't preemptively wrap every helper"). The MVP is intentionally small.

## 4. Implementation sequencing (one focused session ≈ half day)

1. **Decide language.** TypeScript is the path of least resistance — matches user's other MCPs (telegram, EKB, Bigin via `@modelcontextprotocol/sdk`). Rust is possible but no MCP-server prior art in this repo.
2. **Stdio MCP scaffolding.** Standard `@modelcontextprotocol/sdk` server with 3 tools registered.
3. **Tool 1 (`run_corpus`).** Cheapest; pure subprocess wrapper. Implements pattern for the others.
4. **Tool 2 (`dump_ir`).** Requires extending Rust `debug_<fixture>` helpers to emit JSON. Add a `#[cfg(test)] fn debug_<fixture>_json()` variant per fixture, OR a single parameterized `debug_fixture_json(name: &str)` that uses the registry.
5. **Tool 3 (`compare_constant_pools`).** Reuses `dump_ir` output; pure data manipulation.
6. **Wire into Claude Code.** Add MCP server to `~/.claude.json` or project `.mcp.json` (whichever the user prefers — local stdio ideally lives at `.mcp.json` next to the repo so it's auto-loaded for sessions in this directory).
7. **Smoke test.** Run sigmao S4 / next live session through the MCP path; confirm output matches handwritten `debug_sigmao` calls.

## 5. Decision points before implementation

1. **TS vs Rust.** TS recommended; faster to write; matches existing MCPs.
2. **`.mcp.json` scope.** Project-local (in repo) vs user-global (in `~/.claude.json`). Project-local is cleaner — the MCP only makes sense in this codebase.
3. **JSON schema for `outer_valdefs`.** Pick once; don't churn. Suggest: `{id, tpe (debug-format), rhs_kind (enum string)}`.
4. **Error handling.** What does the MCP return when `debug_<fixture>` doesn't exist? Suggest: structured error with hint to create the helper per the per-debug_fixture convention.
5. **Caching.** `dump_ir` can be slow if it triggers a recompile. Cache by HEAD hash + fixture name? Or always re-run? Suggest: always re-run; honesty over speed at session frequency.

## 6. Forward-pace metric

After Session A lands, the per-fixture session bash invocations of `cargo test test_significant_15 --release -- --ignored 2>&1 | grep MATCH` collapse to `mcp__sigma_audit__run_corpus(corpus="sig-15")` returning structured JSON. The `/audit-fixture-session` skill will reference these MCP tools by name once available; until then, the skill's Probe 1 step uses the existing bash invocations.

## 7. Risks / non-obvious

- **Subprocess flakiness.** `cargo test` output format can drift across cargo versions. Pin `cargo-build-rust-toolchain.toml` if test count parsing fails.
- **MCP startup cost.** Stdio MCP processes spawn per-session; if `cargo` warm-up is slow, first call is slow. Mitigate: pre-warm by running `cargo build --release --tests` at MCP startup.
- **Schema churn.** Don't over-spec the JSON output before second use case. MVP ships ugly-but-correct; refactor when 2nd consumer (skill or another session pattern) surfaces a need.

## 8. NOT this session

Implementation is **its own focused session**. This file is the spec. The user's call when to schedule it. Recommendation: after sigmao S4 completes (which is already running). Sigmao S4's empirical needs inform whether the 3 MVP tools are right or whether one is missing.
