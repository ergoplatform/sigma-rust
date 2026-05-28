# Contributing to `ergoscript-compiler`

This crate compiles ErgoScript source to ergotree IR. It is a pure-Rust
alternative to the reference Scala compiler (sigmastate-interpreter). When
the two compilers produce identical bytes for a given source we say the
output is **byte-MATCH**; when they differ the source is **diverged**.

This document covers:

1. Running tests + reading their output
2. The byte-MATCH regression gate (don't break what already works)
3. Diagnosing a new divergence
4. Closing a divergence without breaking existing MATCHes
5. The node-fallback safety net (`compile_canonical`)

The crate-level CI rules (warnings-as-errors, tests, rustfmt, clippy) are
covered by the top-level [`CONTRIBUTING.md`](../CONTRIBUTING.md). Run them
locally before pushing:

```text
RUSTFLAGS="-D warnings" cargo build -p ergoscript-compiler
cargo fmt -p ergoscript-compiler --check
cargo clippy -p ergoscript-compiler -- -D warnings
cargo test -p ergoscript-compiler
```

---

## 1. Testing harness

Three byte-match test groups, each with its own fixture set. Running them
locally requires `cargo test -- --ignored --nocapture` (they're marked
ignored because they're slow):

### Significant-15 (sig-15)

15 hand-picked real-world contracts living in
`tests/fixtures/significant_15/`. The harness compiles each, byte-compares
to the Scala node's output, and reports MATCH or USED NODE.

```text
cargo test -p ergoscript-compiler test_significant_15 -- --ignored --nocapture
```

Expected tail of output:

```text
=== sig-15 summary: 12 match / 3 fallback / 0 skip / 0 error ===
```

The 3 fallbacks (sigmao, paideia, gluon) are documented architectural
ceilings — see §4 below for context.

### Ecosystem batch

14 real-world contracts from deployed Ergo projects in
`tests/fixtures/ecosystem/`. Same harness shape.

```text
cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture
```

Expected tail:

```text
=== Results: 14 local match, 0 node fallback, 0 compile errors, 0 node unavailable out of 14 ===
```

### F.2 differential fuzz corpus

575 generated programs covering language-construct combinations. Counts
MATCH / DIFF / BOTH_FAIL (= both compilers correctly reject).

```text
cargo test -p ergoscript-compiler --test diff_fuzz test_diff_fuzz -- --ignored --nocapture
```

Expected tail:

```text
MATCH      : 563
DIFF       : 10
RUST_FAIL  : 0
SCALA_FAIL : 0
BOTH_FAIL  : 2
```

---

## 2. The regression gate

The current state is **the floor**. Any change to compiler logic must
preserve every fixture currently at MATCH. If a change moves a sig-15 or
ecosystem fixture from MATCH to USED NODE — or shifts an F.2 program from
MATCH to DIFF — the change is a regression and should not land without
analysis.

The simplest regression discipline:

1. Before edit, save baseline:
   ```text
   cargo test -p ergoscript-compiler test_significant_15 -- --ignored --nocapture > /tmp/sig15_before.txt 2>&1
   cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture > /tmp/eco_before.txt 2>&1
   ```
2. Make edit.
3. Re-run, diff against baseline:
   ```text
   cargo test -p ergoscript-compiler test_significant_15 -- --ignored --nocapture > /tmp/sig15_after.txt 2>&1
   diff /tmp/sig15_before.txt /tmp/sig15_after.txt
   ```
4. If any LOCAL MATCH became USED NODE in the diff, the change regressed
   something. Either narrow the change, or document why the regression
   is acceptable.

`LOCAL MATCH` vs `USED NODE (local N bytes)` is binary — don't paraphrase.
Quote the actual test output verbatim in PR descriptions and commit
bodies. The `probe_sig15_local_hex` helper only emits LOCAL hex without
comparing to NODE — it does **not** verify byte-MATCH and shouldn't be
used as such.

---

## 3. Diagnosing a new divergence

If a new fixture (or an edit that legitimately changes output) introduces
a divergence:

1. **Get both byte streams**. Compile locally, then use `compile_canonical`
   (see §5) to grab the node-produced reference bytes.
2. **First-diff offset**. Compare hex byte-by-byte from offset 0; find
   the lowest offset where they differ. Often the divergence is in the
   constants pool layout (early in the file) or a specific outer ValDef.
3. **Outer ValDef multiset**. Dump LOCAL and NODE outer ValDefs (the
   `debug_<fixture>` helper functions in `compiler.rs` do this when
   `SIG15_DUMP_OUTER_SHAPES_VERBOSE=1` is set). Diff the multisets:
   shapes appearing in NODE only (`N-only`), in LOCAL only (`L-only`),
   or in both (`common`).
4. **Classify** the residual. Common classes encountered so far:
   - **Walker-completeness** — a CSE walker (`replace_all`,
     `contains_val_use`, `direct_children`, etc.) is missing an arm for
     a specific `Expr` variant; the walker silently passes through
     without recursing. Fix: add the missing arms.
   - **HIR-layer constant fold gap** — Scala folds a Const+Const pattern
     at compile time that Rust emits as a runtime BinOp. Fix: extend the
     fold in `mir/lower.rs` for the specific pattern.
   - **Inner-If LCA bump** — a sub-expression appears outer-once plus in
     ≥2 inner-If branches; LOCAL leaves it at branch scope, NODE places
     it at the surrounding scope via `findGlobalDefinition`. Fix: a
     narrow predicate at the global-bump loop in
     `process_ast_graph_branch`.
   - **Cross-driver dispatch routing** — `contains_func_value` and
     similar dispatch gates determine which top-level CSE path a
     fixture takes; missing arms there shift fixtures between paths,
     changing output. Fix: keep these walkers comprehensive.

---

## 4. Closing a divergence safely

The pattern that has worked across the existing closures:

1. **Probe 1 (empirical anchor)** — dump the actual byte/structural
   divergence before proposing a fix. Don't theorize about what's wrong
   without measuring it. Byte-encoding math matters: a "+N byte
   improvement" claim that doesn't survive `Δ = X·(N-1) - 2·(N+1)`
   arithmetic for the specific shape (where X is the encoded RHS size
   and N is use count) is a false signal.
2. **Narrow predicate**. Match only the specific Expr-shape signature
   that the divergent fixture has. Use existing predicates in
   `process_ast_graph_branch` and the various `is_*` helpers as
   templates. Broad gates almost always regress something elsewhere.
3. **Cross-fixture preflight**. Before committing, run all three test
   groups (sig-15, ecosystem, F.2) under default config AND with your
   new gate active. Quote the verbatim output in your commit body.
4. **One root cause per commit**. If a fix incidentally closes multiple
   F.2 programs because they share a root cause, that's fine. But if
   you're bundling two unrelated changes (e.g. one walker arm fix + one
   constant fold) into one commit, split them — review burden vs.
   review yield matters.

There are currently three sig-15 fixtures (sigmao, paideia, gluon) where
the divergence is at the hash-cons / per-Thunk distinct sym layer, which
the current CSE architecture cannot reach. They fall back to node bytes
via `compile_canonical` (§5) and that is the accepted resting state.

---

## 5. Node-fallback safety net — `compile_canonical`

`compile()` runs Rust-only and returns whatever bytes the compiler
produces. `compile_canonical()` additionally calls out to a configured
Ergo node, compares byte-by-byte, and on mismatch returns the NODE bytes
instead of the LOCAL bytes:

```rust
let result = compile_canonical(source, env, node_url, api_key)?;
// result.bytes == NODE bytes on mismatch, LOCAL bytes on match
// result.used_node tells you which path produced the bytes
```

This means downstream callers can use this crate even for fixtures the
compiler doesn't byte-MATCH yet — the node-fallback guarantees consensus
compatibility. The compiler's job is to expand the MATCH set over time;
the fallback prevents shipping incorrect bytes in the meantime.

When proposing a PR that adds a new fixture or closes a divergence:

- If the fixture is now LOCAL MATCH: great, no fallback needed at runtime
- If it's still USED NODE: document the residual class (per §3) so
  future contributors have a starting anchor

---

## What to put in a PR description

Per the top-level repo's guidance, PR descriptions should be human-written.
The maintainers (sethdusek, kushti) review the methodology as much as the
code. A useful template:

- **What changed**: 1-2 sentences on the actual code edit.
- **Why**: the empirical evidence (which fixture, which divergence,
  measured how).
- **Verbatim test output**: quote the relevant `=== sig-15 summary ===` /
  `=== Results ===` lines from before and after.
- **Risk surface**: which other fixtures could plausibly be affected by
  this change and why they aren't (cross-fixture preflight result).
- **What's NOT in this PR**: scope boundaries to help reviewers know
  what to skip.

Avoid LLM-generated walls of text. Avoid bundling unrelated changes.
Avoid claiming "preserved" without quoting the test output that proves it.
