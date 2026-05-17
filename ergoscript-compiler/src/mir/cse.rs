//! Common Subexpression Elimination (CSE) pass on MIR.
//!
//! Detects subexpressions appearing 2+ times and lifts them to ValDefs,
//! matching the Scala ErgoScript compiler's CSE behavior.

use ergotree_ir::mir::block::BlockValue;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
use ergotree_ir::mir::val_def::{ValDef, ValId};
use ergotree_ir::mir::val_use::ValUse;
use ergotree_ir::source_span::{SourceSpan, Spanned};
use ergotree_ir::types::stype::SType;

/// Apply CSE to a MIR expression tree.
/// Runs on a dedicated thread with 16MB stack for large contracts.
pub fn apply_cse(expr: Expr) -> Expr {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            // S62 / direction #2: snapshot the outer BlockValue's user-declared
            // ValDef IDs in source order (items[] index) BEFORE strip_source_spans
            // and CSE-extraction pollute the picture. Used by `dfs_reassign_val_ids`
            // to schedule outer-block symbols by source-creation order — matches
            // Scala's TreeBuilding, which assigns symbol IDs in source-lowering
            // order, not result-DFS-encounter order. Closes Phoenix HodlERG Bank.
            let mut user_val_source_pos: HashMap<u32, usize> = HashMap::new();
            if let Expr::BlockValue(s) = &expr {
                for (idx, item) in s.expr.items.iter().enumerate() {
                    if let Expr::ValDef(vd) = item {
                        user_val_source_pos.insert(vd.expr.id.0, idx);
                    }
                }
            }

            // Normalize all source spans to empty so CSE hash-consing
            // treats structurally identical nodes as equal regardless
            // of source position. Without this, two identical expressions
            // from different source locations (e.g. SELF.R4[T].get used
            // in two places) would be treated as different nodes.
            trace_slot_shift("00-pre_cse_entry", &expr);
            let expr = strip_source_spans(expr);
            trace_slot_shift("00-strip_source_spans", &expr);
            // sig-15 paideia_stake_state S2: inline ValDefs whose RHS is a
            // primitively-context-pure alias (`ByIndex(GlobalVars, Const)`,
            // i.e. `INPUTS(N)` / `OUTPUTS(N)` aliases like
            // `val newStakeBox = OUTPUTS(1)`). Source-level aliases
            // bound inside If branches survive HIR `is_inline_always_rhs`
            // (which only handles Literal / Context / bare GlobalVars) and
            // pin a branch-local ValId. Downstream CSE then rejects every
            // candidate whose RHS references that ValId via the
            // `branch_local_ids` forward-ref guard in
            // `process_ast_graph_impl`, so structurally-identical
            // `OUTPUTS(N).tokens(...)` chains across the cond and both
            // branches never collapse into a single root-bound ValDef
            // (NODE has 36 outer ValDefs vs LOCAL's 27 here). Inlining the
            // alias ValDefs unblocks structural matching at outer scope;
            // CSE re-extracts shared chains at the LCA.
            // WS-G.2.4b Probe 1 (19th falsification, diag-only, no code change):
            // Hypothesis was that `inline_alias_vals` destroys the source-order
            // val structure relied on by the hash-cons walker, causing dexy
            // (+44) and duckpools (-2) under-extraction under `CSE_HASH_CONS=1`.
            // Empirical probe: gating this call off when `hash_cons_enabled()`
            // leaves dexy 353B / duck 596B UNCHANGED and regresses paideia
            // 1495B → 1550B (+27 → +82). Path C as stated does NOT recover
            // either fixture. Root cause must be inside the walker itself
            // (`process_ast_graph_hash_cons` / `SymTable::find_or_intern`
            // scope-chain semantics) or in a downstream pass that destroys
            // root anchoring, NOT pre-CSE `inline_alias_vals` — non-alias
            // boolean vals like dexy's `validMint` were never inlined here.
            // See target/diff_fuzz/clusters/closed/<commit>_ws-g2-4b-path-c-falsified.md.
            let expr = inline_alias_vals(expr);
            trace_slot_shift("01-inline_alias_vals", &expr);
            let global_max_id = find_max_val_id(&expr);
            let cse_result = cse_expr(expr, global_max_id, false);
            trace_slot_shift("02-cse_expr", &cse_result);
            let branch_cse_max = find_max_val_id(&cse_result);
            let branch_cse = apply_cse_within_branches(cse_result, branch_cse_max);
            trace_slot_shift("03-apply_cse_within_branches", &branch_cse);
            let pre_extract_max = find_max_val_id(&branch_cse);
            let mut next_id = pre_extract_max + 1;
            let pre_extracted = pre_extract_from_valdefs(branch_cse, &mut next_id);
            trace_slot_shift("04-pre_extract_from_valdefs", &pre_extracted);
            let inlined = inline_single_use_vals(pre_extracted);
            trace_slot_shift("05a-inline_single_use_vals", &inlined);
            let deduped = deduplicate_inner_consts(inlined);
            trace_slot_shift("06-deduplicate_inner_consts", &deduped);
            // Re-run outer inlining: Phase 3 (relational BinOp dedup, sig-15
            // sigmausd_bank close) extracts a new branch-scope ValDef whose
            // RHS may consume two prior uses of an OUTER user-bound val
            // (e.g. `oraclePoolNFT`), reducing it to a single use. The earlier
            // `inline_single_use_vals` pass ran before that drop, so the now-
            // single-use outer ValDef would otherwise survive as overhead.
            let deduped = inline_single_use_vals(deduped);
            trace_slot_shift("05b-inline_single_use_vals", &deduped);
            // QB-SESSION-10 / WS-G sig-15 sigmao S31 — Probe 1+2 of renumbering
            // pipeline. Re-enables S29 `promote_branch_emerged_s2` under
            // env gate `CSE_PROBE_S31_S2=1` so default pipeline preserves
            // 12/15 baseline; flag turns the pass on for instrumentation runs.
            //
            // S31 PROBE 0 (metals): Scala's `processAstGraph` has NO
            // renumbering pass. IDs are assigned serially per-scope during
            // emission (`var curId = defId; ...; curId += 1; ValDef(curId)`);
            // the cross-scope DAG identity is carried by hash-cons Sym
            // construction, not by ID. LOCAL's `dfs_reassign_val_ids →
            // reorder_valdefs → sequential_renumber` is a Rust-only artifact.
            //
            // S31 PROBE 1 (orphan trace): re-enabling s2 promotion at this
            // stage with `CSE_TRACE_SLOT_SHIFT=1` reveals the actual failure
            // mode — the hoisted shape `ByIdx<or>[VU(61), K(Int), VU(?)]`
            // contains a free `ValUse(61)` whose `ValDef(61)` lives in an
            // INNER scope (introduced by branch-CSE). When promoted to outer
            // scope, `ValUse(61)` is orphaned at every downstream stage
            // (07→09 keep id=61, 10→12 renumber to id=47). Disambig's
            // `Expr::ValUse` arm walks the scope chain from inner→outer and
            // bails when not found, leaving the orphan unrenamed at outer
            // scope. Segregation FALLBACK is the SYMPTOM; the CAUSE is
            // structurally invalid IR (outer body referencing inner-scope
            // binding). NO renumbering fix can address this — the bytecode
            // reader can't resolve outer ValUse→inner ValDef regardless of
            // ID assignment.
            //
            // S31 PROBE 2 (scope-soundness gate): `extract_inline_shape` now
            // computes the matched subtree's free ValUse refs (via
            // `collect_orphan_ids`) against the OUTER block's scope; refuses
            // hoists whose free refs are not all outer-bound. Empirical: both
            // candidate sigmao matches refused (orphan=[61] each); LOCAL
            // stays 1124B / Δ -24 plateau preserved; constants_len 61/61
            // segregation OK; `probe_sig15_collisions` sigmao OK; 12 MATCH
            // / paideia +2 / gluon +102 plateaus all preserved; F.2 563/575
            // / lib 251/251 / conformance 164/164 / ecosystem 11/14 all
            // preserved under both default and `CSE_PROBE_S31_S2=1`.
            //
            // S31 REFRAME (falsification fingerprint #31, new methodology
            // class — post-hoc-framing-of-downstream-symptom): the S29
            // "renumbering-pipeline-crash" framing was post-hoc — the
            // symptom (constants_len 61→0, segregation FAIL) was correctly
            // observed but the cause was attributed to the wrong layer
            // (renumbering pipeline). The actual bug is at the extraction-
            // predicate-soundness layer. Generalizes to all Cohort B
            // promotion shapes (s9/s3/s6): any "hoist inline shape to
            // outer scope" pass MUST scope-validate free vars; the
            // pipeline can't rescue an unsound hoist.
            //
            // Closure of sigmao -24 via s2 promotion at the current code
            // surface requires either (a) restructuring promotion to insert
            // the new ValDef at the LOCAL scope of the match (not outer),
            // which loses the cross-scope CSE benefit; or (b) reaching
            // inner-scope bindings via a different mechanism (e.g. hoist
            // the inner ValDef chain alongside) — both architectural
            // questions for Session 11+.
            // QB-SESSION-08 / WS-G sig-15 sigmao S29 — Cohort B s2
            // post-CSE promotion (`ByIdx<or>[VU,K(Int),VU]`) FALSIFIED at
            // Probe 2 (29th falsification fingerprint instance, this commit).
            // Probe 1 inventory across 15 sig-15 + ecosystem fixtures cleanly
            // discriminated sigmao_option (outer_vd_s2=5 / tree 5→6 at
            // stage 03) from paideia/gluon/MATCH (outer_vd_s2=0 everywhere).
            // Probe 2: insertion of `promote_branch_emerged_s2` between
            // stage 05b (`inline_single_use_vals` second pass) and stage 07
            // (`flatten_nested_blocks`) — chosen specifically to avoid the
            // single-use re-inline path that undid an earlier stage 03b
            // insertion variant. Shape-evolution probe confirms gate fires
            // correctly: sigmao s2 outer_vd 5 → 6 at stage 05c, survives
            // 07/08/09/10/11/12 downstream, L outer_vd parity 45 → 46
            // (matching NODE 46), SHAPE diff IMPROVES (common 38→39 /
            // N-only 8→7). BUT: LOCAL bytes 1124 → **1005** (segregation
            // FAIL: constants_len 61 → 0; emitter falls back to
            // unsegregated; sigmao_option `probe_sig15_collisions` baseline
            // OK → FAIL). Same renumbering-pipeline-crash class as S6
            // (#12) and S5 broad Fix A (#11) — even ONE additional
            // surgical-narrow ValDef extraction post-CSE triggers the
            // `dfs_reassign_val_ids → reorder_valdefs → sequential_renumber`
            // pipeline's failure mode when the new ValDef references
            // ValUse IDs scoped to inner BlockValues that have since been
            // disambiguated/renumbered. Cohort B byte-ADDING direction
            // hits the SAME architectural gate as Cohort A byte-SHRINKING
            // (S26/S27/S28) — confirming that the renumbering-pipeline
            // rewrite is the structural blocker for ALL extra-extraction
            // surgical attacks at the current code surface, regardless of
            // byte direction. Per QB-HANDOFF-15-OF-15 §0 anti-pattern,
            // closure requires WS-G architectural rewrite (DAG-identity
            // hash-cons migration), not narrow promotion. See
            // `target/diff_fuzz/clusters/closed/<commit>_sig15-sigmao-S29-cohort-B-s2-promotion-falsified.md`.
            // Helper functions `promote_branch_emerged_s2`,
            // `extract_inline_shape`, `map_children_extract_shape` retained
            // below as durable artifact — they correctly implement the
            // narrow predicate; the failure is downstream pipeline
            // incompatibility, not the predicate itself.
            // (Pass call REMOVED to preserve sig-15 12/15 + collisions OK
            //  baseline. Re-enable for renumbering-pipeline-rewrite probes.)
            //
            // QB-SESSION-11 / WS-G sig-15 sigmao S32 — Session 11 Probe 2
            // FALSIFIED both closure-path (b) implementation strategies:
            //
            // Strategy I (recursive free-var lift): under `CSE_PROBE_S31_S2=1
            //   CSE_S32_LIFT=1`, `promote_branch_emerged_s2` computes the
            //   transitive orphan closure (sigmao: 2 ValDefs — 60 + 61, where
            //   ValDef(61) = PropertyCall(ValUse(60), tokens) and ValDef(60)
            //   = ByIndex(Outputs, Const(2: SInt))) and LIFTS the closure
            //   to outer scope, removing the bindings from inner BlockValues.
            //   Empirical result: VALID IR (segregation OK, constants_len
            //   61/61) but LOCAL 1124B → 1128B = Δ -24 → -20 (REGRESS +4B).
            //   Outer ValDef count went 45 → 48 (NODE has 46). +1 outer s2
            //   ValDef matches NODE but +2 from the lifted closure is net
            //   over-extraction. The structural fix works but byte-arithmetic
            //   is wrong direction. Same P2.5 compensating-extraction class
            //   as S26/S27 (#26 / #27).
            //
            // Strategy II (recursive inline): replacing orphan ValUse refs in
            //   the matched shape with `inner_bindings[id].body` (recursive
            //   until self-contained) was NOT implemented in code due to
            //   invasive scaffolding cost AND a Probe-0 byte-arithmetic
            //   estimate predicting WORSE regression than Strategy I: the
            //   self-contained body becomes ByIdx(PropertyCall(ByIndex(
            //   Outputs, Const(2:SInt)), tokens), K(2), VU(?)) ≈ 25-30B; 2
            //   inline occurrences each save 8B (10B shape → 2B ValUse) =
            //   ~16B saved; new ValDef cost ~28B; predicted net ≈ +12B
            //   regression (worse than Strategy I's +4B).
            //
            // Closure path (b) FULL FALSIFICATION — both Strategy I empirical
            // and Strategy II analytical falsified by the SAME mechanism:
            // compensating-extraction. ANY structural fix that hoists orphan
            // refs to outer scope adds outer ValDef bytes faster than it
            // saves inline-occurrence bytes (sigmao's specific arithmetic).
            // 32nd falsification fingerprint instance.
            //
            // Closure path (a) (insert at LOCAL scope) was already falsified
            // in S31 (loses cross-scope CSE benefit; net zero byte movement).
            // Closure path (c) (accept plateau and pivot) remains open.
            //
            // Strategy I helpers (`collect_inner_valdefs`, `collect_match_orphans`,
            // `remove_valdefs_in_blocks`, `map_children_remove`) + `extract_inline_shape`
            // scope-soundness gate retained `#[allow(dead_code)]` as durable
            // artifact. Pass call remains REMOVED — closure-path (b) attack
            // surface for sigmao is empirically EXHAUSTED at the current code
            // surface; closure requires WS-G architectural rewrite per
            // QB-HANDOFF-15-OF-15 §0.
            let flattened = flatten_nested_blocks(deduped);
            trace_slot_shift("07-flatten_nested_blocks", &flattened);
            // sig-15 paideia_stake_state S5: post-PAG chain-rewrite for
            // outer ValDefs whose RHS is a stable chain link. Sub-passes
            // (`apply_cse_within_branches`, `pre_extract_from_valdefs`)
            // can extract a chain root (e.g. `INPUTS(1)`) into outer items[]
            // via `flatten_nested_blocks` while leaving other outer items'
            // inline references unrewritten. See doc on
            // `rewrite_byindex_globalvars_chain` for the rewrite shape set.
            let flattened = rewrite_byindex_globalvars_chain(flattened);
            trace_slot_shift("08-rewrite_byindex_globalvars_chain", &flattened);

            // Capture outer items[] IDs pre-disambig in items-order — paired with
            // post-disambig IDs (items[] order is preserved by disambig) gives us
            // the per-instance rename so we can re-key user_val_source_pos.
            let pre_disambig_outer_ids: Vec<u32> = if let Expr::BlockValue(s) = &flattened {
                s.expr
                    .items
                    .iter()
                    .filter_map(|i| {
                        if let Expr::ValDef(vd) = i {
                            Some(vd.expr.id.0)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // S60: disambiguate FIRST so outer/inner ValDef ids don't collide.
            // Both `dfs_reassign_val_ids` and `reorder_valdefs.emit_deps`
            // build outer val_rhs/val_map from items[].id and then walk the
            // outer body for ValUses. Without prior disambiguation, an inner
            // ValUse(K) where K coincidentally equals an outer ValDef id is
            // wrongly attributed to the outer ValDef — polluting body-walk
            // encounter order and (for OpenOrderToken) shifting `_tokenId`
            // from items[3] to items[8]. Disambig is order-independent on
            // tree shape; it only renames ids to be globally unique.
            let disambiguated = disambiguate_val_ids(flattened);
            trace_slot_shift("09-disambiguate_val_ids", &disambiguated);

            let post_disambig_outer_ids: Vec<u32> = if let Expr::BlockValue(s) = &disambiguated {
                s.expr
                    .items
                    .iter()
                    .filter_map(|i| {
                        if let Expr::ValDef(vd) = i {
                            Some(vd.expr.id.0)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // Re-key user_val_source_pos by post-disambig IDs. Items[] positions
            // are preserved by disambig (it only renames ValDef.id), so zipping
            // by index gives the correct per-instance pre→post mapping.
            let mut source_positions_post_disambig: HashMap<u32, usize> = HashMap::new();
            if pre_disambig_outer_ids.len() == post_disambig_outer_ids.len() {
                for (pre_id, post_id) in pre_disambig_outer_ids
                    .iter()
                    .zip(post_disambig_outer_ids.iter())
                {
                    if let Some(pos) = user_val_source_pos.get(pre_id) {
                        source_positions_post_disambig.insert(*post_id, *pos);
                    }
                }
            }

            let reassigned = dfs_reassign_val_ids(disambiguated, &source_positions_post_disambig);
            trace_slot_shift("10-dfs_reassign_val_ids", &reassigned);
            let reordered = reorder_valdefs(reassigned);
            trace_slot_shift("11-reorder_valdefs", &reordered);
            let final_expr = sequential_renumber(reordered);
            trace_slot_shift("12-sequential_renumber", &final_expr);
            final_expr
        })
        .expect("failed to spawn CSE thread")
        .join()
        .expect("CSE thread panicked")
}

/// Strip all source spans from an expression tree, replacing with empty spans.
/// This ensures CSE hash-consing treats structurally identical nodes as equal
/// regardless of their source position.
fn strip_source_spans(expr: Expr) -> Expr {
    use ergotree_ir::source_span::SourceSpan;
    let empty = SourceSpan::empty();
    // First recurse into children
    let expr = map_children(expr, strip_source_spans);
    // Then clear the span on this node
    match expr {
        Expr::BinOp(mut s) => {
            s.source_span = empty;
            Expr::BinOp(s)
        }
        Expr::PropertyCall(mut s) => {
            s.source_span = empty;
            Expr::PropertyCall(s)
        }
        Expr::MethodCall(mut s) => {
            s.source_span = empty;
            Expr::MethodCall(s)
        }
        Expr::ByIndex(mut s) => {
            s.source_span = empty;
            Expr::ByIndex(s)
        }
        Expr::SelectField(mut s) => {
            s.source_span = empty;
            Expr::SelectField(s)
        }
        Expr::ExtractRegisterAs(mut s) => {
            s.source_span = empty;
            Expr::ExtractRegisterAs(s)
        }
        Expr::OptionGet(mut s) => {
            s.source_span = empty;
            Expr::OptionGet(s)
        }
        Expr::OptionIsDefined(mut s) => {
            s.source_span = empty;
            Expr::OptionIsDefined(s)
        }
        Expr::OptionGetOrElse(mut s) => {
            s.source_span = empty;
            Expr::OptionGetOrElse(s)
        }
        Expr::ValDef(mut s) => {
            s.source_span = empty;
            Expr::ValDef(s)
        }
        Expr::BlockValue(mut s) => {
            s.source_span = empty;
            Expr::BlockValue(s)
        }
        Expr::Filter(mut s) => {
            s.source_span = empty;
            Expr::Filter(s)
        }
        Expr::Exists(mut s) => {
            s.source_span = empty;
            Expr::Exists(s)
        }
        Expr::ForAll(mut s) => {
            s.source_span = empty;
            Expr::ForAll(s)
        }
        Expr::Map(mut s) => {
            s.source_span = empty;
            Expr::Map(s)
        }
        Expr::Fold(mut s) => {
            s.source_span = empty;
            Expr::Fold(s)
        }
        Expr::LogicalNot(mut s) => {
            s.source_span = empty;
            Expr::LogicalNot(s)
        }
        Expr::Negation(mut s) => {
            s.source_span = empty;
            Expr::Negation(s)
        }
        Expr::GetVar(mut s) => {
            s.source_span = empty;
            Expr::GetVar(s)
        }
        Expr::Slice(mut s) => {
            s.source_span = empty;
            Expr::Slice(s)
        }
        Expr::Append(mut s) => {
            s.source_span = empty;
            Expr::Append(s)
        }
        Expr::TreeLookup(mut s) => {
            s.source_span = empty;
            Expr::TreeLookup(s)
        }
        Expr::And(mut s) => {
            s.source_span = empty;
            Expr::And(s)
        }
        Expr::Or(mut s) => {
            s.source_span = empty;
            Expr::Or(s)
        }
        other => other, // non-Spanned: Const, GlobalVars, ValUse, etc.
    }
}

/// True if `rhs` is a "trivial alias" RHS — an expression whose lowering is so
/// small that inlining it at every use site costs at most ~the same as keeping
/// the ValDef + ValUses, AND whose value is structurally re-computable at any
/// scope (no transitive dependence on a local ValUse). Currently: bare
/// `ByIndex(GlobalVars, Const)` with no default — the canonical
/// `INPUTS(N)` / `OUTPUTS(N)` alias shape.
///
/// HIR's `is_inline_always_rhs` already inlines bare `Literal` / `Context` /
/// `GlobalVars` (excluding `GroupGenerator`), so by MIR pre-CSE only the
/// ByIndex variant survives.
fn is_alias_rhs(rhs: &Expr) -> bool {
    if let Expr::ByIndex(s) = rhs {
        matches!(&*s.expr.input, Expr::GlobalVars(_))
            && matches!(&*s.expr.index, Expr::Const(_))
            && s.expr.default.is_none()
    } else {
        false
    }
}

/// Inline ValDefs whose RHS satisfies `is_alias_rhs` so structurally-
/// identical alias RHSes (one direct in an If's condition, one via a
/// branch-local ValDef) collapse during downstream CSE candidate counting
/// and the chain extracts at the right LCA. Two scoping rules apply,
/// each gated by a separate cross-scope evidence counter (see the call
/// site for details and the `should_inline` decision in
/// `inline_alias_vals_walk`).
///
/// Naively inlining every alias would either over-extract on Rust CSE's
/// re-pass (governance-reserve / oracle_refresh / chaincash_reserve
/// regressions where NODE preserves the source-author val) or hash-cons
/// across mutually-exclusive If arms in a way NODE doesn't (Lilium
/// SaleLP). The split eager / in-branch occurrence counts gate against
/// both classes.
///
/// Sig-15 paideia_stake_state S2 partial close.
fn inline_alias_vals(mut expr: Expr) -> Expr {
    // Pre-scan: count EAGER (outside any If branch) AND IN-BRANCH
    // occurrences of every is_alias_rhs sub-expression separately.
    //
    // Inlining decisions per ValDef location:
    //   - Inside an If branch: inline if there's an EAGER occurrence
    //     elsewhere (eager_count >= 1). Eager occurrences indicate the
    //     alias is also visible at a surrounding scope where outer CSE
    //     could extract it — inlining unblocks that LCA-scope extraction.
    //   - Outside any If branch (outer scope): inline if there's an
    //     IN-BRANCH occurrence elsewhere (branch_count >= 1). The outer
    //     val isn't itself the rejection target, but its inlining lets
    //     branch-local references collapse with each other and with this
    //     outer val's uses to a single root-bound CSE extraction matching
    //     NODE (paideia `val newStakeStateBox = OUTPUTS(0)` outer plus
    //     line 326/443 deep references).
    //
    // Sibling-If-branch occurrences alone DON'T trigger inlining: NODE
    // sees mutually-exclusive ThunkDefs as separate sym scopes (Lilium
    // SaleLP `OUTPUTS(0/1)` regression). The outer-or-eager counterpart
    // is the cross-scope evidence we need.
    let mut eager_counts: Vec<(Expr, usize)> = Vec::new();
    let mut branch_counts: Vec<(Expr, usize)> = Vec::new();
    count_alias_split_occurrences(&expr, false, &mut eager_counts, &mut branch_counts);
    inline_alias_vals_walk(&mut expr, false, &eager_counts, &branch_counts);
    expr
}

/// Walk `expr`, tracking whether the current position is inside any If
/// branch. For every `is_alias_rhs` sub-expression encountered, record it
/// in `eager_counts` (if NOT inside an If branch) or `branch_counts` (if
/// inside one). Stored as Vecs because `Expr` doesn't implement `Hash`.
fn count_alias_split_occurrences(
    expr: &Expr,
    in_if_branch: bool,
    eager_counts: &mut Vec<(Expr, usize)>,
    branch_counts: &mut Vec<(Expr, usize)>,
) {
    if is_alias_rhs(expr) {
        let target = if in_if_branch {
            &mut *branch_counts
        } else {
            &mut *eager_counts
        };
        if let Some(entry) = target.iter_mut().find(|(e, _)| e == expr) {
            entry.1 += 1;
        } else {
            target.push((expr.clone(), 1));
        }
    }
    if let Expr::If(if_op) = expr {
        count_alias_split_occurrences(
            &if_op.condition,
            in_if_branch,
            eager_counts,
            branch_counts,
        );
        count_alias_split_occurrences(&if_op.true_branch, true, eager_counts, branch_counts);
        count_alias_split_occurrences(&if_op.false_branch, true, eager_counts, branch_counts);
        return;
    }
    use ergotree_ir::traversable::Traversable;
    for child in expr.children() {
        count_alias_split_occurrences(child, in_if_branch, eager_counts, branch_counts);
    }
}

fn alias_count_for(rhs: &Expr, counts: &[(Expr, usize)]) -> usize {
    counts
        .iter()
        .find(|(e, _)| e == rhs)
        .map(|(_, c)| *c)
        .unwrap_or(0)
}

fn inline_alias_vals_walk(
    expr: &mut Expr,
    in_if_branch: bool,
    eager_counts: &[(Expr, usize)],
    branch_counts: &[(Expr, usize)],
) {
    use ergotree_ir::traversable::Traversable;
    match expr {
        Expr::If(if_op) => {
            // Cond evaluates eagerly at the surrounding scope; branches
            // enter the if-branch scope (Scala ThunkDef).
            inline_alias_vals_walk(
                &mut if_op.condition,
                in_if_branch,
                eager_counts,
                branch_counts,
            );
            inline_alias_vals_walk(&mut if_op.true_branch, true, eager_counts, branch_counts);
            inline_alias_vals_walk(&mut if_op.false_branch, true, eager_counts, branch_counts);
        }
        Expr::BlockValue(s) => {
            // Recurse children first (bottom-up).
            for item in s.expr.items.iter_mut() {
                inline_alias_vals_walk(item, in_if_branch, eager_counts, branch_counts);
            }
            inline_alias_vals_walk(&mut s.expr.result, in_if_branch, eager_counts, branch_counts);

            let mut inline_map: indexmap::IndexMap<u32, Expr> = indexmap::IndexMap::new();
            for item in s.expr.items.iter() {
                if let Expr::ValDef(vd) = item {
                    if !is_alias_rhs(&vd.expr.rhs) {
                        continue;
                    }
                    let should_inline = if in_if_branch {
                        // Inside an If branch: inline if there's a separate
                        // eager occurrence the outer LCA can extract toward.
                        alias_count_for(&vd.expr.rhs, eager_counts) >= 1
                    } else {
                        // Outer scope: inline only if there's an in-branch
                        // occurrence of the same RHS (so inlining lets the
                        // branch-local references collapse with this val's
                        // uses to a single root-bound CSE extraction matching
                        // NODE). Otherwise, leave the source-author val
                        // intact (governance-reserve / oracle_refresh case).
                        alias_count_for(&vd.expr.rhs, branch_counts) >= 1
                    };
                    if should_inline {
                        inline_map.insert(vd.expr.id.0, (*vd.expr.rhs).clone());
                    }
                }
            }

            if inline_map.is_empty() {
                return;
            }

            // Drop the inlined ValDefs from items.
            let kept: Vec<Expr> = std::mem::take(&mut s.expr.items)
                .into_iter()
                .filter(|i| {
                    if let Expr::ValDef(vd) = i {
                        !inline_map.contains_key(&vd.expr.id.0)
                    } else {
                        true
                    }
                })
                .collect();
            s.expr.items = kept;

            // Substitute every ValUse(id) site within remaining items + result.
            for (val_id, rhs) in &inline_map {
                let val_use = Expr::ValUse(ValUse {
                    val_id: ValId(*val_id),
                    tpe: rhs.tpe(),
                });
                let new_items: Vec<Expr> = s.expr
                    .items
                    .iter()
                    .map(|i| replace_all(i, &val_use, rhs))
                    .collect();
                s.expr.items = new_items;
                let new_result = replace_all(&s.expr.result, &val_use, rhs);
                s.expr.result = new_result.into();
            }
        }
        // Everything else: structural recurse via Traversable so we never
        // miss an Expr variant (the same approach used by `disambig_walk`).
        other => {
            for child in other.children_mut() {
                inline_alias_vals_walk(child, in_if_branch, eager_counts, branch_counts);
            }
        }
    }
}


/// Re-assign val IDs in DFS traversal order from the result expression.
///
/// In Scala's graph IR, symbol IDs are assigned during graph construction
/// which follows DFS order from the result. When `reorder_valdefs` sorts
/// If-branch deps by val ID, the IDs need to reflect this DFS order rather
/// than the source/binder order. This pass walks each BlockValue's result
/// in DFS order, recording the encounter order of ValUse references, and
/// reassigns IDs accordingly.
fn dfs_reassign_val_ids(expr: Expr, source_positions: &HashMap<u32, usize>) -> Expr {
    // Only process the top-level BlockValue. Inner blocks are handled
    // by reorder_valdefs which recurses independently.
    match expr {
        Expr::BlockValue(s) => {
            if s.expr.items.is_empty() {
                return Expr::BlockValue(s);
            }
            // S62 / direction #2: schedule outer-block symbols in
            // SOURCE-CREATION order, mirroring Scala's TreeBuilding.
            //
            // Pass 1a: visit surviving user-declared ValDefs in source order
            //   (items[] index at HIR→MIR time, captured pre-strip in apply_cse
            //    and re-keyed through disambig). For each user val, walk its
            //   RHS deps-first so dep IDs land before the dependent.
            // Pass 1b: walk the result expression for any vals not yet covered
            //   (e.g. CSE-extracted vals reachable only from result, not from
            //    any user val's RHS).
            //
            // Why source order matters: in Scala's graph IR, symbol IDs reflect
            // the order their RHS was first lowered during source processing.
            // Source-earlier vals get smaller IDs; the later sort-by-ID in
            // emit_deps's If arm then preserves that order across both branches.
            // Result-DFS-encounter order (the previous strategy) instead orders
            // by which val is mentioned first in the result expr — fine for
            // single-branch contracts, wrong for ifs whose branches both
            // reference the same vals (Phoenix: validBankRecreation appears
            // first in true_branch, displacing earlier `price` in the schedule).
            let mut ordered_ids: Vec<u32> = Vec::new();
            let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
            {
                // Build val_id -> ValDef RHS map (borrows from s.expr.items).
                // Scoped so it's dropped before we move `s` into block_expr.
                let mut val_rhs: HashMap<u32, &Expr> = HashMap::new();
                for item in &s.expr.items {
                    if let Expr::ValDef(vd) = item {
                        val_rhs.insert(vd.expr.id.0, &vd.expr.rhs);
                    }
                }
                if val_rhs.is_empty() {
                    return Expr::BlockValue(s);
                }

                // Pass 1a: only seed COMPOUND user vals (RHS that references at
                // least one other outer val) in source order. Trivial vals
                // (RHS = leaf register-read / const / lone field-extract) are
                // left for the result-walk in Pass 1b — Scala places them at
                // their first-use site in the body, not at their declaration
                // site.  Phoenix FULL: seeding R-register reads here pushes
                // them to the front of the schedule, but Scala emits them
                // alongside their consumers (R5,R4 with `price`; R6,R7,R8 with
                // `validBankRecreation`).
                //
                // Compound = RHS contains ValUse(id) for some id in val_rhs.
                let rhs_has_user_val_use = |rhs: &Expr, val_rhs: &HashMap<u32, &Expr>| -> bool {
                    let mut found = false;
                    let mut work: Vec<&Expr> = vec![rhs];
                    while let Some(e) = work.pop() {
                        if let Expr::ValUse(vu) = e {
                            if val_rhs.contains_key(&vu.val_id.0) {
                                found = true;
                                break;
                            }
                        }
                        for child in body_walk_children(e) {
                            work.push(child);
                        }
                    }
                    found
                };
                let mut user_vals_in_source_order: Vec<u32> = source_positions
                    .keys()
                    .filter(|id| {
                        val_rhs
                            .get(id)
                            .map(|rhs| rhs_has_user_val_use(rhs, &val_rhs))
                            .unwrap_or(false)
                    })
                    .copied()
                    .collect();
                user_vals_in_source_order.sort_by_key(|id| source_positions[id]);

                // S65 — Pass 1a applicability gate: skip Pass 1a iff the outer
                // result expression is NOT an If (after stripping
                // sigmaProp/BoolToSigmaProp wrappers).
                //
                // Background: Pass 1a seeds compound user vals in source-order
                // with deps-first walks, giving each user val a low ID
                // reflecting its source declaration. This is correct when the
                // result is `if (cond) <true> else <false>`: reorder_valdefs's
                // cond walk emits cond's ValDef chain first, then the
                // If-branch sort-by-ID emits the remaining vals in NODE's
                // schedule order (Phoenix HodlERG Bank: validBankRecreation's
                // And needs the highest ID among branch deps so it emits last;
                // src_pos seeding gives it that since And's source position is
                // last among compound candidates).
                //
                // But when the result is a logical AND chain wrapping a nested
                // If (e.g. `sigmaProp(a && b && validAction && c)` in
                // Spectrum's pool fixtures), src_pos seeding gives nested-If-
                // branch-only vals (Spectrum's reservesY0 SelectField,
                // deltaReservesY BinOp) low IDs that put them BEFORE the
                // CSE-extracted Upcast wrappers in the inner If's sort. NODE
                // wants them ordered by hash-cons creation (≈first-use in the
                // result-walk), not by source declaration. Skipping Pass 1a
                // lets Pass 1b's plain DFS over the result assign IDs in
                // result-walk encounter order, which matches Scala's emission.
                //
                // The discriminator is purely the result-expression shape.
                // Outer-If contracts (BondContract*, Phoenix, OpenOrder, ...)
                // need Pass 1a; outer-AND contracts (spectrum n2t/t2t pools,
                // and any other AND-chain-wrapped contracts) skip it.
                let outer_is_if = {
                    fn strip_to_if_test(e: &Expr) -> &Expr {
                        match e {
                            Expr::BoolToSigmaProp(s) => strip_to_if_test(&s.input),
                            _ => e,
                        }
                    }
                    matches!(strip_to_if_test(&s.expr.result), Expr::If(_))
                };
                if outer_is_if {
                    // S66: body-schedule simulation. Walk the result with
                    // non-leaf children processed before leaves, mirroring
                    // Scala's `AstGraph.freeVars` collection semantics. This
                    // replaces the earlier source-order seeding (which was
                    // correct for Phoenix HodlERG Bank but wrong for ergoraffle
                    // — see 06b §"Why even the structurally-correct tree is
                    // +37B off NODE"). Body-schedule walk gives Phoenix the
                    // same ordering source-order did (validBankRecreation last
                    // because it's a leaf-VU sibling of the non-leaf
                    // validBankDeposit chain), and gives ergoraffle NODE's
                    // ordering (deadline → OUTPUTS → addresses → outR4 →
                    // ByIndex chain → totalSold/outTotalSold → Const →
                    // totalRaised/BI).
                    let _ = user_vals_in_source_order; // unused under S66
                    body_schedule_walk_collect(
                        &s.expr.result,
                        &val_rhs,
                        &mut ordered_ids,
                        &mut visited,
                    );
                }

                // Pass 1b: walk the result for any vals the user-val schedule
                // didn't cover (e.g. an If's cond/branch that references a
                // CSE-extracted val whose source position is unknown).
                dfs_collect_val_order(&s.expr.result, &val_rhs, &mut ordered_ids, &mut visited);
            }

            // Pass 2: sweep the entire tree (incl. inner BlockValue items) for
            // any ValDef IDs not yet in ordered_ids; append in tree-encounter
            // order. Pass 1 deliberately skips inner items so they don't leak
            // ValUses into outer-scope ordering, but rewrite_ids still needs a
            // mapping for every ValDef id in the tree — otherwise an unmapped
            // inner id can collide with a freshly-assigned outer id.
            let block_expr = Expr::BlockValue(s);
            collect_all_valdef_ids_in_order(&block_expr, &mut ordered_ids, &mut visited);

            // Build old_id -> new_id map (1-indexed to match Scala's curId scheme)
            let mut id_map: HashMap<u32, u32> = HashMap::new();
            for (new_idx, old_id) in ordered_ids.iter().enumerate() {
                id_map.insert(*old_id, (new_idx + 1) as u32);
            }

            // Only rewrite if the mapping actually changes something
            if id_map.iter().all(|(old, new)| old == new) {
                return block_expr;
            }

            rewrite_ids(block_expr, &id_map)
        }
        other => other,
    }
}

/// Children walk that respects ThunkDef boundaries: for inner BlockValues,
/// only descend into the result (skip items). This mirrors Scala's
/// processAstGraph, which scopes inner ThunkDef contents to their own
/// schedule rather than leaking them into the outer scope's ordering.
fn body_walk_children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::BlockValue(bv) => vec![&bv.expr.result],
        _ => direct_children(expr),
    }
}

/// DFS walk an expression, collecting val IDs in encounter order.
/// Uses an iterative approach to avoid stack overflow on deeply nested trees.
fn dfs_collect_val_order(
    root: &Expr,
    val_rhs: &HashMap<u32, &Expr>,
    ordered: &mut Vec<u32>,
    visited: &mut std::collections::HashSet<u32>,
) {
    // Use an explicit stack instead of recursion
    let mut work: Vec<&Expr> = vec![root];
    while let Some(expr) = work.pop() {
        match expr {
            Expr::ValUse(vu) => {
                let id = vu.val_id.0;
                // S60: only count ValUses that refer to OUTER ValDefs (id is
                // in val_rhs). Inner-scope ValUses with ids that happen to
                // coincide with outer ValDef ids (pre-disambig collisions are
                // common, since disambiguation runs *after* dfs_reassign)
                // would otherwise pollute the outer body-walk encounter
                // order — Pass 2's `collect_all_valdef_ids_in_order` already
                // sweeps inner ValDef ids for the id_map coverage invariant.
                // Without this filter, OpenOrderToken's `_tokenId` (an outer
                // top-level val referenced inside an inner BlockValue's
                // result) was getting displaced to ordered position 9 by
                // unrelated inner ValUses, which then placed its ValDef at
                // items[8] (last) and shifted its constant from pool[1] (the
                // node-correct position) to pool[7].
                if val_rhs.contains_key(&id) && !visited.contains(&id) {
                    visited.insert(id);
                    if let Some(rhs) = val_rhs.get(&id) {
                        // Process deps first (children-before-parent), then record id.
                        dfs_collect_val_order_inner(rhs, val_rhs, ordered, visited);
                    }
                    ordered.push(id);
                }
            }
            _ => {
                // Push children in REVERSE order so first child is processed first.
                // Use body_walk_children to skip inner BlockValue items — their
                // ValUses belong to the inner scope's schedule, not the outer's.
                let children = body_walk_children(expr);
                for child in children.into_iter().rev() {
                    work.push(child);
                }
            }
        }
    }
}

/// Body-schedule simulation walk: non-leaf children processed BEFORE leaves.
///
/// Mirrors Scala's `AstGraph.freeVars` semantics — body schedule is DFS
/// post-order over body-internal nodes; freeVars are collected by iterating
/// body schedule and recording each body sym's external (out-of-body) deps.
/// Non-leaf children are body syms that appear in body schedule BEFORE the
/// parent, so their external deps are recorded BEFORE the parent's external
/// (leaf-ValUse) deps. We approximate this by partitioning each node's
/// children into leaves (direct `ValUse`s) and non-leaves, and pushing
/// leaves onto the work stack first so non-leaves get popped (and their
/// externals recorded) ahead of the leaves.
///
/// Used only when the outer result is an `If`. AND-chain contracts (outer
/// `sigmaProp(allOf(...))` such as Spectrum n2t/t2t pools) need the simpler
/// left-to-right pre-order — this body-schedule walk over-reorders their
/// CSE-extracted vals.
///
/// Sig-15 #6 (ergoraffle) line 36 `outTotalSold == totalSold + currentSold`:
/// `BinOp(+)` (non-leaf) processed first → `totalSold` recorded before
/// `outTotalSold` (matches NODE's `totalSold` ID < `outTotalSold` ID).
fn body_schedule_walk_collect(
    root: &Expr,
    val_rhs: &HashMap<u32, &Expr>,
    ordered: &mut Vec<u32>,
    visited: &mut std::collections::HashSet<u32>,
) {
    let mut work: Vec<&Expr> = vec![root];
    while let Some(expr) = work.pop() {
        match expr {
            Expr::ValUse(vu) => {
                let id = vu.val_id.0;
                if val_rhs.contains_key(&id) && !visited.contains(&id) {
                    visited.insert(id);
                    if let Some(rhs) = val_rhs.get(&id) {
                        dfs_collect_val_order_inner(rhs, val_rhs, ordered, visited);
                    }
                    ordered.push(id);
                }
            }
            _ => {
                let children = body_walk_children(expr);
                let (leaves, nonleaves): (Vec<&Expr>, Vec<&Expr>) = children
                    .into_iter()
                    .partition(|c| matches!(c, Expr::ValUse(_)));
                // Push leaves first (deeper in stack — popped LATER); then
                // non-leaves (top of stack — popped FIRST).
                for child in leaves.into_iter().rev() {
                    work.push(child);
                }
                for child in nonleaves.into_iter().rev() {
                    work.push(child);
                }
            }
        }
    }
}

/// Recursive helper for processing a ValDef's RHS deps.
/// This can recurse through val chains but is bounded by the number of vals.
fn dfs_collect_val_order_inner(
    expr: &Expr,
    val_rhs: &HashMap<u32, &Expr>,
    ordered: &mut Vec<u32>,
    visited: &mut std::collections::HashSet<u32>,
) {
    // Use iterative DFS for expression tree traversal
    let mut work: Vec<&Expr> = vec![expr];
    while let Some(e) = work.pop() {
        match e {
            Expr::ValUse(vu) => {
                let id = vu.val_id.0;
                // S60 mirror of dfs_collect_val_order: skip ValUses whose id
                // doesn't belong to the outer scope's val_rhs map. See comment
                // in dfs_collect_val_order for rationale.
                if val_rhs.contains_key(&id) && !visited.contains(&id) {
                    visited.insert(id);
                    if let Some(rhs) = val_rhs.get(&id) {
                        dfs_collect_val_order_inner(rhs, val_rhs, ordered, visited);
                    }
                    ordered.push(id);
                }
            }
            _ => {
                let children = body_walk_children(e);
                for child in children.into_iter().rev() {
                    work.push(child);
                }
            }
        }
    }
}

/// Pass 2 sweep: walk the full tree (including inner BlockValue items) and
/// append every ValId not already in `ordered`. Collects ValDef.id,
/// FuncArg.idx, and ValUse.val_id so id_map is total over the whole tree.
///
/// Why all three: rewrite_ids leaves any unmapped id as-is. After Pass 1
/// renames root ValDefs to small new ids (1..k), an unmapped FuncArg whose
/// id happens to equal a fresh new id creates a phantom collision —
/// reorder_valdefs's emit_deps then treats a lambda-internal ValUse
/// (really a FuncArg ref) as a reference to the renamed outer ValDef and
/// recurses into its RHS, which contains the same FuncArg ValUse → cycle →
/// stack overflow. Mapping every id removes the collision class entirely.
fn collect_all_valdef_ids_in_order(
    root: &Expr,
    ordered: &mut Vec<u32>,
    visited: &mut std::collections::HashSet<u32>,
) {
    let mut work: Vec<&Expr> = vec![root];
    while let Some(expr) = work.pop() {
        match expr {
            Expr::ValDef(vd) => {
                let id = vd.expr.id.0;
                if !visited.contains(&id) {
                    visited.insert(id);
                    ordered.push(id);
                }
            }
            Expr::ValUse(vu) => {
                let id = vu.val_id.0;
                if !visited.contains(&id) {
                    visited.insert(id);
                    ordered.push(id);
                }
            }
            Expr::FuncValue(fv) => {
                for arg in fv.args() {
                    let id = arg.idx.0;
                    if !visited.contains(&id) {
                        visited.insert(id);
                        ordered.push(id);
                    }
                }
            }
            _ => {}
        }
        let children = direct_children(expr);
        for child in children.into_iter().rev() {
            work.push(child);
        }
    }
}

/// Reorder BlockValue items to match Scala's DFS dependency ordering.
///
/// The Scala compiler linearizes its graph in DFS order from the root, so
/// ValDefs appear in the order they're first needed by the result expression
/// (with dependencies emitted before dependents). Our CSE prepends all
/// extracted vals before user vals, which can produce a different order.
fn reorder_valdefs(expr: Expr) -> Expr {
    // Recurse into If branches to reorder inner blocks too
    let expr = map_children(expr, reorder_valdefs);

    match expr {
        Expr::BlockValue(s) => {
            if s.expr.items.is_empty() {
                return Expr::BlockValue(s);
            }
            // Build map: val_id -> ValDef
            let mut val_map: HashMap<u32, Expr> = HashMap::new();
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    val_map.insert(vd.expr.id.0, item.clone());
                }
            }

            let mut emitted: Vec<Expr> = Vec::new();
            let mut emitted_ids: HashSet<u32> = HashSet::new();

            // Walk the result expression DFS; when a ValUse is encountered,
            // recursively emit its ValDef (and that ValDef's deps) first.
            emit_deps(
                &s.expr.result,
                &val_map,
                &mut emitted,
                &mut emitted_ids,
                false,
                false,
            );

            // Drop ValDefs with zero uses (dead post-fold residue).
            // Scala's TreeBuilding emits no ValDef when `hasManyUsagesGlobal`
            // is false. `emit_deps` is incomplete (skips Atleast/SizeOf/Coll/
            // SigmaPropBytes/...), so use `count_val_uses_in` (walks
            // `direct_children`) for the live/dead decision.
            let mut use_counts: HashMap<u32, usize> = HashMap::new();
            count_val_uses_in(&s.expr.result, &mut use_counts);
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    let id = vd.expr.id.0;
                    if !emitted_ids.contains(&id) {
                        // Include transitive uses via other already-emitted
                        // ValDefs' RHSes (those will be kept regardless).
                        for em in &emitted {
                            if let Expr::ValDef(em_vd) = em {
                                count_val_uses_in(&em_vd.expr.rhs, &mut use_counts);
                            }
                        }
                        let used = use_counts.get(&id).copied().unwrap_or(0) > 0;
                        if used {
                            emitted_ids.insert(id);
                            emitted.push(item.clone());
                        }
                        // else: drop (dead ValDef post-fold)
                    }
                }
            }

            if emitted.is_empty() {
                return *s.expr.result;
            }

            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: emitted,
                    result: s.expr.result,
                },
            })
        }
        other => other,
    }
}

/// Inline ValDefs that are used exactly once in their enclosing BlockValue.
/// After CSE extraction, some user-defined vals (e.g., `val reserveIn = SELF.value`)
/// may become single-use because their only reference is inside a CSE-extracted
/// val (e.g., `Upcast(ValUse(reserveIn), BigInt)`). Inlining these single-use
/// vals produces the combined form that Scala's graph IR would generate
/// (e.g., `Upcast(ExtractAmount(Self), BigInt)` as a single val).
/// Pre-inline CSE: for each BlockValue B, extract sub-expressions that
/// appear in an If's cond AND in one of its branches within B's tree.
///
/// Motivation: source-level `val v = if(c) t else f` declarations get
/// the `if` expression inlined by HIR→MIR when v has only one use. After
/// inlining, the If lives inside a lazy thunk position (&&/|| right arm
/// or outer If branch), so our scope-restricted CSE can't see its cond
/// sub-expressions at B's scope. But in Scala's graph IR, the If sym
/// (and its cond sub-expressions) were created at B's scope during the
/// val's RHS evaluation — they're in B's schedule regardless of where
/// the val ends up being inlined. `hasManyUsagesGlobal` then extracts
/// them as ValDefs at B.
///
/// This pass approximates Scala's behavior: for every If node appearing
/// anywhere in B's tree (even inside sub-thunks of B), find sub-expressions
/// that appear in If.cond AND also elsewhere in B's tree (≥2 occurrences
/// total). Extract those as new ValDefs at B's scope.
fn pre_extract_from_valdefs(expr: Expr, next_id: &mut u32) -> Expr {
    match expr {
        Expr::BlockValue(s) => {
            let inner = s.expr;
            // Collect candidates appearing in any If.cond inside this BlockValue.
            let (new_defs, rewritten) =
                extract_if_cond_shared(inner.items.clone(), (*inner.result).clone(), next_id);

            // Topologically order: a ValDef Y that references ValUse(id=X)
            // must come AFTER the ValDef defining X. The extracted new_defs
            // may depend on existing items (e.g., SigUSDV1: val_8's RHS
            // references ValUse(3), an existing item). Conversely,
            // existing items may depend on new_defs via replace_all.
            let combined = {
                let mut v = new_defs;
                v.extend(rewritten.0);
                v
            };
            let final_items = topo_order_valdefs(combined);
            let result_expr = rewritten.1;

            // Recurse into nested BlockValues.
            let processed_items: Vec<Expr> = final_items
                .into_iter()
                .map(|i| pre_extract_from_valdefs(i, next_id))
                .collect();
            let processed_result = pre_extract_from_valdefs(result_expr, next_id);
            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: processed_items,
                    result: processed_result.into(),
                },
            })
        }
        other => map_children_with_id_mut(other, next_id, pre_extract_from_valdefs),
    }
}

/// At a BlockValue scope, find sub-expressions X that:
///   1. Appear inside at least one If.cond somewhere in the scope's tree
///      (possibly nested inside sub-thunks), AND
///   2. Have ≥ 2 total occurrences in the scope's tree, AND
///   3. Are extractable (non-leaf, hash-consable), AND
///   4. Don't depend on ValDefs defined inside the scope (other than the
///      pre-existing items, which are fine).
///
/// Returns (new_valdefs_to_prepend, (rewritten_items, rewritten_result)).
fn extract_if_cond_shared(
    items: Vec<Expr>,
    result: Expr,
    next_id: &mut u32,
) -> (Vec<Expr>, (Vec<Expr>, Expr)) {
    // Build a combined view of the scope's tree for counting:
    // wrap items+result in a synthetic BlockValue so count_occurrences sees
    // everything in one shot.
    let synthetic = Expr::BlockValue(Spanned {
        source_span: SourceSpan::empty(),
        expr: BlockValue {
            items: items.clone(),
            result: result.clone().into(),
        },
    });

    // Collect sub-expressions that appear inside any If.cond within the
    // scope's tree (ordered DFS, dedup).
    let mut if_cond_subs: Vec<Expr> = Vec::new();
    collect_if_cond_subexprs(&synthetic, &mut if_cond_subs);

    let mut unique: Vec<Expr> = Vec::new();
    for sub in if_cond_subs {
        if !unique.iter().any(|u| u == &sub) {
            unique.push(sub);
        }
    }

    // Don't hoist things depending on vals defined inside nested BlockValues.
    // But items' ValDefs are fine to depend on — they're at this scope.
    let mut nested_locals = std::collections::HashSet::new();
    collect_nested_block_val_ids(&result, &mut nested_locals);
    for item in &items {
        collect_nested_block_val_ids(item, &mut nested_locals);
    }

    // Local ValDef IDs at this scope. A candidate that references one of
    // these is "anchored" to this scope in Scala's graph IR — its sym lives
    // here (via createDefinition's capture-set placement), so its global
    // usage count is what matters. A candidate that references NO local
    // item could live in any scope; in Scala, sibling-Thunk uses each
    // build their own sym, so the outer-scope sym would have only the
    // direct outer-scope uses. Mirror this by switching count modes.
    let scope_local_ids: std::collections::HashSet<u32> = items
        .iter()
        .filter_map(|i| {
            if let Expr::ValDef(vd) = i {
                Some(vd.expr.id.0)
            } else {
                None
            }
        })
        .collect();

    let mut extracted_defs: Vec<Expr> = Vec::new();
    let mut current_items = items;
    let mut current_result = result;
    for sub in unique {
        if !is_extractable(&sub) {
            continue;
        }
        if !is_graph_shared(&sub) {
            continue;
        }
        if references_locally_defined(&sub, &nested_locals) {
            continue;
        }
        // Bidirectional Thunk-aware count:
        //   - candidate references a local item → count globally (rescue:
        //     ProxyBorrow's PropertyCall(ValUse(N), tokens), `if-else paths`)
        //   - else → count scope-restricted, stopping at inner-Thunk
        //     boundaries (closes SigUSDV1 (B) OptionGet(extReg(R5))
        //     and OpenOrders constant-collapse over-extractions)
        let rescue =
            !scope_local_ids.is_empty() && expr_references_any_local(&sub, &scope_local_ids);
        // S54: always count globally within this scope's tree. The earlier
        // scope-restricted counter (which stopped at And/Or right arms and
        // If branches) under-counted candidates whose uses straddled a
        // ThunkDef boundary — e.g. `Const(BigInt(0))` used in two sibling
        // If conds inside the orderIsClosed-true_branch BlockValue. The
        // `deeper_block_with_ge_two_occurrences` defer-to-inner-scope guard
        // (now also active in non-rescue mode) prevents over-extraction.
        let _ = rescue;
        let counter: fn(&Expr, &Expr) -> usize = count_occurrences;
        let mut cnt = counter(&current_result, &sub);
        for item in &current_items {
            cnt += counter(item, &sub);
        }
        if cnt < 2 {
            continue;
        }
        // Defer-to-inner-scope guard (S43): when rescue=true (candidate
        // references a local item so we used the global count), suppress
        // extraction at this scope if there's a deeper BlockValue strictly
        // nested in this scope's tree that already contains ≥2 occurrences
        // of the candidate. That inner BlockValue is its own scope where
        // `pre_extract_from_valdefs` will recurse and run another
        // `extract_if_cond_shared` pass — extracting here would hijack it
        // and produce a ValDef at the wrong scope.
        //
        // Mirrors Scala's `bodyIds` placement: when all global uses of a
        // candidate sit inside an inner ThunkDef's BlockValue body, the
        // sym's `bodyIds` is that inner block's, not this scope's.
        //
        // Closes OpenOrders 2B trailing ByIndex extract (candidate's 2
        // occurrences are inside the outer If's true_branch BlockValue).
        // Preserves ProxyBorrow's PropertyCall(VU(N), tokens) — that
        // candidate's occurrences sit inside the right arm of `&&` which
        // has no BlockValue body, so no deeper scope exists.
        // S54: defer extraction whenever the candidate's global occurrences
        // are concentrated inside a single deeper BlockValue (regardless of
        // whether the candidate references a local item). Previously
        // gated on `rescue`; that left bare-Const candidates (no ValUse,
        // never local-rescued) extracting at outer scope, which bloated
        // outer items[] and shifted the constant pool order vs node.
        if deeper_block_with_ge_two_occurrences(&current_items, &current_result, &sub) {
            continue;
        }
        // BISECT-A4 (S12): reject BinOp candidates whose entire subtree
        // contains no `ValUse` — pure-context-property comparisons such
        // as `INPUTS.size == 1` or `INPUTS.size >= 3`. Scala's TreeBuilding
        // gate (`hasManyUsagesGlobal && !IsContextProperty && !IsInternalDef
        // && !IsConstantDef`) leaves these inline in IfNode.cond rather
        // than promoting to a graph Sym (empirically: NODE pool for sigmao
        // keeps Const(1:SInt)=12 / Const(3:SInt)=4 whereas Rust collapses
        // them to 11/3 via this pass). Discriminator empirically verified
        // by `CSE_TRACE_PRE_EXTRACT` (S12 archive): all MATCH-fixture BinOp
        // extractions (chaincash / oracle / sigmausd / gluon / ergoraffle)
        // carry a ValUse child; only sigmao + paideia BinOp extractions
        // here are no-ValUse (and both are currently USED NODE).
        if let Expr::BinOp(ref b) = sub {
            use ergotree_ir::mir::bin_op::{BinOpKind, RelationOp};
            if matches!(
                b.expr.kind,
                BinOpKind::Relation(RelationOp::Eq | RelationOp::NEq)
            ) && !contains_val_use(&sub)
            {
                continue;
            }
        }
        if std::env::var("CSE_TRACE_PRE_EXTRACT").is_ok() {
            let c1 = count_const_sint(&sub, 1);
            let c3 = count_const_sint(&sub, 3);
            let kind = pre_extract_variant_name(&sub);
            // Split count by If.cond vs outside-If.cond positions over the
            // current synthetic scope (items + result).
            let mut in_cond = 0usize;
            let mut out_cond = 0usize;
            let r = count_in_if_cond(&current_result, &sub, false);
            in_cond += r.0;
            out_cond += r.1;
            for it in &current_items {
                let r = count_in_if_cond(it, &sub, false);
                in_cond += r.0;
                out_cond += r.1;
            }
            let dbg = format!("{:?}", sub);
            let dbg_trim: String = dbg.chars().take(160).collect();
            eprintln!(
                "[PRE_EXTRACT] id={} kind={} cnt={} cond={} out={} c1={} c3={} rescue={} shape={}",
                *next_id, kind, cnt, in_cond, out_cond, c1, c3, rescue, dbg_trim
            );
        }
        let id = *next_id;
        *next_id += 1;
        let val_use = Expr::ValUse(ValUse {
            val_id: ValId(id),
            tpe: expr_type(&sub),
        });
        current_result = replace_all(&current_result, &sub, &val_use);
        current_items = current_items
            .into_iter()
            .map(|i| replace_all(&i, &sub, &val_use))
            .collect();
        extracted_defs.push(Expr::ValDef(Spanned {
            source_span: SourceSpan::empty(),
            expr: ValDef {
                id: ValId(id),
                rhs: sub.into(),
            },
        }));
    }
    (extracted_defs, (current_items, current_result))
}

/// Collect sub-expressions that appear inside any If.cond in the tree,
/// without descending into inner BlockValues (those have their own scope
/// and will be handled by the recursive pre_extract_from_valdefs call).
fn collect_if_cond_subexprs(expr: &Expr, out: &mut Vec<Expr>) {
    match expr {
        Expr::BlockValue(s) => {
            // Don't descend into nested blocks — they're their own scope.
            // But DO process ValDefs' RHSs since those are in this scope.
            for item in &s.expr.items {
                collect_if_cond_subexprs(item, out);
            }
            collect_if_cond_subexprs(&s.expr.result, out);
        }
        Expr::ValDef(s) => {
            collect_if_cond_subexprs(&s.expr.rhs, out);
        }
        Expr::If(if_op) => {
            // This If's cond contributes all its non-leaf sub-expressions.
            collect_subexprs(&if_op.condition, out);
            // And recurse into branches for any nested Ifs.
            collect_if_cond_subexprs(&if_op.true_branch, out);
            collect_if_cond_subexprs(&if_op.false_branch, out);
        }
        other => {
            for child in direct_children(other) {
                collect_if_cond_subexprs(child, out);
            }
        }
    }
}

/// Collect ValDef IDs defined inside nested BlockValues (not the top-level
/// items, which belong to the current scope).
fn collect_nested_block_val_ids(expr: &Expr, ids: &mut std::collections::HashSet<u32>) {
    match expr {
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    ids.insert(vd.expr.id.0);
                }
                collect_nested_block_val_ids(item, ids);
            }
            collect_nested_block_val_ids(&s.expr.result, ids);
        }
        other => {
            for child in direct_children(other) {
                collect_nested_block_val_ids(child, ids);
            }
        }
    }
}

/// Topologically order ValDef items so each ValDef appears AFTER all its
/// ValUse dependencies within the list. Non-ValDef items keep their
/// position relative to each other. The algorithm: iteratively emit any
/// item whose unresolved deps are empty; if none, break cycles by emitting
/// the first remaining item (should not happen with well-formed SSA).
fn topo_order_valdefs(items: Vec<Expr>) -> Vec<Expr> {
    // Collect all ValDef IDs present in this list.
    let mut defined_here: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for item in &items {
        if let Expr::ValDef(vd) = item {
            defined_here.insert(vd.expr.id.0);
        }
    }

    // Compute intra-list deps for each item.
    let mut deps: Vec<std::collections::HashSet<u32>> = Vec::with_capacity(items.len());
    for item in &items {
        let mut used = Vec::new();
        match item {
            Expr::ValDef(vd) => collect_all_val_uses(&vd.expr.rhs, &mut used),
            other => collect_all_val_uses(other, &mut used),
        }
        let d: std::collections::HashSet<u32> = used
            .into_iter()
            .filter(|id| defined_here.contains(id))
            .collect();
        // A ValDef does not "depend on itself".
        let d = if let Expr::ValDef(vd) = item {
            let mut d = d;
            d.remove(&vd.expr.id.0);
            d
        } else {
            d
        };
        deps.push(d);
    }

    let n = items.len();
    let mut emitted: Vec<bool> = vec![false; n];
    let mut emitted_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut out: Vec<Expr> = Vec::with_capacity(n);
    // Keep iterating; Kahn-style.
    loop {
        let mut progressed = false;
        for i in 0..n {
            if emitted[i] {
                continue;
            }
            // Dependencies satisfied if every dep ID is already emitted.
            if deps[i].iter().all(|id| emitted_ids.contains(id)) {
                if let Expr::ValDef(vd) = &items[i] {
                    emitted_ids.insert(vd.expr.id.0);
                }
                out.push(items[i].clone());
                emitted[i] = true;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    // Append any stragglers (cycle — shouldn't happen) to keep items preserved.
    for i in 0..n {
        if !emitted[i] {
            out.push(items[i].clone());
        }
    }
    out
}

fn references_locally_defined(
    expr: &Expr,
    locally_defined: &std::collections::HashSet<u32>,
) -> bool {
    if locally_defined.is_empty() {
        return false;
    }
    match expr {
        Expr::ValUse(vu) => locally_defined.contains(&vu.val_id.0),
        _ => direct_children(expr)
            .into_iter()
            .any(|c| references_locally_defined(c, locally_defined)),
    }
}

fn inline_single_use_vals(expr: Expr) -> Expr {
    match expr {
        Expr::BlockValue(s) => {
            let inner = s.expr;
            // First, recurse into nested blocks (If branches, inner BlockValues)
            let items: Vec<Expr> = inner
                .items
                .into_iter()
                .map(inline_single_use_vals)
                .collect();
            let result = inline_single_use_vals(*inner.result);

            // Count ValUse references for each val_id in items + result.
            // Only count uses OUTSIDE of ValDef RHS for the same val.
            let mut use_counts: std::collections::HashMap<u32, usize> =
                std::collections::HashMap::new();
            for item in &items {
                if let Expr::ValDef(vd) = item {
                    // Count uses of OTHER vals in this RHS
                    count_val_uses_in(&vd.expr.rhs, &mut use_counts);
                } else {
                    count_val_uses_in(item, &mut use_counts);
                }
            }
            count_val_uses_in(&result, &mut use_counts);

            // Build map of single-use val_id -> RHS for inlining
            //
            // S63: when a single-use val's RHS is itself a BlockValue (the user
            // wrote `val outer = { val inner = ...; body }`), substituting the
            // whole BlockValue at the use site traps `inner` inside whatever
            // scope the use site lives in (e.g. an inner If branch). Scala's
            // TreeBuilding instead lifts the inner ValDefs to the surrounding
            // block — `inner`'s sym is owned by the outer Lambda scope, and
            // only the BlockValue's RESULT lands at the use site. Mirror that:
            // hoist the inner items[] to the surrounding `items` list and
            // inline only the result expression. Closes spectrum_n2t/t2t pool
            // byte-match (the +2 was the trapped `_deltaSupplyLP` block
            // wrapper inside validRedemption's branch).
            // IndexMap (insertion-order) instead of HashMap so that the
            // sequential `replace_all` calls below run in source order.
            // HashMap iteration order is non-deterministic, and the
            // substitutions are not commutative when one inlined ValDef's
            // RHS references another inlined ValDef — different visit orders
            // produce different final trees. Source order (the order ValDefs
            // appear in `items`) is the canonical order Scala's TreeBuilding
            // would visit, so freezing iteration to insertion order both
            // makes USED NODE byte counts deterministic and matches Scala
            // parity. Closes the run-to-run non-determinism on the volatile
            // USED NODE cluster (sigmausd, paideia) — WS-E.3.
            let mut inline_map: indexmap::IndexMap<u32, Expr> = indexmap::IndexMap::new();
            let mut hoisted_items: Vec<Expr> = Vec::new();
            // S64: track ids of hoisted ValDefs so a post-hoist dedup pass
            // can fold structurally-identical inline expressions in the
            // surrounding scope into ValUses of the hoisted ValDef. Mirrors
            // Scala's graph-IR hash-cons: an inline `Upcast(deltaSupplyLP,
            // BigInt)` in `validDepositing`'s body collapses to the same
            // symbol as the hoisted `_deltaSupplyLP = deltaSupplyLP.toBigInt`
            // from `validRedemption`'s body, so the inline 4-byte
            // `Upcast(ValUse(N), BigInt)` becomes a 2-byte `ValUse(_dLP)`
            // reference. Closes spectrum_n2t/t2t's remaining +2-byte gap.
            let mut hoisted_ids: Vec<u32> = Vec::new();
            for item in &items {
                if let Expr::ValDef(vd) = item {
                    let id = vd.expr.id.0;
                    let count = use_counts.get(&id).copied().unwrap_or(0);
                    if count == 1 {
                        let mut rhs_to_inline =
                            if let Expr::BlockValue(inner_bv) = &*vd.expr.rhs {
                                // Hoist inner items to surrounding scope; inline
                                // only the BlockValue's result at the use site.
                                for inner in inner_bv.expr.items.iter() {
                                    if let Expr::ValDef(inner_vd) = inner {
                                        hoisted_ids.push(inner_vd.expr.id.0);
                                    }
                                    hoisted_items.push(inner.clone());
                                }
                                (*inner_bv.expr.result).clone()
                            } else {
                                (*vd.expr.rhs).clone()
                            };
                        // Forward-substitute prior single-use inlines into this
                        // RHS so the later sequential `replace_all` over the
                        // block doesn't reintroduce a ValUse(K) whose ValDef(K)
                        // has already been dropped. Without this, a chain like
                        //   val K = ...; val J = f(K)
                        // (where both are single-use, J is the only user of K,
                        // and J is used in the block result) inlines K's
                        // ValDef on iteration 1 (no block-side ValUse(K) to
                        // substitute), then on iteration 2 substitutes ValUse(J)
                        // → f(ValUse(K)), leaving a dangling ValUse(K) at the
                        // use site. Surfaced by oracle_refresh's
                        //   val sum     = lastSortedSum._2._2
                        //   val average = sum / dataPoints.size
                        // pair, which broke the constant-segregation roundtrip
                        // with `ValDefIdNotFound(ValId(13))`.
                        for (prior_id, prior_rhs) in &inline_map {
                            let prior_use = Expr::ValUse(ValUse {
                                val_id: ValId(*prior_id),
                                tpe: prior_rhs.tpe(),
                            });
                            rhs_to_inline = replace_all(&rhs_to_inline, &prior_use, prior_rhs);
                        }
                        inline_map.insert(id, rhs_to_inline);
                    }
                }
            }

            if inline_map.is_empty() {
                return Expr::BlockValue(Spanned {
                    source_span: s.source_span,
                    expr: BlockValue {
                        items,
                        result: result.into(),
                    },
                });
            }

            // Remove inlined ValDefs from items, splicing hoisted ValDefs
            // (from inlined BlockValue-RHS vals) into the surrounding `items`
            // at the inlined val's position. This preserves source-order so
            // dfs_reassign_val_ids can schedule the hoisted vals correctly.
            let mut remaining_items: Vec<Expr> =
                Vec::with_capacity(items.len() + hoisted_items.len());
            for item in items.into_iter() {
                if let Expr::ValDef(vd) = &item {
                    if inline_map.contains_key(&vd.expr.id.0) {
                        if let Expr::BlockValue(inner_bv) = &*vd.expr.rhs {
                            let n = inner_bv.expr.items.len();
                            let take: Vec<Expr> = hoisted_items.drain(..n).collect();
                            remaining_items.extend(take);
                        }
                        // Skip the val itself (it is being inlined)
                        continue;
                    }
                }
                remaining_items.push(item);
            }

            // Substitute all inlined vals using replace_all
            let mut block = Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: remaining_items.clone(),
                    result: result.into(),
                },
            });
            for (val_id, rhs) in &inline_map {
                let val_use = Expr::ValUse(ValUse {
                    val_id: ValId(*val_id),
                    tpe: rhs.tpe(),
                });
                block = replace_all(&block, &val_use, rhs);
            }
            // S64 post-hoist dedup: for each hoisted ValDef whose RHS is a
            // small wrapper (Upcast/Negation of a ValUse), find any inline
            // occurrence of that exact RHS *elsewhere* in the surrounding
            // block and replace it with a ValUse to the hoisted ValDef.
            // Skip the hoisted ValDef's own item to avoid self-reference.
            //
            // Example: validRedemption's body declares
            //   `val _dLP = deltaSupplyLP.toBigInt` (=Upcast(ValUse(N), SBigInt))
            // and validDepositing's body has the same Upcast inline (auto-
            // upcast of `deltaSupplyLP <= sharesUnlocked`). After hoist,
            // the ValDef sits in outer items[]; this dedup folds the inline
            // copy into a ValUse so both sites share the hoisted symbol —
            // matching Scala's graph-IR hash-cons.
            if !hoisted_ids.is_empty() {
                if let Expr::BlockValue(bv) = block {
                    let mut new_items: Vec<Expr> = bv.expr.items.to_vec();
                    let mut new_result: Expr = (*bv.expr.result).clone();
                    for hid in &hoisted_ids {
                        let mut target_rhs: Option<Expr> = None;
                        for it in new_items.iter() {
                            if let Expr::ValDef(vd) = it {
                                if vd.expr.id.0 == *hid && is_dedupable_wrapper(&vd.expr.rhs) {
                                    target_rhs = Some((*vd.expr.rhs).clone());
                                    break;
                                }
                            }
                        }
                        let target_rhs = match target_rhs {
                            Some(t) => t,
                            None => continue,
                        };
                        let target_use = Expr::ValUse(ValUse {
                            val_id: ValId(*hid),
                            tpe: target_rhs.tpe(),
                        });
                        // Rewrite every item except the hoisted ValDef itself.
                        for it in new_items.iter_mut() {
                            if let Expr::ValDef(vd) = it {
                                if vd.expr.id.0 == *hid {
                                    continue;
                                }
                            }
                            *it = replace_all(it, &target_rhs, &target_use);
                        }
                        new_result = replace_all(&new_result, &target_rhs, &target_use);
                    }
                    block = Expr::BlockValue(Spanned {
                        source_span: bv.source_span,
                        expr: BlockValue {
                            items: new_items,
                            result: new_result.into(),
                        },
                    });
                }
            }

            // If every item was inlined (no non-ValDef items remain), the
            // BlockValue is just a wrapper over its result — collapse it to
            // match Scala's graph IR, which never emits an items-less BlockValue.
            if remaining_items.is_empty() && hoisted_ids.is_empty() {
                if let Expr::BlockValue(s) = block {
                    return *s.expr.result;
                }
            }
            block
        }
        other => map_children(other, inline_single_use_vals),
    }
}

/// True if RHS is a small wrapper whose dedup is byte-shrinking when the
/// wrapped expression repeats inline. Restrict to single-arg numeric wrappers
/// where the inner is a ValUse so the dedup target is uniquely structurally
/// identifiable.
fn is_dedupable_wrapper(rhs: &Expr) -> bool {
    match rhs {
        Expr::Upcast(uc) => matches!(&*uc.input, Expr::ValUse(_)),
        Expr::Negation(s) => matches!(&*s.expr.input, Expr::ValUse(_)),
        _ => false,
    }
}
/// Count ValUse references in an expression, incrementing counts in the map.
fn count_val_uses_in(expr: &Expr, counts: &mut std::collections::HashMap<u32, usize>) {
    match expr {
        Expr::ValUse(vu) => {
            *counts.entry(vu.val_id.0).or_insert(0) += 1;
        }
        _ => {
            for child in direct_children(expr) {
                count_val_uses_in(child, counts);
            }
        }
    }
}

/// Extract duplicate constants within inner BlockValues as ValDefs.
/// After CSE scope-checking prevents extracting Upcast(Const(X), BigInt) from
/// If branches, the same Const(X) may appear multiple times in an inner block.
/// ConstantStore::put() doesn't deduplicate, so each bare Const creates a
/// separate constant pool entry. This pass finds duplicates and extracts them
/// as vals, matching Scala's graph IR behavior.
fn deduplicate_inner_consts(expr: Expr) -> Expr {
    match expr {
        // Only dedup inside If branches — these correspond to Scala's ThunkDef
        // scopes where multi-use constants get their own graph symbols.
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: deduplicate_inner_consts(*if_op.condition).into(),
            true_branch: dedup_consts_in_block(*if_op.true_branch).into(),
            false_branch: dedup_consts_in_block(*if_op.false_branch).into(),
        }),
        // For all other nodes, just recurse to find nested If expressions
        other => map_children(other, deduplicate_inner_consts),
    }
}

/// Hoist and flatten BlockValues to match Scala's `processAstGraph` output.
///
/// Scala creates ONE flat BlockValue per ThunkDef scope with all ValDefs,
/// and wrapper nodes (BoolToSigmaProp, SigmaAnd, SigmaOr) appear inside
/// the BlockValue's result. Our CSE creates BlockValues inside these
/// wrappers, producing nested structures. This pass normalizes by:
///
/// 1. `BoolToSigmaProp(BlockValue([items], R))` → `BlockValue([items], BoolToSigmaProp(R))`
/// 2. `SigmaAnd/SigmaOr` with BlockValue items → hoist items out
/// 3. `BlockValue([A], BlockValue([B], R))` → `BlockValue([A, B], R)`
fn flatten_nested_blocks(expr: Expr) -> Expr {
    // Recurse bottom-up so inner nesting is resolved first
    let expr = map_children(expr, flatten_nested_blocks);

    match expr {
        // Step 1: Hoist BlockValue through BoolToSigmaProp
        Expr::BoolToSigmaProp(bts) => {
            match *bts.input {
                Expr::BlockValue(inner_s) => {
                    // BoolToSigmaProp(BlockValue([items], R)) → BlockValue([items], BoolToSigmaProp(R))
                    Expr::BlockValue(Spanned {
                        source_span: inner_s.source_span,
                        expr: BlockValue {
                            items: inner_s.expr.items,
                            result: Expr::BoolToSigmaProp(
                                ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                                    input: inner_s.expr.result,
                                },
                            )
                            .into(),
                        },
                    })
                }
                other => Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                    input: other.into(),
                }),
            }
        }

        // Step 2: Hoist BlockValue items out of SigmaAnd/SigmaOr
        Expr::SigmaAnd(sa) => {
            let mut hoisted_items: Vec<Expr> = Vec::new();
            let mut new_sa_items: Vec<Expr> = Vec::new();
            for item in sa.items.into_iter() {
                if let Expr::BlockValue(bv_s) = item {
                    hoisted_items.extend(bv_s.expr.items);
                    new_sa_items.push(*bv_s.expr.result);
                } else {
                    new_sa_items.push(item);
                }
            }
            let sigma_and = Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: new_sa_items.try_into().expect("SigmaAnd >= 2"),
            });
            if hoisted_items.is_empty() {
                sigma_and
            } else {
                Expr::BlockValue(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: BlockValue {
                        items: hoisted_items,
                        result: sigma_and.into(),
                    },
                })
            }
        }
        Expr::SigmaOr(so) => {
            let mut hoisted_items: Vec<Expr> = Vec::new();
            let mut new_so_items: Vec<Expr> = Vec::new();
            for item in so.items.into_iter() {
                if let Expr::BlockValue(bv_s) = item {
                    hoisted_items.extend(bv_s.expr.items);
                    new_so_items.push(*bv_s.expr.result);
                } else {
                    new_so_items.push(item);
                }
            }
            let sigma_or = Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: new_so_items.try_into().expect("SigmaOr >= 2"),
            });
            if hoisted_items.is_empty() {
                sigma_or
            } else {
                Expr::BlockValue(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: BlockValue {
                        items: hoisted_items,
                        result: sigma_or.into(),
                    },
                })
            }
        }

        // Step 3: Merge nested BlockValues
        Expr::BlockValue(s) => {
            let mut items = s.expr.items;
            let mut result = *s.expr.result;

            // Iteratively merge while result is another BlockValue
            while let Expr::BlockValue(inner_s) = result {
                items.extend(inner_s.expr.items);
                result = *inner_s.expr.result;
            }

            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items,
                    result: result.into(),
                },
            })
        }
        other => other,
    }
}

/// sig-15 paideia_stake_state S5 (Bug B): post-PAG chain-rewrite for
/// stable chain-link outer ValDefs.
///
/// When a `ByIndex(GlobalVars, Const)` candidate is rejected at PAG/Root as
/// `not-in-main-scope` (its uses sit inside If branches) but extracted at
/// PAG/Branch and then hoisted to outer scope by `flatten_nested_blocks`,
/// other outer-scope items whose RHSes still contain the inline form are
/// not rewritten — the chain is broken (e.g. paideia: `val 32 = INPUTS(1)`
/// lands at outer items[] but later items keep inline `ByIndex(Inputs,1)`
/// inside their `PropertyCall(box.tokens)` chain).
///
/// This pass scans outer BlockValue items for ValDefs whose RHS is a
/// stable chain link — `ByIndex(GlobalVars(_), Const(_))`, `ByIndex(ValUse,
/// Const(_))`, or `PropertyCall(ValUse, _)` — and rewrites every other
/// occurrence (in items' RHSes and in `result`) to `ValUse(id)`. The
/// existence of an outer ValDef binding the expression means CSE already
/// decided the candidate should be a sym; we propagate that decision to
/// chain references that earlier passes missed.
///
/// The pass iterates to a fixed point so deeper rungs collapse in the same
/// run — once `INPUTS(1)` is rewritten to `ValUse(N)`, the surrounding
/// `PropertyCall(ValUse(N), tokens)` becomes structurally identical to a
/// `box.tokens` ValDef, and the next iteration collapses those refs too.
///
/// Topological correctness across the rewritten items list is restored by
/// the downstream `reorder_valdefs` body-walk emission, so callers don't
/// need to topo-sort here.
fn rewrite_byindex_globalvars_chain(expr: Expr) -> Expr {
    let Expr::BlockValue(spanned) = expr else {
        return expr;
    };
    let bv = spanned.expr;
    let mut items = bv.items;
    let mut result = *bv.result;

    let is_chain_rhs = |rhs: &Expr| -> bool {
        match rhs {
            Expr::ByIndex(s) => {
                matches!(&*s.expr.input, Expr::GlobalVars(_) | Expr::ValUse(_))
                    && matches!(&*s.expr.index, Expr::Const(_))
            }
            Expr::PropertyCall(s) => matches!(&*s.expr.obj, Expr::ValUse(_)),
            _ => false,
        }
    };
    let mut applied_ids: HashSet<u32> = HashSet::new();
    // Iteration cap is a safety net — the rewrite is monotone (each iteration
    // marks at least one ValDef as applied) so it converges in O(items.len()).
    for _ in 0..items.len().saturating_add(1) {
        let snapshot: Vec<(usize, u32, Expr)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if let Expr::ValDef(vd) = item {
                    let rhs: &Expr = &vd.expr.rhs;
                    if is_chain_rhs(rhs) && !applied_ids.contains(&vd.expr.id.0) {
                        return Some((idx, vd.expr.id.0, rhs.clone()));
                    }
                }
                None
            })
            .collect();
        if snapshot.is_empty() {
            break;
        }
        for (def_idx, val_id, rhs) in snapshot {
            applied_ids.insert(val_id);
            let val_use = Expr::ValUse(ValUse {
                val_id: ValId(val_id),
                tpe: expr_type(&rhs),
            });
            for (k, item) in items.iter_mut().enumerate() {
                if k == def_idx {
                    continue;
                }
                *item = replace_all(item, &rhs, &val_use);
            }
            result = replace_all(&result, &rhs, &val_use);
        }
    }

    Expr::BlockValue(Spanned {
        source_span: spanned.source_span,
        expr: BlockValue {
            items,
            result: result.into(),
        },
    })
}

/// Dedup inner-scope expressions within an If branch.
/// Phase 1: OptionGet dedup (single pass, original behavior).
/// Phase 2: PropertyCall/ByIndex dedup (iterative, handles dependency chains).
fn dedup_consts_in_block(expr: Expr) -> Expr {
    // First recurse to handle nested If expressions
    let mut expr = deduplicate_inner_consts(expr);

    // Compute set of ValIds defined ONLY inside nested-If branches of this
    // expr (not at expr's top-level items). Candidates that reference these
    // cannot be hoisted to expr's top-level scope — doing so creates a
    // forward ValUse(id) reference to a branch-local ValDef.
    let branch_local_ids = collect_branch_local_val_ids(&expr);
    let filter_candidates = |duplicates: Vec<Expr>| -> Vec<Expr> {
        duplicates
            .into_iter()
            .filter(|d| {
                let mut uses = Vec::new();
                collect_all_val_uses(d, &mut uses);
                uses.iter().all(|id| !branch_local_ids.contains(id))
            })
            .collect()
    };

    // Bidirectional Thunk-aware count: a candidate that references a
    // local-scope ValDef is "anchored" to this block (its sym lives here
    // in Scala's IR via capture-set placement) — count global occurrences.
    // A candidate that references no local item could live in any sibling
    // Thunk and would NOT hash-cons up to this scope — count only
    // occurrences reachable without crossing inner-Thunk boundaries.
    let scope_local_ids = top_level_val_ids(&expr);
    let count_for_rule = |target: &Expr, tree: &Expr| -> usize {
        let rescue =
            !scope_local_ids.is_empty() && expr_references_any_local(target, &scope_local_ids);
        if rescue {
            // S75 thunk-only-discount: even when target references a local
            // ValDef, suppress extraction if EVERY occurrence sits inside
            // a sibling &&/|| right-arm Thunk (or If branch) of this scope.
            // In Scala's graph IR, those uses each build their own
            // per-Thunk sym via findOrCreateDefinition — the outer scope
            // sym never sees them, so hasManyUsagesGlobal returns false
            // and no ValDef is created here.
            //
            // Closes chaincash receiptOut.R6[Int].get over-extraction:
            // both uses are deep right arms of the properReceipt && chain,
            // never reaching main scope.
            //
            // Safety: candidates with at least one main-scope occurrence
            // (e.g. ProxyBorrow's PropertyCall(VU(N), tokens) where the
            // ValDef RHS lives in the else-branch BlockValue main items)
            // pass the check and still extract.
            if !appears_in_main_scope(tree, target) {
                return 0;
            }
            count_occurrences(tree, target)
        } else {
            count_occurrences_scope(tree, target)
        }
    };

    // Phase 1: OptionGet dedup — single pass matching original behavior.
    let mut option_gets: Vec<Expr> = Vec::new();
    collect_option_gets(&expr, &mut option_gets);
    let mut og_seen: Vec<Expr> = Vec::new();
    let mut og_duplicates: Vec<Expr> = Vec::new();
    for c in &option_gets {
        if og_seen.iter().any(|s| s == c) {
            continue;
        }
        og_seen.push(c.clone());
        if count_for_rule(c, &expr) >= 2 {
            og_duplicates.push(c.clone());
        }
    }
    let og_duplicates = filter_candidates(og_duplicates);
    if !og_duplicates.is_empty() {
        expr = extract_inner_vals(expr, og_duplicates);
    }

    // Phase 2: PropertyCall/ByIndex dedup — iterative to handle dependency
    // chains (PropertyCall extracted first, then ByIndex referencing it).
    //
    // Thresholds:
    // - PropertyCall: ≥3 uses (with 2 uses, extraction costs 1B: inline 2×5B=10B
    //   vs extracted 7B+2×2B=11B). With ≥3, it saves bytes.
    // - ByIndex: ≥2 uses (extraction is body-size-neutral but saves ~2B from
    //   constant pool dedup — each inline ByIndex has its own Const index node).
    loop {
        let mut candidates: Vec<Expr> = Vec::new();
        collect_property_byindex_candidates(&expr, &mut candidates);

        let mut seen: Vec<Expr> = Vec::new();
        let mut duplicates: Vec<Expr> = Vec::new();
        for c in &candidates {
            if seen.iter().any(|s| s == c) {
                continue;
            }
            seen.push(c.clone());
            let min_count = if matches!(c, Expr::PropertyCall(_)) {
                3
            } else {
                2
            };
            if count_for_rule(c, &expr) >= min_count {
                duplicates.push(c.clone());
            }
        }

        let duplicates = filter_candidates(duplicates);
        if duplicates.is_empty() {
            break;
        }

        // Sort by AST size (smallest first) to handle dependencies
        let mut duplicates = duplicates;
        duplicates.sort_by_key(expr_size);

        // Extract all duplicates of the smallest size in this pass
        let smallest_size = expr_size(&duplicates[0]);
        let batch: Vec<Expr> = duplicates
            .into_iter()
            .take_while(|e| expr_size(e) == smallest_size)
            .collect();

        expr = extract_inner_vals(expr, batch);
    }

    // Phase 3: Arithmetic BinOp dedup — extracts expressions like
    // Minus(ValUse(X), Const(Y)) that appear ≥2 times within a branch.
    // In Scala's graph, arithmetic ops are hash-consed (singleton ExactNumeric).
    // Body-wise extraction costs 1B with 2 uses, but saves ~2B from constant
    // pool dedup (each inline BinOp has its own Const node), net -1B.
    //
    // S42: skip BinOps whose both operands are ValUses — those over-extract
    // (e.g. SigUSDV1's Minus(VU, VU) at count=3 costs 1B). And always run
    // inline_single_use_vals afterwards: gating it on `extracted.is_empty()`
    // leaks ~23B in cascade because single-use vals (like the ExtractRegisterAs
    // inside an OptionGet) stay split into separate ValDefs.
    {
        let log = std::env::var("CSE_PHASE3_LOG").is_ok();

        let mut candidates: Vec<Expr> = Vec::new();
        collect_binop_candidates(&expr, &mut candidates);

        if log {
            eprintln!(
                "[P3] === Phase 3 BinOp dedup === (raw_candidates={})",
                candidates.len()
            );
        }

        let mut seen: Vec<Expr> = Vec::new();
        let mut duplicates: Vec<Expr> = Vec::new();
        for c in &candidates {
            if seen.iter().any(|s| s == c) {
                continue;
            }
            seen.push(c.clone());
            let cnt = count_for_rule(c, &expr);
            let kept = cnt >= 2;
            if log {
                eprintln!(
                    "[P3] cand size={} count={} kept_thresh={} :: {}",
                    expr_size(c),
                    cnt,
                    kept,
                    format_binop_brief(c)
                );
            }
            if kept {
                duplicates.push(c.clone());
            }
        }

        let duplicates = filter_candidates(duplicates);

        // Drop arithmetic BinOps whose both operands are ValUses (S42
        // over-extract pattern — e.g. SigUSDV1's `Minus(VU, VU)` at count=3
        // costs 1B). Relational BinOps (Eq/NEq/LT/LE/GT/GE) ARE kept even
        // with both-ValUse operands: their two-arm `&&`-chain occurrences
        // (sig-15 sigmausd_bank's `dataInput.tokens(0)._1 == oraclePoolNFT`)
        // genuinely save bytes when extracted into one branch ValDef + 2
        // ValUses (vs. two inline BinOps + the otherwise-required outer ValDef
        // for the const operand).
        let duplicates: Vec<Expr> = duplicates
            .into_iter()
            .filter(|d| {
                if let Expr::BinOp(s) = d {
                    let is_arith =
                        matches!(s.expr.kind, ergotree_ir::mir::bin_op::BinOpKind::Arith(_));
                    let both_valuse = matches!(&*s.expr.left, Expr::ValUse(_))
                        && matches!(&*s.expr.right, Expr::ValUse(_));
                    if log && is_arith && both_valuse {
                        eprintln!("[P3] DROP both-ValUse arith :: {}", format_binop_brief(d));
                    }
                    !(is_arith && both_valuse)
                } else {
                    true
                }
            })
            .collect();

        if log {
            eprintln!("[P3] final extract count: {}", duplicates.len());
        }

        if !duplicates.is_empty() {
            expr = extract_inner_vals(expr, duplicates);
        }
        // Always inline: even when nothing was extracted, some source-level
        // vals may be single-use (e.g. ExtractRegisterAs nested under OptionGet)
        // and must collapse to match Scala's graph output.
        expr = inline_single_use_vals(expr);
    }

    expr
}

/// Collect ValDef IDs at the top level of a block-shaped expression.
/// Recognizes Expr::BlockValue and Expr::BoolToSigmaProp(BlockValue).
fn top_level_val_ids(expr: &Expr) -> std::collections::HashSet<u32> {
    let items: &[Expr] = match expr {
        Expr::BlockValue(s) => &s.expr.items,
        Expr::BoolToSigmaProp(bts) => match &*bts.input {
            Expr::BlockValue(s) => &s.expr.items,
            _ => return std::collections::HashSet::new(),
        },
        _ => return std::collections::HashSet::new(),
    };
    items
        .iter()
        .filter_map(|i| {
            if let Expr::ValDef(vd) = i {
                Some(vd.expr.id.0)
            } else {
                None
            }
        })
        .collect()
}

/// Collect ValIds defined inside nested-If branches of `expr`, excluding
/// the top-level items of the outermost BlockValue in `expr`.
fn collect_branch_local_val_ids(expr: &Expr) -> std::collections::HashSet<u32> {
    let mut ids = std::collections::HashSet::new();
    fn walk(expr: &Expr, inside_if_branch: bool, ids: &mut std::collections::HashSet<u32>) {
        match expr {
            Expr::ValDef(vd) => {
                if inside_if_branch {
                    ids.insert(vd.expr.id.0);
                }
                walk(&vd.expr.rhs, inside_if_branch, ids);
            }
            Expr::If(if_op) => {
                walk(&if_op.condition, inside_if_branch, ids);
                walk(&if_op.true_branch, true, ids);
                walk(&if_op.false_branch, true, ids);
            }
            other => {
                for c in direct_children(other) {
                    walk(c, inside_if_branch, ids);
                }
            }
        }
    }
    walk(expr, false, &mut ids);
    ids
}

/// Extract a batch of duplicate expressions as inner ValDefs.
fn extract_inner_vals(expr: Expr, duplicates: Vec<Expr>) -> Expr {
    match expr {
        Expr::BlockValue(s) => {
            let mut next_id = find_max_val_id(&Expr::BlockValue(s.clone())) + 1;
            let mut result_items = s.expr.items;
            let mut result_expr = *s.expr.result;
            let mut new_defs: Vec<Expr> = Vec::new();

            for dup_expr in duplicates {
                let val_use = Expr::ValUse(ValUse {
                    val_id: ValId(next_id),
                    tpe: dup_expr.tpe(),
                });
                result_items = result_items
                    .into_iter()
                    .map(|item| replace_all(&item, &dup_expr, &val_use))
                    .collect();
                result_expr = replace_all(&result_expr, &dup_expr, &val_use);
                new_defs.push(Expr::ValDef(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: ValDef {
                        id: ValId(next_id),
                        rhs: dup_expr.into(),
                    },
                }));
                next_id += 1;
            }

            new_defs.extend(result_items);
            // Topologically order: the new ValDefs may reference existing
            // item IDs via ValUse (if the extracted expr contained such
            // references). And existing items may reference new ValDef IDs
            // via replace_all. Sort so every ValUse sees its ValDef earlier.
            let ordered = topo_order_valdefs(new_defs);
            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: ordered,
                    result: result_expr.into(),
                },
            })
        }
        // BoolToSigmaProp(BlockValue) — add items to inner BlockValue so
        // new ValDefs share the same scope as existing vals (enables inlining
        // of vals that become single-use after extraction).
        Expr::BoolToSigmaProp(bts) if matches!(&*bts.input, Expr::BlockValue(_)) => {
            let inner = extract_inner_vals(*bts.input, duplicates);
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: inner.into(),
            })
        }
        // BoolToSigmaProp(If(...)) — similar unwrap through BoolToSigmaProp
        Expr::BoolToSigmaProp(bts) if matches!(&*bts.input, Expr::If(_)) => {
            let inner = extract_inner_vals(*bts.input, duplicates);
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: inner.into(),
            })
        }
        other => {
            // Bare expression — wrap in new BlockValue
            let mut next_id = find_max_val_id(&other) + 1;
            let mut result_expr = other;
            let mut new_defs: Vec<Expr> = Vec::new();

            for dup_expr in duplicates {
                let val_use = Expr::ValUse(ValUse {
                    val_id: ValId(next_id),
                    tpe: dup_expr.tpe(),
                });
                result_expr = replace_all(&result_expr, &dup_expr, &val_use);
                new_defs.push(Expr::ValDef(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: ValDef {
                        id: ValId(next_id),
                        rhs: dup_expr.into(),
                    },
                }));
                next_id += 1;
            }

            Expr::BlockValue(Spanned {
                source_span: SourceSpan::empty(),
                expr: BlockValue {
                    items: new_defs,
                    result: result_expr.into(),
                },
            })
        }
    }
}

/// Collect all Const nodes in an expression tree.
#[allow(dead_code)]
fn collect_consts(expr: &Expr, out: &mut Vec<Expr>) {
    if let Expr::Const(_) = expr {
        out.push(expr.clone());
    }
    for child in direct_children(expr) {
        collect_consts(child, out);
    }
}

/// Collect OptionGet nodes in an expression tree (Phase 1).
fn collect_option_gets(expr: &Expr, out: &mut Vec<Expr>) {
    if let Expr::OptionGet(_) = expr {
        out.push(expr.clone());
    }
    for child in direct_children(expr) {
        collect_option_gets(child, out);
    }
}

/// Collect PropertyCall/ByIndex candidates for inner-scope dedup (Phase 2).
/// Only collects expressions whose direct input is a ValUse (derived from
/// outer-scope or previously-extracted inner val). This prevents extracting
/// root-level expressions like PropertyCall(SELF) or ByIndex(OUTPUTS, ...)
/// which are already handled by root-scope CSE.
fn collect_property_byindex_candidates(expr: &Expr, out: &mut Vec<Expr>) {
    match expr {
        Expr::PropertyCall(pc) => {
            if matches!(&*pc.expr.obj, Expr::ValUse(_)) {
                out.push(expr.clone());
            }
        }
        Expr::ByIndex(bi) => {
            if matches!(&*bi.expr.input, Expr::ValUse(_)) {
                out.push(expr.clone());
            }
        }
        _ => {}
    }
    for child in direct_children(expr) {
        collect_property_byindex_candidates(child, out);
    }
}

/// One-line brief for a BinOp candidate, for Phase 3 instrumentation logs.
/// Format: `Kind(left, right)` where leaves are summarized (ValUse(id),
/// `Const(<type>=<short>)`, or just the variant name for nested expressions).
fn format_expr_leaf(expr: &Expr) -> String {
    match expr {
        Expr::ValUse(vu) => format!("VU({})", vu.val_id.0),
        Expr::Const(c) => format!("Const({:?})", c.tpe),
        Expr::BinOp(s) => format!(
            "BinOp({:?}, {}, {})",
            s.expr.kind,
            format_expr_leaf(&s.expr.left),
            format_expr_leaf(&s.expr.right)
        ),
        Expr::OptionGet(og) => format!("OptionGet({})", format_expr_leaf(&og.expr.input)),
        Expr::PropertyCall(pc) => format!("PC.{}", pc.expr.method.name()),
        Expr::ByIndex(_) => "ByIndex(..)".to_string(),
        other => {
            let s = format!("{:?}", other);
            let s = s.split_whitespace().next().unwrap_or("?").to_string();
            s.chars().take(24).collect()
        }
    }
}

fn format_binop_brief(expr: &Expr) -> String {
    if let Expr::BinOp(s) = expr {
        format!(
            "BinOp({:?}, {}, {})",
            s.expr.kind,
            format_expr_leaf(&s.expr.left),
            format_expr_leaf(&s.expr.right)
        )
    } else {
        format_expr_leaf(expr)
    }
}

/// Collect BinOp candidates for inner-scope dedup (Phase 3).
/// Collects:
///   - arithmetic BinOps (Minus, Plus, Multiply, etc.) where at least one child
///     is a ValUse — Scala hash-conses these via singleton `ExactNumeric`.
///   - relational BinOps (Eq/NEq/LT/LE/GT/GE) where at least one child is a
///     ValUse. Equals/NotEquals nodes ARE hash-consed when both operands are
///     graph-shared syms (per `is_graph_shared`); Ordering ops use singleton
///     `ExactOrdering`. Including these closes the sig-15 sigmausd_bank gap
///     where `BinOp(Eq, dataInput.tokens(0)._1, oraclePoolNFT)` appears in two
///     `&&`-chain arms within the if-true branch — `count_dag_usages_scope`
///     undercounts because both occurrences sit inside `&&` right-arm
///     ThunkDefs, so root and branch CSE never see ≥2 occurrences. The Phase
///     3 `count_for_rule` rescue path (used when the candidate references a
///     scope-local ValDef) DOES count globally with the main-scope guard, so
///     adding the candidate here is enough.
fn collect_binop_candidates(expr: &Expr, out: &mut Vec<Expr>) {
    if let Expr::BinOp(s) = expr {
        let is_eligible = matches!(
            s.expr.kind,
            ergotree_ir::mir::bin_op::BinOpKind::Arith(_)
                | ergotree_ir::mir::bin_op::BinOpKind::Relation(_)
        );
        if is_eligible
            && (matches!(&*s.expr.left, Expr::ValUse(_))
                || matches!(&*s.expr.right, Expr::ValUse(_)))
        {
            out.push(expr.clone());
        }
    }
    for child in direct_children(expr) {
        collect_binop_candidates(child, out);
    }
}

/// Count AST nodes in an expression (for sorting duplicates by size).
fn expr_size(expr: &Expr) -> usize {
    1 + direct_children(expr)
        .iter()
        .map(|c| expr_size(c))
        .sum::<usize>()
}

/// DFS walk an expression; when a ValUse is found whose ValDef is in val_map
/// and hasn't been emitted yet, recursively emit its RHS dependencies first,
/// then emit the ValDef.
///
/// Collect all ValUse IDs referenced in an expression tree.
fn collect_all_val_uses(expr: &Expr, out: &mut Vec<u32>) {
    if let Expr::ValUse(vu) = expr {
        out.push(vu.val_id.0);
    }
    for child in direct_children(expr) {
        collect_all_val_uses(child, out);
    }
}

/// In Scala's graph, this corresponds to being inside a ThunkDef body. The
/// ThunkDef's flatSchedule lists inner ThunkDefs before their parent expressions,
/// which causes the innermost || in a left-associative chain to effectively
/// process its right arm before the left arm.
///
/// `parent_is_or` is true iff this expression's immediate parent in the
/// recursion is a `BinOp(Or)`. Used to gate the OR-chain reverse_inner logic
/// so it only fires inside actual OR chains. For a standalone `(a || b)` whose
/// parent is `And`, sigmaProp wrap, or a ValDef RHS, this stays false and the
/// natural left-then-right walk is preserved (sig-15 sigmausd_bank S4 fix).
fn emit_deps(
    expr: &Expr,
    val_map: &HashMap<u32, Expr>,
    emitted: &mut Vec<Expr>,
    emitted_ids: &mut HashSet<u32>,
    in_thunk: bool,
    parent_is_or: bool,
) {
    match expr {
        Expr::ValUse(vu) => {
            let id = vu.val_id.0;
            if !emitted_ids.contains(&id) {
                if let Some(vd_expr) = val_map.get(&id) {
                    if let Expr::ValDef(vd) = vd_expr {
                        // Emit dependencies of this ValDef's RHS first
                        emit_deps(&vd.expr.rhs, val_map, emitted, emitted_ids, in_thunk, false);
                    }
                    emitted_ids.insert(id);
                    emitted.push(vd_expr.clone());
                }
            }
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                emit_deps(item, val_map, emitted, emitted_ids, in_thunk, false);
            }
            emit_deps(&s.expr.result, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::ValDef(s) => emit_deps(&s.expr.rhs, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::BinOp(s) => {
            let is_logical = matches!(
                s.expr.kind,
                ergotree_ir::mir::bin_op::BinOpKind::Logical(
                    ergotree_ir::mir::bin_op::LogicalOp::And
                        | ergotree_ir::mir::bin_op::LogicalOp::Or
                )
            );
            let is_or = matches!(
                s.expr.kind,
                ergotree_ir::mir::bin_op::BinOpKind::Logical(
                    ergotree_ir::mir::bin_op::LogicalOp::Or
                )
            );
            if is_logical {
                // In Scala's graph, logical &&/|| wraps the right arm in a ThunkDef.
                // Inside a ThunkDef body's flatSchedule, inner ThunkDefs are listed
                // before their parent expressions. For a left-associative || chain
                // ((a || b) || c) || d, this means the innermost ||'s right arm (b)
                // is processed before its left arm (a). The innermost || is identified
                // by its left operand NOT being another BinOp(Or).
                let left_is_or = matches!(&*s.expr.left, Expr::BinOp(lb) if matches!(
                    lb.expr.kind,
                    ergotree_ir::mir::bin_op::BinOpKind::Logical(
                        ergotree_ir::mir::bin_op::LogicalOp::Or
                    )
                ));
                // Gate `reverse_inner` on `parent_is_or` so it only fires for the
                // innermost OR of an actual OR chain (where the parent is also an
                // OR). A standalone `(a || b)` whose parent is `And`, `sigmaProp`,
                // or a ValDef RHS has parent_is_or=false → no reverse, preserving
                // natural left-to-right dep-walk. sig-15 sigmausd_bank S4 close.
                let reverse_inner = in_thunk && is_or && !left_is_or && parent_is_or;
                if reverse_inner {
                    // Innermost || in thunk context: right arm first (matches Scala's
                    // ThunkDef body flatSchedule where ThunkDef children come before parents)
                    emit_deps(&s.expr.right, val_map, emitted, emitted_ids, true, is_or);
                    emit_deps(&s.expr.left, val_map, emitted, emitted_ids, true, is_or);
                } else {
                    // Left arm is in main scope (not thunk)
                    emit_deps(&s.expr.left, val_map, emitted, emitted_ids, in_thunk, is_or);
                    // Right arm enters thunk context
                    emit_deps(&s.expr.right, val_map, emitted, emitted_ids, true, is_or);
                }
            } else {
                emit_deps(&s.expr.left, val_map, emitted, emitted_ids, in_thunk, false);
                emit_deps(&s.expr.right, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::BoolToSigmaProp(bts) => {
            emit_deps(&bts.input, val_map, emitted, emitted_ids, in_thunk, false)
        }
        Expr::If(if_op) => {
            // Condition is in the main scope — process normally
            emit_deps(&if_op.condition, val_map, emitted, emitted_ids, in_thunk, false);
            // If branches are ThunkDef scopes in Scala's graph IR.
            // ThunkDef.deps (free variables) are ordered by symbol ID,
            // so we collect all val refs from both branches, sort by ID,
            // and emit in that order (matching Scala's schedule).
            let mut branch_val_ids: Vec<u32> = Vec::new();
            collect_all_val_uses(&if_op.true_branch, &mut branch_val_ids);
            collect_all_val_uses(&if_op.false_branch, &mut branch_val_ids);
            // Sig-15 sigmausd_bank S5: detect "transitive interleave" where a
            // direct branch VU's RHS references ANOTHER direct branch VU.
            // In this pattern, the sort-by-ID emits the depended-on val
            // before its dependent, but recursive emission of the dependent's
            // RHS surfaces a non-direct val (e.g. val_34=BinOp reachable only
            // via val_35) that should ALSO be emitted before its sibling
            // dep (val_33=If, also a direct branch VU). Sort-by-ID can't see
            // this third val, so it ends up after val_33 instead of before.
            //
            // For sigmausd_bank's `validReserveRatio` outer If: branches
            // contain {VU(scExchange), VU(scCircDelta), VU(maxRR=If),
            // VU(reserveRatioPercentOut=val_35)}. val_35's RHS references
            // VU(maxRR=If) AND VU(BinOp_22*29) — the latter is NOT in
            // direct branches. NODE emits BinOp before If (graph sym
            // creation order); sort-by-ID emits If first (smaller pre-
            // renumber ID), then val_35 surfaces BinOp via its RHS walk.
            //
            // Strict-subset gate: when ANY direct branch VU's RHS references
            // ANOTHER direct branch VU, fall through to recursive emit_deps
            // walk on both branches. Recursive walk preserves first-use-DFS
            // order — when val_35 is the first reached, its RHS walk emits
            // BinOp first (cond), then If (true branch), then val_35 itself,
            // matching NODE.
            let direct_set: HashSet<u32> = branch_val_ids.iter().copied().collect();
            let mut has_direct_interleave = false;
            for &id in &branch_val_ids {
                if let Some(Expr::ValDef(vd)) = val_map.get(&id) {
                    let mut deps: Vec<u32> = Vec::new();
                    collect_all_val_uses(&vd.expr.rhs, &mut deps);
                    if deps
                        .iter()
                        .any(|d| *d != id && direct_set.contains(d))
                    {
                        has_direct_interleave = true;
                        break;
                    }
                }
            }
            // Only apply the recursive-walk path for inner blocks (non-dense
            // val_map). Outer blocks (dense 1..N keys after Pass 1a / Pass 1b)
            // already have body-schedule-aligned IDs, so the existing sort-
            // by-ID emission matches Scala's schedule there.
            let dense_outer = !val_map.is_empty()
                && val_map.keys().copied().max().unwrap_or(0) == val_map.len() as u32
                && val_map.keys().copied().min().unwrap_or(0) >= 1;
            if has_direct_interleave && !dense_outer {
                emit_deps(
                    &if_op.true_branch,
                    val_map,
                    emitted,
                    emitted_ids,
                    in_thunk,
                    false,
                );
                emit_deps(
                    &if_op.false_branch,
                    val_map,
                    emitted,
                    emitted_ids,
                    in_thunk,
                    false,
                );
                return;
            }
            // S62: expand transitively. A direct branch-VU like
            // `validBankRecreation` may have an RHS referencing other outer
            // vals (e.g. minBankValue/R6) that themselves are NOT directly
            // referenced in the branches. Without expansion, the sort below
            // doesn't see those transitive deps and they get emitted only as
            // a side-effect of recursion when their parent is processed —
            // which puts them AFTER siblings that happened to be referenced
            // directly. Transitive collection ensures every reachable outer
            // val is in the sort, producing Scala's schedule (deps before
            // dependent in seq-id order).
            let dense_post_reassign_pre = !val_map.is_empty()
                && val_map.keys().copied().max().unwrap_or(0) == val_map.len() as u32
                && val_map.keys().copied().min().unwrap_or(0) >= 1;
            if dense_post_reassign_pre {
                let mut transitive: HashSet<u32> = branch_val_ids.iter().copied().collect();
                let mut work: Vec<u32> = branch_val_ids.clone();
                while let Some(id) = work.pop() {
                    if let Some(Expr::ValDef(vd)) = val_map.get(&id) {
                        let mut deps: Vec<u32> = Vec::new();
                        collect_all_val_uses(&vd.expr.rhs, &mut deps);
                        for d in deps {
                            if val_map.contains_key(&d) && transitive.insert(d) {
                                work.push(d);
                            }
                        }
                    }
                }
                branch_val_ids = transitive.into_iter().collect();
            }
            branch_val_ids.sort();
            branch_val_ids.dedup();
            // Partition: non-Const(/CP)-RHS ValDefs first, then Const-RHS.
            // In Scala's graph, sym IDs reflect creation order; bare Const
            // syms tend to be created later (when the body's expressions are
            // built) than chained MethodCall/extract syms, so Const-RHS
            // ValDefs land at the end of the outer block. Mirror that
            // empirically here. (S47 — closes SigUSDV1 pool ordering)
            //
            // S60: skip the partition when val_map's keys are dense 1..N
            // (i.e. the enclosing BlockValue has been processed by
            // `dfs_reassign_val_ids`, so IDs already reflect body-walk
            // first-encounter order = Scala's flatSchedule). In that case
            // sorting by ID alone reproduces Scala's order, and partition
            // wrongly defers a top-level `val` Const literal that's
            // referenced early (OpenOrderToken: `_tokenId` was at items[3]
            // in Scala but the partition pushed it to items[8]). Inner
            // BlockValues are NOT touched by `dfs_reassign_val_ids`, so
            // their pre-renumber IDs include CSE-extracted high IDs and
            // are sparse — partition still needed there (SigUSDV1's
            // `currency` ConstPH inside the true-branch BlockValue).
            let dense_post_reassign = !val_map.is_empty()
                && val_map.keys().copied().max().unwrap_or(0) == val_map.len() as u32
                && val_map.keys().copied().min().unwrap_or(0) >= 1;
            let branch_val_ids: Vec<u32> = if dense_post_reassign {
                branch_val_ids
            } else {
                let (const_ids, nonconst_ids): (Vec<u32>, Vec<u32>) =
                    branch_val_ids.into_iter().partition(|id| {
                        val_map.get(id).is_some_and(|vd| {
                            if let Expr::ValDef(s) = vd {
                                matches!(&*s.expr.rhs, Expr::Const(_) | Expr::ConstPlaceholder(_))
                            } else {
                                false
                            }
                        })
                    });
                nonconst_ids.into_iter().chain(const_ids).collect()
            };
            for id in branch_val_ids {
                if !emitted_ids.contains(&id) {
                    if let Some(vd_expr) = val_map.get(&id) {
                        if let Expr::ValDef(vd) = vd_expr {
                            emit_deps(&vd.expr.rhs, val_map, emitted, emitted_ids, in_thunk, false);
                        }
                        emitted_ids.insert(id);
                        emitted.push(vd_expr.clone());
                    }
                }
            }
        }
        Expr::Filter(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.condition, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::Exists(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.condition, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::ForAll(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.condition, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::Map(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.mapper, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::Fold(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.zero, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.fold_op, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::FuncValue(fv) => emit_deps(fv.body(), val_map, emitted, emitted_ids, in_thunk, false),
        Expr::PropertyCall(s) => emit_deps(&s.expr.obj, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::MethodCall(s) => {
            emit_deps(&s.expr.obj, val_map, emitted, emitted_ids, in_thunk, false);
            for a in &s.expr.args {
                emit_deps(a, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::ByteArrayToBigInt(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false)
        }
        Expr::ExtractAmount(ea) => emit_deps(&ea.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::ExtractRegisterAs(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false)
        }
        Expr::ExtractScriptBytes(esb) => {
            emit_deps(&esb.input, val_map, emitted, emitted_ids, in_thunk, false)
        }
        Expr::ExtractBytes(eb) => emit_deps(&eb.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::ExtractId(ei) => emit_deps(&ei.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::ExtractCreationInfo(eci) => {
            emit_deps(&eci.input, val_map, emitted, emitted_ids, in_thunk, false)
        }
        Expr::SizeOf(so) => emit_deps(&so.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::ByIndex(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.index, val_map, emitted, emitted_ids, in_thunk, false);
            if let Some(ref d) = s.expr.default {
                emit_deps(d, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::SelectField(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::OptionGet(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::OptionIsDefined(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false)
        }
        Expr::OptionGetOrElse(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.default, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::Slice(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.from, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.until, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::LogicalNot(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::Negation(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::SigmaPropBytes(spb) => emit_deps(&spb.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::Upcast(uc) => emit_deps(&uc.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::Downcast(dc) => emit_deps(&dc.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::CalcBlake2b256(cb) => emit_deps(&cb.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::CreateProveDlog(cpd) => {
            emit_deps(&cpd.input, val_map, emitted, emitted_ids, in_thunk, false)
        }
        Expr::CreateProveDhTuple(cpd) => {
            emit_deps(&cpd.g, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&cpd.h, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&cpd.u, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&cpd.v, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::SigmaAnd(sa) => {
            for i in sa.items.iter() {
                emit_deps(i, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::SigmaOr(so) => {
            for i in so.items.iter() {
                emit_deps(i, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::Tuple(t) => {
            for i in t.items.iter() {
                emit_deps(i, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::TreeLookup(s) => {
            emit_deps(&s.expr.tree, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.key, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.proof, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::Apply(app) => {
            emit_deps(&app.func, val_map, emitted, emitted_ids, in_thunk, false);
            for a in &app.args {
                emit_deps(a, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::And(a) => emit_deps(&a.expr.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::Or(o) => emit_deps(&o.expr.input, val_map, emitted, emitted_ids, in_thunk, false),
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                emit_deps(item, val_map, emitted, emitted_ids, in_thunk, false);
            }
        }
        Expr::Append(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.expr.col_2, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::Exponentiate(s) => {
            emit_deps(&s.left, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.right, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::MultiplyGroup(s) => {
            emit_deps(&s.left, val_map, emitted, emitted_ids, in_thunk, false);
            emit_deps(&s.right, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::DecodePoint(s) => {
            emit_deps(&s.input, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::LongToByteArray(s) => {
            emit_deps(&s.input, val_map, emitted, emitted_ids, in_thunk, false);
        }
        Expr::ByteArrayToLong(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk, false);
        }
        _ => {}
    }
}

/// Renumber all ValIds to match the Scala compiler's assignment scheme.
///
/// The Scala compiler (`TreeBuilding.processAstGraph`) uses this pattern:
///   curId starts at 0
///   for each ValDef: rhs = buildValue(defId=curId); curId += 1; ValDef(curId)
/// When buildValue encounters a lambda: varId = defId + 1 (same as the ValDef id).
/// Lambda body uses an independent counter starting at varId + 1.
/// Alpha-rename every `ValDef` in `expr` to a globally-unique id, with
/// scope-aware `ValUse` rewriting. After this pass:
///
/// - No two `ValDef`s anywhere in the tree share an id.
/// - Each `ValUse(K)` is rewritten to point at the renamed id of the
///   `ValDef` that was in scope at that position (per ergotree's nearest-
///   enclosing-binding rule).
///
/// **Why this exists:** `sequential_renumber` uses a single global
/// `HashMap<old_id, new_id>` keyed by old_id. If the input tree contains
/// two `ValDef`s with the same old_id in disjoint scopes (e.g. one in
/// each If branch, where Scala's IR legitimately resets `curId` per
/// branch), the global map collapses them into the same new_id. When
/// later combined with the `if !id_map.contains_key` skip-advance path,
/// this produces same-scope duplicate `ValDef`s in the rewritten tree.
/// See `mod renumber_scope_safety` for the failing repro that motivates
/// this pass.
///
/// Globally uniquifying ids before `sequential_renumber` runs sidesteps
/// the issue without changing `sequential_renumber`'s logic.
fn disambiguate_val_ids(mut expr: Expr) -> Expr {
    let max = find_max_val_id(&expr);
    let mut st = DisambigState {
        scopes: vec![HashMap::new()],
        next: max + 1,
    };
    disambig_walk(&mut expr, &mut st);
    expr
}

struct DisambigState {
    /// Stack of scope frames. Each frame: pre-pass-old_id → renamed_id.
    /// Pushed at: BlockValue, If branches, FuncValue body. Popped on exit.
    scopes: Vec<HashMap<u32, u32>>,
    /// Next fresh id to allocate.
    next: u32,
}

/// In-place tree walk that alpha-renames `ValDef`s to fresh ids and
/// rewrites `ValUse` references via scope chain lookup.
///
/// Uses `Traversable::children_mut()` for the structural recurse on
/// variants we don't intercept — that trait is implemented for *every*
/// `Expr` variant, so we can't accidentally skip a sub-tree the way
/// `map_children` (which has gaps such as `CreateProveDlog`, `Atleast`,
/// `Append`, `Xor`, etc.) would.
fn disambig_walk(expr: &mut Expr, st: &mut DisambigState) {
    use ergotree_ir::traversable::Traversable;
    match expr {
        Expr::BlockValue(s) => {
            // Pre-bind all top-level ValDef siblings so that ValUse
            // references between siblings (e.g. val B = ValUse(A) where A
            // appears later in source order, or mutual sibling refs that
            // survived HIR dedup) resolve in their RHS walks. Without
            // pre-binding, the first sibling's RHS sees an empty frame and
            // any forward ValUse falls through unrenamed, leaving a dangling
            // reference downstream.
            st.scopes.push(HashMap::new());
            let mut prebound: Vec<Option<u32>> = Vec::with_capacity(s.expr.items.len());
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    let old = vd.expr.id.0;
                    let new_id = st.next;
                    st.next += 1;
                    st.scopes
                        .last_mut()
                        .expect("at least one scope frame")
                        .insert(old, new_id);
                    prebound.push(Some(new_id));
                } else {
                    prebound.push(None);
                }
            }
            for (item, pre) in s.expr.items.iter_mut().zip(prebound.iter()) {
                if let (Expr::ValDef(vd), Some(new_id)) = (&mut *item, pre) {
                    // Walk RHS in the (now fully pre-bound) surrounding scope.
                    disambig_walk(&mut vd.expr.rhs, st);
                    vd.expr.id = ValId(*new_id);
                } else {
                    disambig_walk(item, st);
                }
            }
            disambig_walk(&mut s.expr.result, st);
            st.scopes.pop();
        }
        Expr::ValDef(s) => {
            // ValDef encountered outside a BlockValue (rare). RHS evaluated
            // in the surrounding scope (no self-binding).
            disambig_walk(&mut s.expr.rhs, st);
            let old = s.expr.id.0;
            let new_id = st.next;
            st.next += 1;
            st.scopes
                .last_mut()
                .expect("at least one scope frame")
                .insert(old, new_id);
            s.expr.id = ValId(new_id);
        }
        Expr::ValUse(vu) => {
            for m in st.scopes.iter().rev() {
                if let Some(&v) = m.get(&vu.val_id.0) {
                    vu.val_id = ValId(v);
                    return;
                }
            }
            // Not found in any scope: leave as-is (malformed input —
            // `sequential_renumber` will surface it via missing-id later).
        }
        Expr::If(i) => {
            disambig_walk(&mut i.condition, st);
            st.scopes.push(HashMap::new());
            disambig_walk(&mut i.true_branch, st);
            st.scopes.pop();
            st.scopes.push(HashMap::new());
            disambig_walk(&mut i.false_branch, st);
            st.scopes.pop();
        }
        Expr::FuncValue(fv) => {
            // FuncArgs introduce ids visible in body; their idx slots are
            // structural (parameter positions), so we keep them as-is
            // (map old→old) — `sequential_renumber`'s FuncValue arm will
            // renumber them based on the enclosing `defId`.
            st.scopes.push(HashMap::new());
            for arg in fv.args() {
                st.scopes.last_mut().unwrap().insert(arg.idx.0, arg.idx.0);
            }
            disambig_walk(fv.body_mut(), st);
            st.scopes.pop();
        }
        // Everything else: structural recurse via Traversable.
        other => {
            for child in other.children_mut() {
                disambig_walk(child, st);
            }
        }
    }
}

fn sequential_renumber(expr: Expr) -> Expr {
    let mut id_map: HashMap<u32, u32> = HashMap::new();
    let mut next_id: u32 = 0; // Scala curId starts at 0; first ValDef gets 0+1=1
    collect_and_assign_ids(&expr, &mut id_map, &mut next_id, None);
    rewrite_ids(expr, &id_map)
}

/// Walk the tree in definition order, assigning IDs matching Scala's scheme.
///
/// `def_id` carries the enclosing ValDef's pre-increment counter value through
/// intermediate nodes to FuncValues, so FuncArg idx aliases with its ValDef id.
fn collect_and_assign_ids(
    expr: &Expr,
    id_map: &mut HashMap<u32, u32>,
    next_id: &mut u32,
    def_id: Option<u32>,
) {
    match expr {
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_and_assign_ids(item, id_map, next_id, None);
            }
            collect_and_assign_ids(&s.expr.result, id_map, next_id, def_id);
        }
        Expr::ValDef(s) => {
            let old_id = s.expr.id.0;
            #[allow(clippy::map_entry)]
            if !id_map.contains_key(&old_id) {
                // Scala: defId = curId (before increment)
                let my_def_id = *next_id;
                // Recurse into RHS, passing def_id so FuncArgs can alias
                collect_and_assign_ids(&s.expr.rhs, id_map, next_id, Some(my_def_id));
                // Scala: curId += 1; ValDef(curId). Reset outer counter — lambda body
                // IDs (which use an independent counter) must not leak outward.
                *next_id = my_def_id + 1;
                id_map.insert(old_id, *next_id);
            } else {
                collect_and_assign_ids(&s.expr.rhs, id_map, next_id, None);
            }
        }
        Expr::FuncValue(fv) => {
            // Scala: varId = defId + 1 — always, whether inside a ValDef or standalone.
            // For standalone lambdas, defId = current curId (= *next_id).
            let func_arg_start = match def_id {
                Some(did) => did + 1,
                None => *next_id + 1, // defId = curId = *next_id; varId = defId + 1
            };
            // Assign FuncArg IDs starting at func_arg_start
            let mut body_id = func_arg_start;
            for arg in fv.args() {
                let old_id = arg.idx.0;
                #[allow(clippy::map_entry)]
                if !id_map.contains_key(&old_id) {
                    id_map.insert(old_id, body_id);
                    body_id += 1;
                }
            }

            // No explicit gap needed: the body starts at varId + 1, and the ValDef
            // case increments before assigning (my_def_id + 1), so the first body
            // ValDef naturally gets varId + 2 with a gap at varId + 1.

            // Lambda body uses INDEPENDENT counter — does not advance outer next_id.
            // (When inside a ValDef, the ValDef case resets next_id after we return.
            //  For standalone lambdas, Scala's curId also stays unchanged after buildValue.)
            collect_and_assign_ids(fv.body(), id_map, &mut body_id, None);
        }
        // All remaining arms propagate def_id transparently
        Expr::BinOp(s) => {
            collect_and_assign_ids(&s.expr.left, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.right, id_map, next_id, def_id);
        }
        Expr::BoolToSigmaProp(bts) => collect_and_assign_ids(&bts.input, id_map, next_id, def_id),
        Expr::If(if_op) => {
            collect_and_assign_ids(&if_op.condition, id_map, next_id, def_id);
            // Scala's IfThenElseLazy: both branches are ThunkDefs processed via
            // processAstGraph with the SAME defId (outer curId). Each branch gets
            // an independent local curId starting from defId, producing overlapping
            // val IDs. The outer curId is unchanged after both branches.
            let branch_start = *next_id;
            collect_and_assign_ids(&if_op.true_branch, id_map, next_id, def_id);
            *next_id = branch_start;
            collect_and_assign_ids(&if_op.false_branch, id_map, next_id, def_id);
            *next_id = branch_start;
        }
        Expr::Filter(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.condition, id_map, next_id, def_id);
        }
        Expr::Exists(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.condition, id_map, next_id, def_id);
        }
        Expr::ForAll(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.condition, id_map, next_id, def_id);
        }
        Expr::Map(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.mapper, id_map, next_id, def_id);
        }
        Expr::Fold(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.zero, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.fold_op, id_map, next_id, def_id);
        }
        Expr::PropertyCall(s) => collect_and_assign_ids(&s.expr.obj, id_map, next_id, def_id),
        Expr::MethodCall(s) => {
            collect_and_assign_ids(&s.expr.obj, id_map, next_id, def_id);
            for a in &s.expr.args {
                collect_and_assign_ids(a, id_map, next_id, def_id);
            }
        }
        Expr::ExtractAmount(ea) => collect_and_assign_ids(&ea.input, id_map, next_id, def_id),
        Expr::ExtractRegisterAs(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id)
        }
        Expr::ExtractScriptBytes(esb) => {
            collect_and_assign_ids(&esb.input, id_map, next_id, def_id)
        }
        Expr::ExtractBytes(eb) => collect_and_assign_ids(&eb.input, id_map, next_id, def_id),
        Expr::ExtractId(ei) => collect_and_assign_ids(&ei.input, id_map, next_id, def_id),
        Expr::ExtractCreationInfo(eci) => {
            collect_and_assign_ids(&eci.input, id_map, next_id, def_id)
        }
        Expr::SizeOf(so) => collect_and_assign_ids(&so.input, id_map, next_id, def_id),
        Expr::ByIndex(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.index, id_map, next_id, def_id);
            if let Some(ref d) = s.expr.default {
                collect_and_assign_ids(d, id_map, next_id, def_id);
            }
        }
        Expr::SelectField(s) => collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id),
        Expr::OptionGet(s) => collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id),
        Expr::OptionIsDefined(s) => collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id),
        Expr::OptionGetOrElse(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.default, id_map, next_id, def_id);
        }
        Expr::Slice(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.from, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.until, id_map, next_id, def_id);
        }
        Expr::LogicalNot(s) => collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id),
        Expr::Negation(s) => collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id),
        Expr::SigmaPropBytes(spb) => collect_and_assign_ids(&spb.input, id_map, next_id, def_id),
        Expr::SigmaPropIsProven(sip) => collect_and_assign_ids(&sip.input, id_map, next_id, def_id),
        Expr::ZkProofBlock(zk) => collect_and_assign_ids(&zk.input, id_map, next_id, def_id),
        Expr::Upcast(uc) => collect_and_assign_ids(&uc.input, id_map, next_id, def_id),
        Expr::Downcast(dc) => collect_and_assign_ids(&dc.input, id_map, next_id, def_id),
        Expr::CalcBlake2b256(cb) => collect_and_assign_ids(&cb.input, id_map, next_id, def_id),
        Expr::CreateProveDlog(cpd) => collect_and_assign_ids(&cpd.input, id_map, next_id, def_id),
        Expr::SigmaAnd(sa) => {
            for i in sa.items.iter() {
                collect_and_assign_ids(i, id_map, next_id, def_id);
            }
        }
        Expr::SigmaOr(so) => {
            for i in so.items.iter() {
                collect_and_assign_ids(i, id_map, next_id, def_id);
            }
        }
        Expr::Tuple(t) => {
            for i in t.items.iter() {
                collect_and_assign_ids(i, id_map, next_id, def_id);
            }
        }
        Expr::TreeLookup(s) => {
            collect_and_assign_ids(&s.expr.tree, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.key, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.proof, id_map, next_id, def_id);
        }
        Expr::Apply(app) => {
            collect_and_assign_ids(&app.func, id_map, next_id, def_id);
            for a in &app.args {
                collect_and_assign_ids(a, id_map, next_id, def_id);
            }
        }
        Expr::Atleast(s) => {
            collect_and_assign_ids(&s.bound, id_map, next_id, def_id);
            collect_and_assign_ids(&s.input, id_map, next_id, def_id);
        }
        Expr::And(a) => collect_and_assign_ids(&a.expr.input, id_map, next_id, def_id),
        Expr::Or(o) => collect_and_assign_ids(&o.expr.input, id_map, next_id, def_id),
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                collect_and_assign_ids(item, id_map, next_id, def_id);
            }
        }
        Expr::Collection(_) => {}
        // Single-input wrappers (non-Spanned)
        Expr::LongToByteArray(s) => collect_and_assign_ids(&s.input, id_map, next_id, def_id),
        Expr::DecodePoint(s) => collect_and_assign_ids(&s.input, id_map, next_id, def_id),
        Expr::ExtractBytesWithNoRef(s) => collect_and_assign_ids(&s.input, id_map, next_id, def_id),
        Expr::CalcSha256(s) => collect_and_assign_ids(&s.input, id_map, next_id, def_id),
        Expr::BitInversion(s) => collect_and_assign_ids(&s.input, id_map, next_id, def_id),
        Expr::XorOf(s) => collect_and_assign_ids(&s.input, id_map, next_id, def_id),
        // Single-input wrappers (Spanned)
        Expr::ByteArrayToLong(s) => collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id),
        Expr::ByteArrayToBigInt(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id)
        }
        // Multi-child nodes (Spanned)
        Expr::Append(s) => {
            collect_and_assign_ids(&s.expr.input, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.col_2, id_map, next_id, def_id);
        }
        Expr::SubstConstants(s) => {
            collect_and_assign_ids(&s.expr.script_bytes, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.positions, id_map, next_id, def_id);
            collect_and_assign_ids(&s.expr.new_values, id_map, next_id, def_id);
        }
        // Multi-child nodes (non-Spanned)
        Expr::Xor(s) => {
            collect_and_assign_ids(&s.left, id_map, next_id, def_id);
            collect_and_assign_ids(&s.right, id_map, next_id, def_id);
        }
        Expr::CreateProveDhTuple(s) => {
            collect_and_assign_ids(&s.g, id_map, next_id, def_id);
            collect_and_assign_ids(&s.h, id_map, next_id, def_id);
            collect_and_assign_ids(&s.u, id_map, next_id, def_id);
            collect_and_assign_ids(&s.v, id_map, next_id, def_id);
        }
        Expr::CreateAvlTree(s) => {
            collect_and_assign_ids(&s.flags, id_map, next_id, def_id);
            collect_and_assign_ids(&s.digest, id_map, next_id, def_id);
            collect_and_assign_ids(&s.key_length, id_map, next_id, def_id);
            if let Some(ref vl) = s.value_length {
                collect_and_assign_ids(vl, id_map, next_id, def_id);
            }
        }
        Expr::MultiplyGroup(s) => {
            collect_and_assign_ids(&s.left, id_map, next_id, def_id);
            collect_and_assign_ids(&s.right, id_map, next_id, def_id);
        }
        Expr::Exponentiate(s) => {
            collect_and_assign_ids(&s.left, id_map, next_id, def_id);
            collect_and_assign_ids(&s.right, id_map, next_id, def_id);
        }
        // Optional child only
        Expr::DeserializeRegister(s) => {
            if let Some(ref d) = s.default {
                collect_and_assign_ids(d, id_map, next_id, def_id);
            }
        }
        // True leaves — no child Expr fields
        Expr::Const(_)
        | Expr::ConstPlaceholder(_)
        | Expr::GlobalVars(_)
        | Expr::ValUse(_)
        | Expr::Context
        | Expr::Global
        | Expr::GetVar(_)
        | Expr::DeserializeContext(_) => {}
    }
}

/// Inline ValDefs whose RHS is a Const at all use sites (even multi-use).
/// The Scala compiler never keeps a ValDef for a constant — it inlines directly.
#[allow(dead_code)]
fn inline_const_vals(expr: Expr) -> Expr {
    match expr {
        Expr::BlockValue(s) => {
            // Collect const-valued ValDefs
            let mut const_map: HashMap<u32, Expr> = HashMap::new();
            let mut new_items: Vec<Expr> = Vec::new();

            for item in s.expr.items {
                if let Expr::ValDef(ref vd) = item {
                    if matches!(*vd.expr.rhs, Expr::Const(_)) {
                        const_map.insert(vd.expr.id.0, *vd.expr.rhs.clone());
                        continue; // Remove this ValDef
                    }
                }
                new_items.push(item);
            }

            if const_map.is_empty() {
                return Expr::BlockValue(Spanned {
                    source_span: s.source_span,
                    expr: BlockValue {
                        items: new_items,
                        result: s.expr.result,
                    },
                });
            }

            // Build id_map for rewrite_ids: const val IDs map to themselves
            // But we need to replace ValUse with the actual Const, not just remap IDs.
            // Use replace_all for each const val.
            let mut result_items = new_items;
            let mut result_expr = *s.expr.result;

            for (val_id, const_expr) in &const_map {
                let val_use = Expr::ValUse(ValUse {
                    val_id: ValId(*val_id),
                    tpe: const_expr.tpe(),
                });
                // Replace in all remaining items
                result_items = result_items
                    .into_iter()
                    .map(|item| replace_all(&item, &val_use, const_expr))
                    .collect();
                // Replace in result
                result_expr = replace_all(&result_expr, &val_use, const_expr);
            }

            // Recurse into remaining items (they may contain nested blocks)
            let result_items: Vec<Expr> = result_items.into_iter().map(inline_const_vals).collect();
            let result_expr = inline_const_vals(result_expr);

            if result_items.is_empty() {
                result_expr
            } else {
                Expr::BlockValue(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: BlockValue {
                        items: result_items,
                        result: result_expr.into(),
                    },
                })
            }
        }
        // For non-block expressions, just return as-is
        // (const vals only appear in BlockValue items)
        other => other,
    }
}

/// Apply CSE to an expression, recursing into lambda bodies.
/// `global_max_id` is the highest ValId used in the entire outer tree,
/// ensuring CSE-introduced ValIds don't conflict across scopes.
fn cse_expr(expr: Expr, global_max_id: u32, is_lambda_scope: bool) -> Expr {
    // First, recurse into lambda bodies (at any nesting depth)
    let mut running_max = global_max_id;
    let expr = process_lambdas(expr, &mut running_max);

    // Detect whether the expression contains FuncValue (lambdas from
    // filter/fold/exists/forall). The Scala compiler's flatSchedule counting
    // changes effective usage counts when lambdas are present.
    let has_lambdas = !is_lambda_scope && contains_func_value(&expr);

    let local_max = find_max_val_id(&expr);

    if is_lambda_scope {
        // Lambda scope: re-collect candidates after each extraction (mirrors
        // Scala's `processAstGraph` iterative graph rewrite — `processAstGraph`
        // re-walks the graph after each `lift` so newly-emerging hash-cons
        // candidates that only appear AFTER a parent extraction are picked up).
        // Tiebreak on equal savings prefers the SHALLOWER candidate.
        //
        // sig-15 oracle_refresh fold lambda exhibits this: `t._1` (depth 1,
        // count 3, savings 2) and `t._1._2` (depth 2, count 2, savings 2)
        // tie. NODE picks `t._1` first; after that `SelectField(2,ValUse(t1))`
        // emerges as a fresh candidate (count 2, depth 1) and is extracted
        // next. The pre-fix one-shot sort+iterate picks `t._1._2` first by
        // HashMap iteration luck, dropping `t._1`'s tree count to 1 and
        // missing the second extraction.
        //
        // Lambda bodies use +2 gap from max ID (Scala compiler convention).
        let mut next_id = local_max.max(global_max_id) + 2;
        let mut result = expr;
        let mut new_val_defs: Vec<Expr> = Vec::new();
        loop {
            let mut all_subexprs: Vec<Expr> = Vec::new();
            collect_subexprs(&result, &mut all_subexprs);
            let dag_usages = count_dag_usages(&result);
            let mut best: Option<(Expr, i32, usize)> = None;
            for (sub, dag_count) in &dag_usages {
                if *dag_count < 2 {
                    continue;
                }
                if !is_collectible(sub) {
                    continue;
                }
                let tree_count = all_subexprs.iter().filter(|s| *s == sub).count();
                if tree_count < 2 {
                    continue;
                }
                let depth = expr_depth(sub);
                let savings = (tree_count as i32 - 1) * depth as i32;
                let better = match &best {
                    None => true,
                    Some((_, best_sav, best_depth)) => {
                        savings > *best_sav
                            || (savings == *best_sav && depth < *best_depth)
                    }
                };
                if better {
                    best = Some((sub.clone(), savings, depth));
                }
            }
            let Some((candidate, _, _)) = best else { break };
            let val_id = next_id;
            next_id += 1;
            let tpe = expr_type(&candidate);
            result = replace_all(
                &result,
                &candidate,
                &Expr::ValUse(ValUse {
                    val_id: ValId(val_id),
                    tpe,
                }),
            );
            new_val_defs.push(Expr::ValDef(Spanned {
                source_span: SourceSpan::empty(),
                expr: ValDef {
                    id: ValId(val_id),
                    rhs: candidate.into(),
                },
            }));
        }

        if new_val_defs.is_empty() {
            return result;
        }
        let wrapped = match result {
            Expr::BlockValue(spanned) => {
                let mut items = new_val_defs;
                items.extend(spanned.expr.items);
                Expr::BlockValue(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: BlockValue {
                        items,
                        result: spanned.expr.result,
                    },
                })
            }
            other => Expr::BlockValue(Spanned {
                source_span: SourceSpan::empty(),
                expr: BlockValue {
                    items: new_val_defs,
                    result: other.into(),
                },
            }),
        };
        // sig-15 oracle_refresh: the outer `inline_single_use_vals` and
        // `reorder_valdefs` passes don't recurse into lambda bodies (their
        // driver, `map_children`, intentionally does not cross FuncValue
        // boundaries — that's owned by `process_lambdas`). Source-level vals
        // like `wasSorted`/`oldSum` that survive HIR
        // `has_shared_field_in_clean_scope` (because they share `t._2`
        // access) get pinned as MIR ValDefs and are never inlined despite
        // single use; and CSE-introduced ValDefs end up prepended in
        // emission order rather than DFS-walk order. Scala does both
        // implicitly via TreeBuilding's single graph traversal. Apply
        // inline+reorder here so the lambda-body shape and ID order match
        // NODE.
        reorder_valdefs(inline_single_use_vals(wrapped))
    } else if has_lambdas {
        // Top-level with lambdas: use the old savings-based approach.
        // The Scala compiler's flatSchedule (which includes lambda body nodes)
        // changes effective usage counts, so we block ValUse-containing
        // expressions to match. The graph IR approach over-extracts here
        // because it doesn't model the flatSchedule interaction.
        //
        // Carve-out (sig-15 oracle_refresh R2): the blanket `contains_val_use`
        // block over-rejects structural unary ops on an outer-bound ValUse.
        // Scala's graph IR hash-conses `SelectField(ValUse, n)` and
        // `SizeOf(ValUse)` (their sym depends only on the ValUse, which is
        // root-bound), and `mainG.hasManyUsagesGlobal` counts cross-ThunkDef
        // uses for these — so they get extracted at root regardless of where
        // the uses physically sit. PropertyCall / Extract* / etc. are NOT
        // hash-consed identically (method-call symbols carry method-specific
        // identity), so we keep those blocked. Companion to `168acaf2`'s
        // SelectField-on-ValUse carve-out in `process_ast_graph_impl`.
        //
        // Restricted further to count_in_lambda_bodies == 0: when a candidate
        // also has lambda-body occurrences, the flatSchedule semantics differ
        // and the original block applies.
        let mut all_subexprs: Vec<Expr> = Vec::new();
        collect_subexprs(&expr, &mut all_subexprs);
        let dag_usages = count_dag_usages(&expr);
        let mut candidates: Vec<(Expr, usize)> = Vec::new();
        for (sub, dag_count) in &dag_usages {
            if *dag_count < 2 {
                continue;
            }
            if !is_collectible(sub) {
                continue;
            }
            if contains_val_use(sub) {
                let is_root_bound_unary = matches!(sub,
                    Expr::SelectField(s) if matches!(&*s.expr.input, Expr::ValUse(_))
                ) || matches!(sub,
                    Expr::SizeOf(s) if matches!(&*s.input, Expr::ValUse(_))
                );
                if !is_root_bound_unary || count_in_lambda_bodies(&expr, sub) > 0 {
                    continue;
                }
            }
            let tree_count = all_subexprs.iter().filter(|s| *s == sub).count();
            if tree_count >= 2 {
                candidates.push((sub.clone(), tree_count));
            }
        }
        if candidates.is_empty() {
            return expr;
        }
        candidates.sort_by_key(|(c, count)| {
            let depth = expr_depth(c);
            let savings = (*count as i32 - 1) * depth as i32;
            std::cmp::Reverse(savings)
        });
        let mut next_id = local_max.max(global_max_id) + 1;
        let mut result = expr;
        let mut new_val_defs: Vec<Expr> = Vec::new();
        for (candidate, _count) in &candidates {
            let current_count = count_occurrences(&result, candidate);
            if current_count < 2 {
                continue;
            }
            let val_id = next_id;
            next_id += 1;
            let tpe = expr_type(candidate);
            result = replace_all(
                &result,
                candidate,
                &Expr::ValUse(ValUse {
                    val_id: ValId(val_id),
                    tpe,
                }),
            );
            new_val_defs.push(Expr::ValDef(Spanned {
                source_span: SourceSpan::empty(),
                expr: ValDef {
                    id: ValId(val_id),
                    rhs: candidate.clone().into(),
                },
            }));
        }
        if new_val_defs.is_empty() {
            return result;
        }
        let (new_val_defs, result, _next_id) =
            lambda_rescue_post_pass(new_val_defs, result, next_id);
        match result {
            Expr::BlockValue(spanned) => {
                let mut items = new_val_defs;
                items.extend(spanned.expr.items);
                let items = topo_order_valdefs(items);
                Expr::BlockValue(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: BlockValue {
                        items,
                        result: spanned.expr.result,
                    },
                })
            }
            other => Expr::BlockValue(Spanned {
                source_span: SourceSpan::empty(),
                expr: BlockValue {
                    items: topo_order_valdefs(new_val_defs),
                    result: other.into(),
                },
            }),
        }
    } else {
        // Top-level without lambdas: use graph IR approach (processAstGraph port).
        process_ast_graph(expr, global_max_id)
    }
}

/// Recursively find and apply CSE inside lambda bodies at any nesting depth.
/// Uses a mutable global_max_id so sibling lambdas get unique CSE temp IDs.
fn process_lambdas(expr: Expr, global_max_id: &mut u32) -> Expr {
    match expr {
        Expr::FuncValue(fv) => {
            // Apply CSE to the lambda body as an independent scope
            let body = cse_expr(fv.body().clone(), *global_max_id, true);
            // Update global_max_id so the NEXT sibling lambda gets unique temp IDs
            let body_max = find_max_val_id(&body);
            if body_max > *global_max_id {
                *global_max_id = body_max;
            }
            Expr::FuncValue(FuncValue::new(fv.args().to_vec(), body))
        }
        // For all other types, recurse into children looking for lambdas
        other => map_children_with_id_mut(other, global_max_id, process_lambdas),
    }
}

/// Apply a function to all child expressions, threading a mutable global_max_id.
fn map_children_with_id_mut(expr: Expr, gid: &mut u32, f: fn(Expr, &mut u32) -> Expr) -> Expr {
    match expr {
        Expr::BinOp(s) => Expr::BinOp(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::bin_op::BinOp {
                kind: s.expr.kind,
                left: f(*s.expr.left, gid).into(),
                right: f(*s.expr.right, gid).into(),
            },
        }),
        Expr::BlockValue(s) => {
            let items: Vec<Expr> = s.expr.items.into_iter().map(|i| f(i, gid)).collect();
            let result = f(*s.expr.result, gid);
            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items,
                    result: result.into(),
                },
            })
        }
        Expr::ValDef(s) => Expr::ValDef(Spanned {
            source_span: s.source_span,
            expr: ValDef {
                id: s.expr.id,
                rhs: f(*s.expr.rhs, gid).into(),
            },
        }),
        Expr::BoolToSigmaProp(bts) => {
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: f(*bts.input, gid).into(),
            })
        }
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: f(*if_op.condition, gid).into(),
            true_branch: f(*if_op.true_branch, gid).into(),
            false_branch: f(*if_op.false_branch, gid).into(),
        }),
        Expr::Filter(s) => Expr::Filter(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_filter::Filter {
                input: f(*s.expr.input, gid).into(),
                condition: f(*s.expr.condition, gid).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::Exists(s) => Expr::Exists(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_exists::Exists {
                input: f(*s.expr.input, gid).into(),
                condition: f(*s.expr.condition, gid).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::ForAll(s) => Expr::ForAll(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_forall::ForAll {
                input: f(*s.expr.input, gid).into(),
                condition: f(*s.expr.condition, gid).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::SizeOf(so) => Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
            input: f(*so.input, gid).into(),
        }),
        Expr::ExtractAmount(ea) => {
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: f(*ea.input, gid).into(),
            })
        }
        Expr::PropertyCall(s) => Expr::PropertyCall(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::property_call::PropertyCall {
                obj: f(*s.expr.obj, gid).into(),
                method: s.expr.method,
            },
        }),
        // Slice can wrap a Filter (`coll.filter(...).slice(...)`); without
        // descending into Slice's children, `process_lambdas` never reaches
        // the inner FuncValue and the Filter lambda body is left un-CSE'd.
        // Closes rosen_event_trigger 1B (`box.tokens` 2× inside Filter body).
        Expr::Slice(s) => Expr::Slice(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_slice::Slice {
                input: f(*s.expr.input, gid).into(),
                from: f(*s.expr.from, gid).into(),
                until: f(*s.expr.until, gid).into(),
            },
        }),
        // Fold's `fold_op` is the FuncValue; without descending here,
        // `process_lambdas` never reaches the lambda body and inner-scope
        // CSE on the accumulator-tuple selectors is skipped. Closes
        // oracle_refresh shared `t` / `t._2` selector chain inside the
        // fold lambda body.
        Expr::Fold(s) => Expr::Fold(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_fold::Fold {
                input: f(*s.expr.input, gid).into(),
                zero: f(*s.expr.zero, gid).into(),
                fold_op: f(*s.expr.fold_op, gid).into(),
            },
        }),
        Expr::SigmaAnd(sa) => {
            let items: Vec<Expr> = sa.items.into_iter().map(|i| f(i, gid)).collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: items.try_into().expect("SigmaAnd >= 2"),
            })
        }
        Expr::SigmaOr(so) => {
            let items: Vec<Expr> = so.items.into_iter().map(|i| f(i, gid)).collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: items.try_into().expect("SigmaOr >= 2"),
            })
        }
        other => other,
    }
}

/// Apply a function to all child expressions, threading through global_max_id.
#[allow(dead_code)]
fn map_children_with_id(expr: Expr, gid: u32, f: fn(Expr, u32) -> Expr) -> Expr {
    // Delegate to map_children with a closure that captures gid
    // Unfortunately map_children takes fn(Expr)->Expr, so we need inline matching here.
    // For efficiency, just call map_children since most nodes don't contain lambdas,
    // and process_lambdas only transforms FuncValue. We can wrap in a closure:
    // Actually, fn pointers can't capture, so let's just use the same match:
    match expr {
        Expr::BinOp(s) => Expr::BinOp(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::bin_op::BinOp {
                kind: s.expr.kind,
                left: f(*s.expr.left, gid).into(),
                right: f(*s.expr.right, gid).into(),
            },
        }),
        Expr::BlockValue(s) => Expr::BlockValue(Spanned {
            source_span: s.source_span,
            expr: BlockValue {
                items: s.expr.items.into_iter().map(|i| f(i, gid)).collect(),
                result: f(*s.expr.result, gid).into(),
            },
        }),
        Expr::ValDef(s) => Expr::ValDef(Spanned {
            source_span: s.source_span,
            expr: ValDef {
                id: s.expr.id,
                rhs: f(*s.expr.rhs, gid).into(),
            },
        }),
        Expr::BoolToSigmaProp(bts) => {
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: f(*bts.input, gid).into(),
            })
        }
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: f(*if_op.condition, gid).into(),
            true_branch: f(*if_op.true_branch, gid).into(),
            false_branch: f(*if_op.false_branch, gid).into(),
        }),
        Expr::Filter(s) => Expr::Filter(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_filter::Filter {
                input: f(*s.expr.input, gid).into(),
                condition: f(*s.expr.condition, gid).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::Exists(s) => Expr::Exists(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_exists::Exists {
                input: f(*s.expr.input, gid).into(),
                condition: f(*s.expr.condition, gid).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::ForAll(s) => Expr::ForAll(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_forall::ForAll {
                input: f(*s.expr.input, gid).into(),
                condition: f(*s.expr.condition, gid).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::SizeOf(so) => Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
            input: f(*so.input, gid).into(),
        }),
        Expr::ExtractAmount(ea) => {
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: f(*ea.input, gid).into(),
            })
        }
        Expr::PropertyCall(s) => Expr::PropertyCall(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::property_call::PropertyCall {
                obj: f(*s.expr.obj, gid).into(),
                method: s.expr.method,
            },
        }),
        Expr::SigmaAnd(sa) => {
            let items: Vec<Expr> = sa.items.into_iter().map(|i| f(i, gid)).collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: items.try_into().expect("SigmaAnd >= 2"),
            })
        }
        Expr::SigmaOr(so) => {
            let items: Vec<Expr> = so.items.into_iter().map(|i| f(i, gid)).collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: items.try_into().expect("SigmaOr >= 2"),
            })
        }
        // And/Or wrap a Collection of Boolean expressions; Collection holds
        // individual items. Both can contain nested If nodes that
        // `apply_cse_within_branches` must reach to process their branches.
        Expr::And(a) => Expr::And(ergotree_ir::source_span::Spanned {
            source_span: a.source_span,
            expr: ergotree_ir::mir::and::And {
                input: f(*a.expr.input, gid).into(),
            },
        }),
        Expr::Or(o) => Expr::Or(ergotree_ir::source_span::Spanned {
            source_span: o.source_span,
            expr: ergotree_ir::mir::or::Or {
                input: f(*o.expr.input, gid).into(),
            },
        }),
        Expr::Collection(c) => match c {
            ergotree_ir::mir::collection::Collection::Exprs { elem_tpe, items } => {
                let new_items: Vec<Expr> = items.into_iter().map(|i| f(i, gid)).collect();
                Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs {
                    elem_tpe,
                    items: new_items,
                })
            }
            other => Expr::Collection(other),
        },
        Expr::Atleast(s) => ergotree_ir::mir::atleast::Atleast::new(f(*s.bound, gid), f(*s.input, gid))
            .map(Expr::Atleast)
            .expect("Atleast::new in map_children_with_id"),
        // Other types: pass through (no child Expr fields containing nested Ifs)
        other => other,
    }
}

/// Apply a function to all child expressions of an Expr node.
#[allow(dead_code)]
fn map_children(expr: Expr, f: fn(Expr) -> Expr) -> Expr {
    match expr {
        Expr::BinOp(s) => Expr::BinOp(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::bin_op::BinOp {
                kind: s.expr.kind,
                left: f(*s.expr.left).into(),
                right: f(*s.expr.right).into(),
            },
        }),
        Expr::BlockValue(s) => Expr::BlockValue(Spanned {
            source_span: s.source_span,
            expr: BlockValue {
                items: s.expr.items.into_iter().map(f).collect(),
                result: f(*s.expr.result).into(),
            },
        }),
        Expr::ValDef(s) => Expr::ValDef(Spanned {
            source_span: s.source_span,
            expr: ValDef {
                id: s.expr.id,
                rhs: f(*s.expr.rhs).into(),
            },
        }),
        Expr::BoolToSigmaProp(bts) => {
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: f(*bts.input).into(),
            })
        }
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: f(*if_op.condition).into(),
            true_branch: f(*if_op.true_branch).into(),
            false_branch: f(*if_op.false_branch).into(),
        }),
        Expr::PropertyCall(s) => Expr::PropertyCall(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::property_call::PropertyCall {
                obj: f(*s.expr.obj).into(),
                method: s.expr.method,
            },
        }),
        Expr::ExtractAmount(ea) => {
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: f(*ea.input).into(),
            })
        }
        Expr::ExtractRegisterAs(s) => Expr::ExtractRegisterAs(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::extract_reg_as::ExtractRegisterAs {
                input: f(*s.expr.input).into(),
                register_id: s.expr.register_id,
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::ExtractScriptBytes(esb) => {
            Expr::ExtractScriptBytes(ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes {
                input: f(*esb.input).into(),
            })
        }
        Expr::ExtractBytes(eb) => {
            Expr::ExtractBytes(ergotree_ir::mir::extract_bytes::ExtractBytes {
                input: f(*eb.input).into(),
            })
        }
        Expr::ExtractId(ei) => Expr::ExtractId(ergotree_ir::mir::extract_id::ExtractId {
            input: f(*ei.input).into(),
        }),
        Expr::ExtractCreationInfo(eci) => Expr::ExtractCreationInfo(
            ergotree_ir::mir::extract_creation_info::ExtractCreationInfo {
                input: f(*eci.input).into(),
            },
        ),
        Expr::SizeOf(so) => Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
            input: f(*so.input).into(),
        }),
        Expr::LogicalNot(s) => Expr::LogicalNot(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::logical_not::LogicalNot {
                input: f(*s.expr.input).into(),
            },
        }),
        Expr::Negation(s) => Expr::Negation(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::negation::Negation {
                input: f(*s.expr.input).into(),
            },
        }),
        Expr::SigmaPropBytes(spb) => {
            Expr::SigmaPropBytes(ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes {
                input: f(*spb.input).into(),
            })
        }
        Expr::Upcast(uc) => Expr::Upcast(ergotree_ir::mir::upcast::Upcast {
            input: f(*uc.input).into(),
            tpe: uc.tpe,
        }),
        Expr::CalcBlake2b256(cb) => {
            Expr::CalcBlake2b256(ergotree_ir::mir::calc_blake2b256::CalcBlake2b256 {
                input: f(*cb.input).into(),
            })
        }
        Expr::Filter(s) => Expr::Filter(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_filter::Filter {
                input: f(*s.expr.input).into(),
                condition: f(*s.expr.condition).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::Exists(s) => Expr::Exists(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_exists::Exists {
                input: f(*s.expr.input).into(),
                condition: f(*s.expr.condition).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::ForAll(s) => Expr::ForAll(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_forall::ForAll {
                input: f(*s.expr.input).into(),
                condition: f(*s.expr.condition).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::Map(s) => {
            let input = f(*s.expr.input);
            let mapper = f(*s.expr.mapper);
            ergotree_ir::mir::coll_map::Map::new(input, mapper)
                .map(|m| {
                    Expr::Map(Spanned {
                        source_span: SourceSpan::empty(),
                        expr: m,
                    })
                })
                .expect("Map::new in map_children")
        }
        Expr::Fold(s) => {
            let input = f(*s.expr.input);
            let zero = f(*s.expr.zero);
            let fold_op = f(*s.expr.fold_op);
            ergotree_ir::mir::coll_fold::Fold::new(input, zero, fold_op)
                .map(|fl| {
                    Expr::Fold(Spanned {
                        source_span: SourceSpan::empty(),
                        expr: fl,
                    })
                })
                .expect("Fold::new in map_children")
        }
        Expr::Slice(s) => Expr::Slice(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_slice::Slice {
                input: f(*s.expr.input).into(),
                from: f(*s.expr.from).into(),
                until: f(*s.expr.until).into(),
            },
        }),
        Expr::ByIndex(s) => {
            let input = f(*s.expr.input);
            let index = f(*s.expr.index);
            let default = s.expr.default.map(|d| Box::new(f(*d)));
            ergotree_ir::mir::coll_by_index::ByIndex::new(input, index, default)
                .map(|bi| {
                    Expr::ByIndex(Spanned {
                        source_span: s.source_span,
                        expr: bi,
                    })
                })
                .expect("ByIndex::new in map_children")
        }
        Expr::SelectField(s) => {
            let input = f(*s.expr.input);
            ergotree_ir::mir::select_field::SelectField::new(input, s.expr.field_index)
                .map(|sf| {
                    Expr::SelectField(Spanned {
                        source_span: s.source_span,
                        expr: sf,
                    })
                })
                .expect("SelectField::new in map_children")
        }
        Expr::OptionGet(s) => {
            let input = f(*s.expr.input);
            ergotree_ir::mir::option_get::OptionGet::try_build(input)
                .map(|og| {
                    Expr::OptionGet(Spanned {
                        source_span: s.source_span,
                        expr: og,
                    })
                })
                .expect("OptionGet::try_build in map_children")
        }
        Expr::OptionIsDefined(s) => {
            let input = f(*s.expr.input);
            ergotree_ir::mir::option_is_defined::OptionIsDefined::try_build(input)
                .map(|oid| {
                    Expr::OptionIsDefined(Spanned {
                        source_span: s.source_span,
                        expr: oid,
                    })
                })
                .expect("OptionIsDefined in map_children")
        }
        Expr::OptionGetOrElse(s) => {
            let input = f(*s.expr.input);
            let default = f(*s.expr.default);
            ergotree_ir::mir::option_get_or_else::OptionGetOrElse::new(input, default)
                .map(|oge| {
                    Expr::OptionGetOrElse(Spanned {
                        source_span: s.source_span,
                        expr: oge,
                    })
                })
                .expect("OptionGetOrElse in map_children")
        }
        Expr::MethodCall(s) => {
            let obj = f(*s.expr.obj);
            let args: Vec<Expr> = s.expr.args.into_iter().map(f).collect();
            ergotree_ir::mir::method_call::MethodCall::with_type_args(
                obj,
                s.expr.method,
                args,
                s.expr.explicit_type_args,
            )
            .map(|mc| {
                Expr::MethodCall(Spanned {
                    source_span: s.source_span,
                    expr: mc,
                })
            })
            .expect("MethodCall::with_type_args in map_children")
        }
        Expr::SigmaAnd(sa) => {
            let items: Vec<Expr> = sa.items.into_iter().map(f).collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: items.try_into().expect("SigmaAnd should have >= 2"),
            })
        }
        Expr::SigmaOr(so) => {
            let items: Vec<Expr> = so.items.into_iter().map(f).collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: items.try_into().expect("SigmaOr should have >= 2"),
            })
        }
        Expr::TreeLookup(s) => Expr::TreeLookup(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::tree_lookup::TreeLookup {
                tree: f(*s.expr.tree).into(),
                key: f(*s.expr.key).into(),
                proof: f(*s.expr.proof).into(),
            },
        }),
        Expr::Tuple(t) => {
            let items: Vec<Expr> = t.items.into_iter().map(f).collect();
            Expr::Tuple(ergotree_ir::mir::tuple::Tuple::new(items).expect("valid tuple"))
        }
        Expr::Apply(app) => {
            let func = f(*app.func);
            let args: Vec<Expr> = app.args.into_iter().map(f).collect();
            ergotree_ir::mir::apply::Apply::new(func, args)
                .map(Expr::Apply)
                .expect("Apply::new in map_children")
        }
        Expr::And(a) => Expr::And(ergotree_ir::source_span::Spanned {
            source_span: a.source_span,
            expr: ergotree_ir::mir::and::And {
                input: f(*a.expr.input).into(),
            },
        }),
        Expr::Or(o) => Expr::Or(ergotree_ir::source_span::Spanned {
            source_span: o.source_span,
            expr: ergotree_ir::mir::or::Or {
                input: f(*o.expr.input).into(),
            },
        }),
        Expr::Collection(c) => match c {
            ergotree_ir::mir::collection::Collection::Exprs { elem_tpe, items } => {
                let new_items: Vec<Expr> = items.into_iter().map(f).collect();
                Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs {
                    elem_tpe,
                    items: new_items,
                })
            }
            other => Expr::Collection(other),
        },
        // ---- Variants added during 2026-04 audit (previously fell through
        // to `other => other`, silently skipping their child Expr fields). ----
        Expr::CreateProveDlog(cpd) => {
            ergotree_ir::mir::create_provedlog::CreateProveDlog::try_build(f(*cpd.input))
                .map(Expr::CreateProveDlog)
                .expect("CreateProveDlog::try_build in map_children")
        }
        Expr::CreateProveDhTuple(s) => {
            ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple::new(
                f(*s.g),
                f(*s.h),
                f(*s.u),
                f(*s.v),
            )
            .map(Expr::CreateProveDhTuple)
            .expect("CreateProveDhTuple::new in map_children")
        }
        Expr::CreateAvlTree(s) => ergotree_ir::mir::create_avl_tree::CreateAvlTree::new(
            f(*s.flags),
            f(*s.digest),
            f(*s.key_length),
            s.value_length.map(|vl| Box::new(f(*vl))),
        )
        .map(Expr::CreateAvlTree)
        .expect("CreateAvlTree::new in map_children"),
        Expr::Atleast(s) => ergotree_ir::mir::atleast::Atleast::new(f(*s.bound), f(*s.input))
            .map(Expr::Atleast)
            .expect("Atleast::new in map_children"),
        Expr::Append(s) => {
            ergotree_ir::mir::coll_append::Append::new(f(*s.expr.input), f(*s.expr.col_2))
                .map(|v| {
                    Expr::Append(Spanned {
                        source_span: s.source_span,
                        expr: v,
                    })
                })
                .expect("Append::new in map_children")
        }
        Expr::SubstConstants(s) => ergotree_ir::mir::subst_const::SubstConstants::new(
            f(*s.expr.script_bytes),
            f(*s.expr.positions),
            f(*s.expr.new_values),
        )
        .map(|v| {
            Expr::SubstConstants(Spanned {
                source_span: s.source_span,
                expr: v,
            })
        })
        .expect("SubstConstants::new in map_children"),
        Expr::Xor(s) => ergotree_ir::mir::xor::Xor::new(f(*s.left), f(*s.right))
            .map(Expr::Xor)
            .expect("Xor::new in map_children"),
        Expr::MultiplyGroup(s) => {
            ergotree_ir::mir::multiply_group::MultiplyGroup::new(f(*s.left), f(*s.right))
                .map(Expr::MultiplyGroup)
                .expect("MultiplyGroup::new in map_children")
        }
        Expr::Exponentiate(s) => {
            ergotree_ir::mir::exponentiate::Exponentiate::new(f(*s.left), f(*s.right))
                .map(Expr::Exponentiate)
                .expect("Exponentiate::new in map_children")
        }
        Expr::Downcast(dc) => Expr::Downcast(ergotree_ir::mir::downcast::Downcast {
            input: f(*dc.input).into(),
            tpe: dc.tpe,
        }),
        Expr::DeserializeRegister(s) => Expr::DeserializeRegister(
            ergotree_ir::mir::deserialize_register::DeserializeRegister {
                reg: s.reg,
                tpe: s.tpe,
                default: s.default.map(|d| Box::new(f(*d))),
            },
        ),
        // Single-input wrappers (OneArgOpTryBuild, non-Spanned)
        Expr::LongToByteArray(s) => {
            ergotree_ir::mir::long_to_byte_array::LongToByteArray::try_build(f(*s.input))
                .map(Expr::LongToByteArray)
                .expect("LongToByteArray in map_children")
        }
        Expr::DecodePoint(s) => ergotree_ir::mir::decode_point::DecodePoint::try_build(f(*s.input))
            .map(Expr::DecodePoint)
            .expect("DecodePoint in map_children"),
        Expr::ExtractBytesWithNoRef(s) => {
            ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef::try_build(f(
                *s.input
            ))
            .map(Expr::ExtractBytesWithNoRef)
            .expect("ExtractBytesWithNoRef in map_children")
        }
        Expr::CalcSha256(s) => ergotree_ir::mir::calc_sha256::CalcSha256::try_build(f(*s.input))
            .map(Expr::CalcSha256)
            .expect("CalcSha256 in map_children"),
        Expr::BitInversion(s) => {
            ergotree_ir::mir::bit_inversion::BitInversion::try_build(f(*s.input))
                .map(Expr::BitInversion)
                .expect("BitInversion in map_children")
        }
        Expr::XorOf(s) => Expr::XorOf(ergotree_ir::mir::xor_of::XorOf {
            input: f(*s.input).into(),
        }),
        Expr::ByteArrayToLong(s) => {
            ergotree_ir::mir::byte_array_to_long::ByteArrayToLong::try_build(f(*s.expr.input))
                .map(|v| {
                    Expr::ByteArrayToLong(Spanned {
                        source_span: s.source_span,
                        expr: v,
                    })
                })
                .expect("ByteArrayToLong in map_children")
        }
        Expr::ByteArrayToBigInt(s) => {
            ergotree_ir::mir::byte_array_to_bigint::ByteArrayToBigInt::try_build(f(*s.expr.input))
                .map(|v| {
                    Expr::ByteArrayToBigInt(Spanned {
                        source_span: s.source_span,
                        expr: v,
                    })
                })
                .expect("ByteArrayToBigInt in map_children")
        }
        // ---- end audit additions ----
        // FuncValue intentionally NOT recursed into here — `process_lambdas`
        // owns lambda-body traversal and callers of `map_children` rely on
        // it not crossing the lambda boundary. True leaves pass through.
        other => other,
    }
}

// -----------------------------------------------------------------------
// Subexpression collection
// -----------------------------------------------------------------------

/// Collect all subexpressions (non-leaf) in the tree.
fn collect_subexprs(expr: &Expr, out: &mut Vec<Expr>) {
    // Add this expression itself (if non-leaf)
    if is_collectible(expr) {
        out.push(expr.clone());
    }

    // Recurse into children
    match expr {
        Expr::BinOp(s) => {
            collect_subexprs(&s.expr.left, out);
            collect_subexprs(&s.expr.right, out);
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_subexprs(item, out);
            }
            collect_subexprs(&s.expr.result, out);
        }
        Expr::ValDef(s) => {
            collect_subexprs(&s.expr.rhs, out);
        }
        Expr::BoolToSigmaProp(bts) => {
            collect_subexprs(&bts.input, out);
        }
        Expr::If(if_op) => {
            collect_subexprs(&if_op.condition, out);
            collect_subexprs(&if_op.true_branch, out);
            collect_subexprs(&if_op.false_branch, out);
        }
        Expr::PropertyCall(s) => {
            collect_subexprs(&s.expr.obj, out);
        }
        Expr::MethodCall(s) => {
            collect_subexprs(&s.expr.obj, out);
            for arg in &s.expr.args {
                collect_subexprs(arg, out);
            }
        }
        Expr::ByteArrayToBigInt(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::ExtractAmount(ea) => {
            collect_subexprs(&ea.input, out);
        }
        Expr::ExtractRegisterAs(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::ExtractScriptBytes(esb) => {
            collect_subexprs(&esb.input, out);
        }
        Expr::ExtractBytes(eb) => {
            collect_subexprs(&eb.input, out);
        }
        Expr::ExtractId(ei) => {
            collect_subexprs(&ei.input, out);
        }
        Expr::ExtractCreationInfo(eci) => {
            collect_subexprs(&eci.input, out);
        }
        Expr::SizeOf(so) => {
            collect_subexprs(&so.input, out);
        }
        Expr::ByIndex(s) => {
            collect_subexprs(&s.expr.input, out);
            collect_subexprs(&s.expr.index, out);
            if let Some(ref d) = s.expr.default {
                collect_subexprs(d, out);
            }
        }
        Expr::SelectField(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::OptionGet(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::OptionIsDefined(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::OptionGetOrElse(s) => {
            collect_subexprs(&s.expr.input, out);
            collect_subexprs(&s.expr.default, out);
        }
        Expr::Filter(s) => {
            collect_subexprs(&s.expr.input, out);
            // Don't collect inside lambda — that's handled by cse_lambdas
        }
        Expr::Exists(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::ForAll(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::Map(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::Fold(s) => {
            collect_subexprs(&s.expr.input, out);
            collect_subexprs(&s.expr.zero, out);
        }
        Expr::Slice(s) => {
            collect_subexprs(&s.expr.input, out);
            collect_subexprs(&s.expr.from, out);
            collect_subexprs(&s.expr.until, out);
        }
        Expr::LogicalNot(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::Negation(s) => {
            collect_subexprs(&s.expr.input, out);
        }
        Expr::SigmaPropBytes(spb) => {
            collect_subexprs(&spb.input, out);
        }
        Expr::Upcast(uc) => {
            collect_subexprs(&uc.input, out);
        }
        Expr::Downcast(dc) => {
            collect_subexprs(&dc.input, out);
        }
        Expr::CalcBlake2b256(cb) => {
            collect_subexprs(&cb.input, out);
        }
        Expr::SigmaAnd(sa) => {
            for item in sa.items.iter() {
                collect_subexprs(item, out);
            }
        }
        Expr::SigmaOr(so) => {
            for item in so.items.iter() {
                collect_subexprs(item, out);
            }
        }
        Expr::Tuple(t) => {
            for item in t.items.iter() {
                collect_subexprs(item, out);
            }
        }
        Expr::TreeLookup(s) => {
            collect_subexprs(&s.expr.tree, out);
            collect_subexprs(&s.expr.key, out);
            collect_subexprs(&s.expr.proof, out);
        }
        Expr::Apply(app) => {
            collect_subexprs(&app.func, out);
            for arg in &app.args {
                collect_subexprs(arg, out);
            }
        }
        Expr::Append(s) => {
            collect_subexprs(&s.expr.input, out);
            collect_subexprs(&s.expr.col_2, out);
        }
        Expr::CreateProveDlog(cpd) => {
            collect_subexprs(&cpd.input, out);
        }
        Expr::CreateProveDhTuple(cpd) => {
            collect_subexprs(&cpd.g, out);
            collect_subexprs(&cpd.h, out);
            collect_subexprs(&cpd.u, out);
            collect_subexprs(&cpd.v, out);
        }
        Expr::Atleast(s) => {
            collect_subexprs(&s.bound, out);
            collect_subexprs(&s.input, out);
        }
        Expr::And(a) => collect_subexprs(&a.expr.input, out),
        Expr::Or(o) => collect_subexprs(&o.expr.input, out),
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                collect_subexprs(item, out);
            }
        }
        Expr::FuncValue(_) => {
            // Don't recurse into lambda bodies — handled separately
        }
        // Leaves
        Expr::Const(_)
        | Expr::ConstPlaceholder(_)
        | Expr::GlobalVars(_)
        | Expr::ValUse(_)
        | Expr::Context
        | Expr::Global => {}
        // Other types we don't encounter from our compiler
        _ => {}
    }
}

// -----------------------------------------------------------------------
// Predicates
// -----------------------------------------------------------------------

/// Is this expression worth collecting as a CSE candidate?
fn is_collectible(expr: &Expr) -> bool {
    // S54: allow CSE on bare `Const(SBigInt)` literals. Scala's IR
    // hash-conses Const graph nodes, so duplicate `0L.toBigInt` /
    // `BigInt256(0)` literals collapse to one sym whose schedule placement
    // becomes a `ValDef(rhs: Const)` at the LCA scope. Gating to SBigInt
    // keeps the byte savings (32-byte literals × N inline → 1 ValDef) while
    // avoiding regressions on small types (SLong/SInt) where a ValDef +
    // ValUse pair costs more than redundant inline placeholders.
    if let Expr::Const(c) = expr {
        return matches!(c.tpe, ergotree_ir::types::stype::SType::SBigInt);
    }
    !matches!(
        expr,
        Expr::Const(_)
            | Expr::ConstPlaceholder(_)
            | Expr::GlobalVars(_)
            | Expr::ValUse(_)
            | Expr::ValDef(_)
            | Expr::BlockValue(_)
            | Expr::FuncValue(_)
            | Expr::Context
            | Expr::Global
            | Expr::BoolToSigmaProp(_)
            | Expr::If(_)
            | Expr::SigmaAnd(_)
            | Expr::SigmaOr(_)
    )
}

/// Does the Scala graph hash-cons (deduplicate) this expression type?
///
/// In the Scala IR, each `Def` created via `reifyObject` goes through
/// `findOrCreateDefinition` which deduplicates by structural equality.
/// However, some node types are NOT effectively shared:
///
/// - `MethodCall`: goes through `rewriteDef` which can transform the node
///   differently depending on the current graph state
/// - `PropertyCall`: same as MethodCall (property access is a method call)
/// - `BinOp(Eq/NEq)`: `Equals[A: Elem]()` creates a new instance each time
///   because the implicit `Elem[A]` evidence is non-singleton, so case class
///   equality fails between instances
///
/// Other node types ARE shared:
/// - `BinOp(LT/GT/LE/GE)`: `OrderingLT[T](ord)` uses singleton `ExactOrdering`
/// - `BinOp(arithmetic)`: uses singleton `ExactNumeric`
/// - Constants, GlobalVars, ByIndex, SelectField, etc.
/// Does the Scala graph hash-cons (deduplicate) this expression type?
///
/// In the Scala IR, `findOrCreateDefinition` deduplicates by structural equality.
/// However, some node types are NOT effectively shared:
///
/// - `MethodCall` on non-global objects: goes through `rewriteDef` which produces
///   different results per call site. But MethodCall on GlobalVars (SELF, INPUTS,
///   OUTPUTS) IS shared because the receiver is a singleton symbol.
/// - `PropertyCall` on non-global objects: same as MethodCall.
/// - `BinOp(Eq/NEq)`: `Equals[A: Elem]()` creates a new instance each time
///   because the implicit `Elem[A]` evidence is non-singleton.
///
/// Node types that ARE shared:
/// - `BinOp(LT/GT/LE/GE)`: singleton `ExactOrdering`
/// - `BinOp(arithmetic)`: singleton `ExactNumeric`
/// - PropertyCall/MethodCall on GlobalVars: receiver is a singleton
/// - Constants, ByIndex, SelectField, SizeOf, ExtractAmount, etc.
/// Does the Scala graph hash-cons (deduplicate) this expression type?
///
/// Confirmed via debug logging of the Scala compiler (sigmastate-interpreter):
///
/// NOT shared (each source occurrence → separate graph symbol):
/// - `BinOp(Eq/NEq)`: `Equals[A: Elem]()` creates non-singleton instances
/// - `ExtractAmount` (box.value): separate MethodCall per occurrence
/// - `ExtractRegisterAs` (`box.R4[T]`): separate MethodCall per occurrence
/// - `ExtractScriptBytes` (box.propositionBytes): separate MethodCall per occurrence
/// - `ExtractBytes`, `ExtractId`, `ExtractCreationInfo`: same
/// - `OptionGet` (opt.get): separate per occurrence
/// - `OptionIsDefined` (opt.isDefined): separate per occurrence
///
/// SHARED (structurally identical → same graph symbol):
/// - `BinOp(LT/GT/LE/GE)`: singleton ExactOrdering
/// - `BinOp(arithmetic)`: singleton ExactNumeric
/// - `PropertyCall` (box.tokens): stable rewrite in Scala IR
/// - `SizeOf`, `ByIndex`, `SelectField`: pure structural operations
/// - Constants, GlobalVars
#[allow(clippy::doc_lazy_continuation)]
fn is_graph_shared(expr: &Expr) -> bool {
    match expr {
        // Eq/NEq: In Scala's graph, Equals[A: Elem]() is instantiated per call
        // site, but when both operands are already shared graph symbols the
        // resulting ApplyBinOp node hashes identically → shared.  We allow
        // sharing unconditionally and rely on dag_count >= 2 to extract only
        // when the full expression is actually duplicated.
        Expr::BinOp(_) => true,
        // Box accessor methods: shared when the input is a stable graph node
        // (global like SELF, or val-bound via ValUse).  In Scala's graph IR,
        // ExtractAmount/ExtractScriptBytes on a shared symbol (ValUse) are
        // hash-consed to one node.  Previously we only allowed globals, but
        // val-bound boxes (e.g., `val repaymentBox = OUTPUTS(0)`) also produce
        // stable symbols whose accessors are shared.
        Expr::ExtractAmount(ea) => is_input_stable(&ea.input),
        Expr::ExtractRegisterAs(s) => is_input_stable(&s.expr.input),
        // Sigmao S26 (2026-05-12, Cohort A1 / Session 5 of QB-HANDOFF-15-OF-15)
        // probed narrow suppression of `ExtractScriptBytes(GlobalVars(SelfBox))`
        // here to mirror Scala's per-thunk-distinct sym construction in
        // `Base.findOrCreateDefinition` + `ThunkScope.findDef` (sibling thunks
        // do not share `bodyDefs`, so two `SELF.propositionBytes` sites yield
        // two distinct syms each with `globalUsagesOf=1` → never extracted).
        // Result: sigmao SHAPE diff IMPROVES (common-multiset 38→40, L-only 7→4,
        // N-only 8→6 — two L-only shapes correctly inline `ExScript[Self]`
        // matching NODE) BUT sigmao BYTES REGRESS (Δ -24 → -28, 4B WORSE).
        // Root cause of regression: LOCAL's `ExtractScriptBytes(Self)` ValDef was
        // a "compensating" extraction — its 3-4B saving partially offset OTHER
        // structurally-missing extractions (Cohort B: `PDlog[DecPt[ByIdx]]`
        // 3-deep wrap and `SPBytes[VU]` chain split). Per
        // `feedback_falsification_fingerprint` rule #1 ("if inline costs more
        // bytes than extracted, the existing extraction is *correct*, not a
        // bug"), the existing extraction is empirically correct for current
        // LOCAL state. Standalone A1 suppression REVERTED; closure requires
        // coupled A1 + Cohort B (architectural) and is deferred per QB-SESSION-05
        // §4.4 P2.4-on-target. F1-F7 axes all green during the probe (sig-15
        // 12/15, F.2 563/575, lib 251/251, conformance 164/164, ecosystem 11/14).
        //
        // Sigmao S28 (2026-05-13, Cohort A1+A3 COUPLED / Session 7 of
        // QB-HANDOFF-15-OF-15). Re-applied this A1 suppression simultaneously
        // with S27's A3 `dag_count` override at process_ast_graph_impl (the
        // rule-#2 measurement from S26/S27 archives). Result: sigmao LOCAL
        // 1124 → 1118 (Δ -24 → -30) — EXACT simple sum of A1-alone (-4B) and
        // A3-alone (-2B); NO CASCADE fired. SHAPE diff improves dramatically
        // (common-multiset 38→42, L-only 7→2, N-only 8→4) but byte-direction
        // is wrong. Cross-fixture clean (12 MATCH + paideia +2 + gluon +102 +
        // F.2 563/575 + lib 251/251 + conformance 164/164 + collisions OK).
        // Empirically closes the "single OR coupled Scala-faithful structural
        // fix" class for sigmao; Session 8 pivots to Cohort B byte-ADDING
        // attacks (s2/s3/s6/s9 promotion). 28th falsification fingerprint
        // instance. Reverted; doc-comment durable.
        Expr::ExtractScriptBytes(esb) => is_input_stable(&esb.input),
        Expr::ExtractBytes(eb) => is_input_stable(&eb.input),
        Expr::ExtractId(ei) => is_input_stable(&ei.input),
        Expr::ExtractCreationInfo(eci) => is_input_stable(&eci.input),
        // ByIndex (Coll.apply in Scala): shared when input is stable
        // (global, PropertyCall chain on global, or referencing a val-bound collection).
        // In Scala's graph, MethodCall(coll, apply, [idx]) is extractable when usages >= 2.
        Expr::ByIndex(s) => is_input_stable(&s.expr.input),
        // OptionGet: shared when the input is itself a shared graph node
        // (e.g. ExtractRegisterAs on a stable receiver). Empirically NODE
        // does hoist `box.Rn[T].get` patterns (single ValDef + ValUses) even
        // though the historic note here claimed otherwise — see ergoraffle
        // (Sig-15 #6) where SELF.R4[Coll[Long]].get is bound once and reused
        // across 6 ByIndex sites.
        Expr::OptionGet(s) => is_graph_shared(&s.expr.input),
        // OptionIsDefined: shared when input is a stable graph node.
        // In Scala's graph IR, `MethodCall(stable_sym, OptionIsDefined)`
        // hash-conses to one Def via findOrCreateDefinition. With the S45
        // Upcast(Const,_) per-Thunk dedup landed, extracting
        // OptionIsDefined(stable) at outer scope mirrors node's ValDef
        // structure. (S45)
        Expr::OptionIsDefined(s) => is_input_stable(&s.expr.input),
        // MethodCall: not shared (rewriteDef produces different results per call)
        Expr::MethodCall(_) => false,
        // Pure-constant Upcast wrappers (e.g. `Upcast(Const(100000:SLong), SBigInt)`):
        // Scala's TreeBuilding interns these via findOrCreateDefinition (structural
        // equality on (input, toType)) — they ARE graph-shared syms, scoped per
        // ThunkScope. To match Scala, treat them as shareable here AND apply the
        // strict scope check (`appears_in_main_scope`) at Root mode below — see
        // process_ast_graph_impl `is_pure_const_upcast` gate. (S45)
        // Everything else (PropertyCall/tokens, SizeOf, ByIndex, SelectField,
        // arithmetic BinOp, constants): shared
        _ => true,
    }
}

/// Check if an expression resolves to a stable graph node.
/// In Scala's graph, `MethodCall(coll, apply, idx)` on a val-bound or
/// CSE-extracted collection is shared. `ValUse` indicates a stable binding.
fn is_input_stable(expr: &Expr) -> bool {
    match expr {
        Expr::GlobalVars(_) | Expr::Context => true,
        Expr::PropertyCall(s) => is_input_stable(&s.expr.obj),
        Expr::ValUse(_) => true,
        // ByIndex on a stable collection (e.g. `OUTPUTS(0)`) is itself
        // a stable graph node in Scala's IR — `findOrCreateDefinition`
        // hash-conses identical `coll.apply(idx)` calls. Treating it as
        // stable here lets `box.Rn[T].get` chains rooted at OUTPUTS(0) be
        // recognized as shared candidates (mirrors NODE's outer ValDef for
        // `OUTPUTS(0).R4[Coll[Long]].get`). Sig-15 #6 (ergoraffle) needs
        // this; without it, the inner ExtractR4 chain's is_graph_shared
        // gate trips on ByIndex's "non-stable" classification.
        Expr::ByIndex(s) => is_input_stable(&s.expr.input),
        _ => false,
    }
}

/// Check if an expression contains any ValUse reference.
fn contains_val_use(expr: &Expr) -> bool {
    if let Expr::ValUse(_) = expr {
        return true;
    }
    // sig-15 gluon S4: recurse via `direct_children` for completeness.
    // A missing arm here causes a CSE candidate that *does* contain a ValUse
    // to slip past the `has_lambdas`-branch gate and be extracted at outer
    // scope, hoisting an inner-scoped ValUse out of its binder. Same
    // walker-completeness class as S7's `replace_all` DecodePoint arm.
    direct_children(expr).iter().any(|c| contains_val_use(c))
}

/// Check if an expression tree contains any FuncValue (lambda).
/// Used to detect filter/fold/exists/forall which affect the Scala
/// compiler's flatSchedule usage counting.
fn contains_func_value(expr: &Expr) -> bool {
    match expr {
        Expr::FuncValue(_) => true,
        Expr::BlockValue(bv) => {
            bv.expr.items.iter().any(contains_func_value) || contains_func_value(&bv.expr.result)
        }
        Expr::ValDef(vd) => contains_func_value(&vd.expr.rhs),
        Expr::BinOp(s) => contains_func_value(&s.expr.left) || contains_func_value(&s.expr.right),
        Expr::PropertyCall(s) => contains_func_value(&s.expr.obj),
        Expr::MethodCall(s) => {
            contains_func_value(&s.expr.obj) || s.expr.args.iter().any(contains_func_value)
        }
        Expr::If(ite) => {
            contains_func_value(&ite.condition)
                || contains_func_value(&ite.true_branch)
                || contains_func_value(&ite.false_branch)
        }
        Expr::SigmaAnd(sa) => sa.items.iter().any(contains_func_value),
        Expr::SigmaOr(so) => so.items.iter().any(contains_func_value),
        Expr::BoolToSigmaProp(bsp) => contains_func_value(&bsp.input),
        Expr::SizeOf(so) => contains_func_value(&so.input),
        Expr::ByIndex(s) => {
            contains_func_value(&s.expr.input)
                || contains_func_value(&s.expr.index)
                || s.expr
                    .default
                    .as_ref()
                    .is_some_and(|d| contains_func_value(d))
        }
        Expr::SelectField(s) => contains_func_value(&s.expr.input),
        Expr::ExtractAmount(ea) => contains_func_value(&ea.input),
        Expr::ExtractRegisterAs(s) => contains_func_value(&s.expr.input),
        Expr::ExtractScriptBytes(esb) => contains_func_value(&esb.input),
        Expr::ExtractBytes(eb) => contains_func_value(&eb.input),
        Expr::ExtractId(ei) => contains_func_value(&ei.input),
        Expr::ExtractCreationInfo(eci) => contains_func_value(&eci.input),
        Expr::OptionGet(s) => contains_func_value(&s.expr.input),
        Expr::OptionIsDefined(s) => contains_func_value(&s.expr.input),
        Expr::LogicalNot(s) => contains_func_value(&s.expr.input),
        Expr::Negation(s) => contains_func_value(&s.expr.input),
        Expr::Map(s) => contains_func_value(&s.expr.input) || contains_func_value(&s.expr.mapper),
        Expr::Filter(s) => {
            contains_func_value(&s.expr.input) || contains_func_value(&s.expr.condition)
        }
        Expr::Fold(s) => {
            contains_func_value(&s.expr.input)
                || contains_func_value(&s.expr.zero)
                || contains_func_value(&s.expr.fold_op)
        }
        Expr::Exists(s) => {
            contains_func_value(&s.expr.input) || contains_func_value(&s.expr.condition)
        }
        Expr::ForAll(s) => {
            contains_func_value(&s.expr.input) || contains_func_value(&s.expr.condition)
        }
        Expr::CreateProveDlog(cpd) => contains_func_value(&cpd.input),
        Expr::Upcast(uc) => contains_func_value(&uc.input),
        Expr::ByteArrayToBigInt(s) => contains_func_value(&s.expr.input),
        Expr::CalcBlake2b256(cb) => contains_func_value(&cb.input),
        Expr::SigmaPropBytes(spb) => contains_func_value(&spb.input),
        _ => false,
    }
}

// -----------------------------------------------------------------------
// DAG usage counting
// -----------------------------------------------------------------------

/// Return the immediate sub-expressions of a node (not recursive).
/// This corresponds to the "syms" of a graph node in the Scala compiler —
/// the direct references a node makes to other expressions.
///
/// COVERAGE: this is a **counting walker** in WS-E.1's classification.
/// Adding a missing arm changes how downstream CSE counts occurrences
/// and can regress fixtures (chaincash 2026-05-03: speculative addition
/// of `Exponentiate`/`MultiplyGroup`/`DecodePoint`/`LongToByteArray`/
/// `ByteArrayToLong` regressed -62 → -77). Do NOT speculatively add
/// arms — every addition needs Metals confirmation against Scala's
/// `processAstGraph` traversal AND a concrete failure trace from a
/// specific fixture. Full coverage matrix:
/// [`tests/fixtures/significant_15/parity-handoffs/IR-PASS-COVERAGE-MATRIX.md`](../../tests/fixtures/significant_15/parity-handoffs/IR-PASS-COVERAGE-MATRIX.md)
fn direct_children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::PropertyCall(s) => vec![&s.expr.obj],
        Expr::MethodCall(s) => {
            let mut v = vec![&*s.expr.obj];
            v.extend(s.expr.args.iter());
            v
        }
        Expr::ExtractAmount(ea) => vec![&ea.input],
        Expr::ExtractRegisterAs(s) => vec![&s.expr.input],
        Expr::ExtractScriptBytes(esb) => vec![&esb.input],
        Expr::ExtractBytes(eb) => vec![&eb.input],
        Expr::ExtractId(ei) => vec![&ei.input],
        Expr::ExtractCreationInfo(eci) => vec![&eci.input],
        Expr::SizeOf(so) => vec![&so.input],
        Expr::ByIndex(s) => {
            let mut v = vec![&*s.expr.input, &*s.expr.index];
            if let Some(ref d) = s.expr.default {
                v.push(d);
            }
            v
        }
        Expr::SelectField(s) => vec![&s.expr.input],
        Expr::OptionGet(s) => vec![&s.expr.input],
        Expr::OptionIsDefined(s) => vec![&s.expr.input],
        Expr::BinOp(s) => vec![&s.expr.left, &s.expr.right],
        Expr::LogicalNot(s) => vec![&s.expr.input],
        Expr::Negation(s) => vec![&s.expr.input],
        Expr::BoolToSigmaProp(bsp) => vec![&bsp.input],
        Expr::Upcast(uc) => vec![&uc.input],
        Expr::Downcast(dc) => vec![&dc.input],
        Expr::CalcBlake2b256(cb) => vec![&cb.input],
        Expr::SigmaPropBytes(spb) => vec![&spb.input],
        Expr::ByteArrayToBigInt(s) => vec![&s.expr.input],
        Expr::ByteArrayToLong(s) => vec![&s.expr.input],
        Expr::LongToByteArray(s) => vec![&s.input],
        Expr::DecodePoint(s) => vec![&s.input],
        Expr::MultiplyGroup(s) => vec![&s.left, &s.right],
        Expr::Exponentiate(s) => vec![&s.left, &s.right],
        Expr::CreateProveDlog(cpd) => vec![&cpd.input],
        Expr::CreateProveDhTuple(cpd) => vec![&cpd.g, &cpd.h, &cpd.u, &cpd.v],
        Expr::Atleast(s) => vec![&s.bound, &s.input],
        Expr::If(ite) => vec![&ite.condition, &ite.true_branch, &ite.false_branch],
        Expr::BlockValue(bv) => {
            let mut v: Vec<&Expr> = bv.expr.items.iter().collect();
            v.push(&bv.expr.result);
            v
        }
        Expr::ValDef(vd) => vec![&vd.expr.rhs],
        Expr::FuncValue(fv) => vec![fv.body()],
        Expr::Map(s) => vec![&s.expr.input, &s.expr.mapper],
        Expr::Filter(s) => vec![&s.expr.input, &s.expr.condition],
        Expr::Fold(s) => vec![&s.expr.input, &s.expr.zero, &s.expr.fold_op],
        Expr::Exists(s) => vec![&s.expr.input, &s.expr.condition],
        Expr::ForAll(s) => vec![&s.expr.input, &s.expr.condition],
        Expr::SigmaAnd(sa) => sa.items.iter().collect(),
        Expr::SigmaOr(so) => so.items.iter().collect(),
        Expr::Tuple(t) => t.items.iter().collect(),
        Expr::Apply(app) => {
            let mut v = vec![&*app.func];
            v.extend(app.args.iter());
            v
        }
        Expr::TreeLookup(s) => vec![&s.expr.tree, &s.expr.key, &s.expr.proof],
        Expr::OptionGetOrElse(s) => vec![&s.expr.input, &s.expr.default],
        Expr::Slice(s) => vec![&s.expr.input, &s.expr.from, &s.expr.until],
        Expr::Append(s) => vec![&s.expr.input, &s.expr.col_2],
        Expr::And(a) => vec![&a.expr.input],
        Expr::Or(o) => vec![&o.expr.input],
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            items.iter().collect()
        }
        // Single-input arms (1-child) — G.2.3a, audit 1.A.
        Expr::CalcSha256(s) => vec![&s.input],
        Expr::BitInversion(s) => vec![&s.input],
        Expr::ExtractBytesWithNoRef(s) => vec![&s.input],
        Expr::SigmaPropIsProven(s) => vec![&s.input],
        Expr::ZkProofBlock(s) => vec![&s.input],
        Expr::XorOf(s) => vec![&s.input],
        // Multi-child arms — G.2.3a, audit 1.A.
        Expr::Xor(s) => vec![&s.left, &s.right],
        Expr::SubstConstants(s) => vec![
            &s.expr.script_bytes,
            &s.expr.positions,
            &s.expr.new_values,
        ],
        Expr::CreateAvlTree(s) => {
            let mut v: Vec<&Expr> = vec![&s.flags, &s.digest, &s.key_length];
            if let Some(vl) = s.value_length.as_deref() {
                v.push(vl);
            }
            v
        }
        Expr::DeserializeRegister(s) => match s.default.as_deref() {
            Some(d) => vec![d],
            None => vec![],
        },
        // Leaf nodes — no children
        Expr::Const(_)
        | Expr::ConstPlaceholder(_)
        | Expr::GlobalVars(_)
        | Expr::ValUse(_)
        | Expr::Context
        | Expr::Global => vec![],
        // Catch-all for remaining less common nodes.
        //
        // WS-G.2.3a (audit 1.A fix): the previously-missing-arm set
        // (SubstConstants, CalcSha256, Xor, BitInversion, ExtractBytesWithNoRef,
        // SigmaPropIsProven, ZkProofBlock, XorOf, CreateAvlTree,
        // DeserializeRegister) now has explicit arms above. The remaining
        // variants in this fall-through truly have no Expr children
        // (GetVar, DeserializeContext, Collection::BoolConstants, …) — drop
        // is correct by construction. `walker_completeness_probe` mod at file
        // bottom pins this invariant per variant.
        _ => vec![],
    }
}

/// Count DAG usages for each unique sub-expression.
///
/// Simulates the Scala compiler's hash-consing: structurally identical
/// sub-expressions are ONE node. For each unique node, counts how many
/// DISTINCT parent nodes reference it as a direct child. This matches
/// `hasManyUsagesGlobal` in the Scala compiler's `processAstGraph`.
///
/// Returns a list of (expression, dag_usage_count) pairs.
//
// Walker-completeness empirical validation (sig-15 paideia S9, 2026-05-10):
// CSE_TRACE_EXTRACT against debug_paideia at HEAD reports
// `[PAG/Root] extract id=82 dag_count=3 :: Tuple([Coll[Byte](), Const(0:SLong)])` —
// the count walker (via collect_subexprs + direct_children) correctly identifies
// the cross-branch shared pure-const Tuple at dag_count=3 globally and extracts.
// 15th falsification fingerprint instance: brief's "branch-local count walker
// undercounts cross-branch sym" hypothesis FALSIFIED at HEAD. Paideia's Δ +2
// residual is structural placement (Scala first-DFS-scope vs Rust outermost-
// eligible-scope), NOT walker completeness. Distinct from #13 sigmao S7 +
// #14 gluon S4 (both 1-line walker-arm fixes); paideia plateau hardens with
// empirical evidence the structural barrier is real. See `project_sig15_paideia_s9_*`
// memory + cluster archive `<HEAD>_sig15-paideia-session9-walker-lens-falsified.md`.
//
// Inversion A — paideia surgical-fix space EMPTY (sig-15 paideia S20/Inversion A,
// 2026-05-11, HEAD `b8864e16`): Probe 1 re-confirmed dag_count=3 for the Tuple at
// id=82, and source-AST inspection placed the 5 syntactic occurrences across TWO
// top-level val.RHS thunks (validStakeTx addStake-else, validUnstakeTx partial-
// branch) — Row 2 of the inversion-A decision tree (cross-thunk shared). The
// surgical-fix-Row-2 space was already empirically exhausted by S7 (Bug A pure-
// const Tuple predicate alone → +14B via per-branch sibling extraction) and S8
// (B-medium HashSet cross-branch suppression → +15B via wrong-branch suppression).
// Closure requires Scala's first-DFS-construction-scope (sibling-thunk owner-by-
// first-encounter) mechanism, which IS the WS-G hash-cons migration falsified
// across 11 sessions / 19 falsification fingerprint instances. The "surgical-first,
// architectural-second" hypothesis from the inversion analysis is empirically
// EMPTY for paideia. 20th falsification fingerprint instance. Plateau hardens.
// See archive `b8864e16_sig15-paideia-S20-inversion-A-row2-confirmed-fix-space-empty.md`.
//
// Inversion C — sigmao -32 amortization cost-model FALSIFIED at Probe 0 (sig-15
// sigmao S8 / Inversion C, 2026-05-11, HEAD `51f4effc`). The S7 archive
// (`83a962f0`) coined "amortization-aware extraction threshold" as the residual
// class for sigmao's -32B gap: hypothesis was that `needs_check` (or a sibling
// gate) should reject extraction when ValDef-header overhead exceeds sharing
// savings (`savings = N_uses * inline_size`, `cost = HEADER + N_uses * VALUSE`).
// Metals read of `sigma.compiler.ir.TreeBuilding.processAstGraph` (4 predicates:
// `hasManyUsagesGlobal && !IsContextProperty && !IsInternalDef && !IsConstantDef`)
// + `sigma.compiler.ir.AstGraphs.hasManyUsagesGlobal` (literal
// `globalUsagesOf(s).length > 1`) shows **no size, rhs-shape, or amortization
// predicate anywhere in Scala's gate**. Row C4 of the decision tree fires:
// hypothesis falsified at Probe 0 layer; sigmao's -32 reframed as Rust over-
// extracting where Scala's `>1` test rejects (live-tree use count, not threshold).
// 22nd falsification fingerprint instance. All three inversions (A architectural,
// B post-pass merge, C cost-model threshold) now exhausted; WS-G stop condition
// triggers. 12/15 is the honest ship state.
// See archive `<commit>_sig15-sigmao-S8-inversion-C-amortization-falsified.md`.
//
// BISECT-A — sigmao -32 per-sym divergence FALSIFIED at Probe 1 (sig-15
// sigmao S9, 2026-05-11, HEAD `1ab6c3e0`). BISECT-A handoff opened a new
// methodology layer (per-sym empirical bisection of count / predicate /
// scope) and locked the target sym as `CreateProveDlog(DecodePoint(
// ByIndex(VU(8), 0)))` citing S7 archive (`83a962f0`) as "Rust extracts
// at root, dag_count=2; Scala doesn't extract." Probe 0 metals closed
// bucket (b) structurally (`IsContextProperty` / `IsInternalDef` /
// `IsConstantDef` unapply arms verified, none match CreateProveDlog).
// Probe 1 empirical (`CSE_TRACE_EXTRACT=1 debug_sigmao` at HEAD)
// FALSIFIES the premise:
//     [PAG/Root] skip dag_count=1 :: CreateProveDlog(DecodePoint(VU(21)))
// Rust and Scala AGREE on this sym — both skip. S7's `dag_count=2` was
// conditional on the uncommitted B0 widened-gate experiment, not baseline
// HEAD. The real -32B residual lives in constant-pool emission (LOCAL
// constants_len 59 vs NODE 61, first byte-diff at offset 1 — the
// constants_len varint itself), one layer downstream of this function's
// extraction gate. 23rd falsification fingerprint instance — new class:
// handoff-premise carrying conditional-state forward as baseline. Rule:
// when a handoff locks a specific sym/expression/number from a prior
// archive, re-anchor via `CSE_TRACE_EXTRACT=1 debug_<fixture>` at HEAD
// (~30s) BEFORE building bucket frameworks. Bisection methodology not
// invalidated but requires re-derivation; sigmao's residual likely falls
// OUTSIDE its applicability (constant-pool emission, not extraction-gate
// decision). See archive
// `<commit>_sig15-sigmao-S9-bisect-A-handoff-premise-falsified.md`.
//
// BISECT-A2 — sigmao -32 constant-pool bucket A2.2 CLASSIFIED (sig-15
// sigmao S10, 2026-05-11, HEAD `e845da3e`). Pure measurement session per
// `BISECT-A2-SIGMAO-CONSTANT-POOL-HANDOFF.md`. Probe 0 metals on
// `sigma.serialization.ConstantStore` (full source) shows `put` is
// non-deduping: `store += c.asInstanceOf[Constant[SType]]; mkConstant-
// Placeholder(store.length - 1, tpe)` — every call appends, equal
// Constants do NOT collapse. Probe 0 metals on
// `sigma.compiler.ir.TreeBuilding::buildValue` shows the `Const(x)` arm
// unconditionally calls `store.put(constant)` when `constantsProcessing
// = Some(store)`; the `IsConstantDef` gate in `processAstGraph` excludes
// Const from ValDef emission only, not from pool emission. Therefore
// `constants_len` divergence ⇔ distinct-`Const`-Sym count divergence in
// the post-CSE live graph. Bucket (A2.1) ConstantStore::put dedup is
// structurally FALSIFIED. Probe 1 empirical (multiset diff of LOCAL +
// NODE pool dumps from `debug_sigmao`): symmetric difference is exactly
// `NODE - LOCAL = +1 × Const(1: SInt) + +1 × Const(3: SInt)`; all 13
// other distinct pool-values agree in multiplicity exactly. Bucket
// (A2.2) upstream pass collapses CLASSIFIED — a Rust HIR or MIR pass
// collapses one Const(1: SInt) and one Const(3: SInt) graph node that
// Scala's IR preserves as distinct. Next session: per-Const-value count
// dump at HIR→MIR boundary and each pre-CSE stage to localize the
// collapsing pass. Candidate surfaces ranked: HIR const-fold > MIR
// lowering literal-memoization > inline_alias_vals /
// inline_single_use_vals / deduplicate_inner_consts. See sub-handoff
// `BISECT-A2.2-SIGMAO-COLLAPSING-PASS-HANDOFF.md` and archive
// `<commit>_sig15-sigmao-S10-BISECT-A2-bucket-A2.2-classified.md`.
fn count_dag_usages(expr: &Expr) -> Vec<(Expr, usize)> {
    // Step 1: Collect all sub-expressions and deduplicate by structural equality
    let mut all_subexprs: Vec<Expr> = Vec::new();
    collect_subexprs(expr, &mut all_subexprs);

    let mut unique: Vec<Expr> = Vec::new();
    for sub in &all_subexprs {
        if !unique.iter().any(|u| u == sub) {
            unique.push(sub.clone());
        }
    }

    // Also include the root expression and any top-level structure
    // (BlockValue items, If branches, etc.) to capture parent references
    // from the tree root that aren't themselves sub-expressions of other nodes.

    // Step 2: For each unique expression, count edge incidences from its
    // parents (including the root). Mirrors Scala's `buildUsageMap` in
    // `AstGraphs.AstGraph` which uses `node.usages: DBuffer[Int]` and
    // appends `symId` per dep slot — `hasManyUsagesGlobal(s)` then checks
    // `usages.length > 1`. So a parent like `Coll(x, x)` contributes 2 to
    // x's usage count, NOT 1 (which a HashSet of distinct parents would
    // give). Without edge multiplicity, structurally-identical siblings
    // inside SigmaAnd/SigmaOr/Tuple/ConcreteCollection (e.g. cluster 001:
    // `Coll(proveDlog(g), proveDlog(g))` inside `atLeast`) appear to have
    // a single distinct parent and miss CSE extraction.
    let mut parent_counts: Vec<usize> = vec![0; unique.len()];

    // Also process the root expression itself as a parent
    let root_children = direct_children(expr);
    for child in &root_children {
        if let Some(child_idx) = unique.iter().position(|u| u == *child) {
            parent_counts[child_idx] += 1;
        }
    }

    for parent in unique.iter() {
        let children = direct_children(parent);
        for child in children {
            if let Some(child_idx) = unique.iter().position(|u| u == child) {
                parent_counts[child_idx] += 1;
            }
        }
    }

    // Diagnostic infrastructure (sig-15 sigmao S6, 2026-05-09):
    // For each sym in `unique`, log (collectible_parent_count, structural_
    // parent_count) where structural-parent set is restricted to semantic
    // Scala graph parents (BoolToSigmaProp / SigmaAnd / SigmaOr / If) —
    // excluding emit-time wrappers (ValDef / BlockValue / FuncValue) which
    // are not pre-extraction syms in Scala's `flatSchedule`.
    //
    // No behavioral effect — purely observational. Gated by CSE_TRACE_STRUCT_PARENTS.
    //
    // Empirical findings (sigmao session 6):
    // - Strict-subset gate (collectible==0 AND structural>=2) is INERT on
    //   sigmao: CreateProveDlog has collectible=1 structural=1 (SigmaPropBytes
    //   is a collectible parent). Brief's "3 BoolToSigmaProp parents" framing
    //   empirically falsified — only 1 structural parent exists for the sym.
    // - Wider gate (collectible<2 AND total>=2) tipping CreateProveDlog from
    //   count=1 → 2 regresses sigmao 1116B → 987B with segregation FAIL —
    //   matches S5 broad Fix A failure mode verbatim. Renumbering pipeline
    //   prerequisite is real even for single-sym extraction.
    let trace = std::env::var("CSE_TRACE_STRUCT_PARENTS").is_ok();
    if trace {
        let mut structural_parents: Vec<&Expr> = Vec::new();
        collect_structural_parents(expr, &mut structural_parents);
        let mut unique_struct: Vec<&Expr> = Vec::new();
        for sp in &structural_parents {
            if !unique_struct.iter().any(|u| u == sp) {
                unique_struct.push(*sp);
            }
        }
        let mut structural_counts: Vec<usize> = vec![0; unique.len()];
        for sp in &unique_struct {
            for child in direct_children(sp) {
                if let Some(child_idx) = unique.iter().position(|u| u == child) {
                    structural_counts[child_idx] += 1;
                }
            }
        }
        for i in 0..unique.len() {
            if structural_counts[i] >= 1 || parent_counts[i] >= 1 {
                let gate_strict = parent_counts[i] == 0 && structural_counts[i] >= 2;
                let gate_total =
                    parent_counts[i] < 2 && parent_counts[i] + structural_counts[i] >= 2;
                eprintln!(
                    "[STRUCT_GATE_INFO] sym={:.140?} collectible={} structural={} strict_fires={} total_fires={}",
                    unique[i], parent_counts[i], structural_counts[i], gate_strict, gate_total
                );
            }
        }
    }

    // Step 3: Return (expr, usage_count) pairs
    unique.into_iter().zip(parent_counts).collect()
}

/// Recursively collect every structural-only (non-collectible) non-leaf node
/// reachable from `expr` that is a SEMANTIC graph parent in Scala's IR.
/// Excludes ValDef and BlockValue: those are emit-time encoder wrappers, not
/// pre-extraction graph nodes (Scala's processAstGraph operates on Defs, not
/// on encoded ValDef/BlockValue forms — those don't exist as syms in
/// `flatSchedule`).
fn collect_structural_parents<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    let is_struct_parent = matches!(
        expr,
        Expr::BoolToSigmaProp(_) | Expr::SigmaAnd(_) | Expr::SigmaOr(_) | Expr::If(_)
    );
    if is_struct_parent {
        out.push(expr);
    }
    for child in direct_children(expr) {
        collect_structural_parents(child, out);
    }
}

// -----------------------------------------------------------------------
// Probe 1 — orphan ValUse detector (sig-15 sigmao S7, 2026-05-09)
// -----------------------------------------------------------------------
//
// Scope-aware walker that detects ValUse(K) refs whose ValDef(K) is NOT
// in the lexical scope at the use site. Pure observability.
//
// Scope semantics:
//   - BlockValue(items, result):
//       - For each ValDef item at index i, its rhs sees the OUTER scope
//         plus prior items[0..i].id.
//       - The result sees outer scope plus all items[].id.
//   - FuncValue(args, body):
//       - body sees outer scope plus all args.idx.
//   - Other forms: pass current scope through to children.
//
// An orphan ValUse(K) is one where K is in NEITHER the outer scope NOR
// any in-scope sibling at the point of reference.
#[allow(dead_code)]
fn count_orphan_valuses(expr: &Expr) -> usize {
    let mut count = 0usize;
    let scope: HashSet<u32> = HashSet::new();
    walk_orphan(expr, &scope, &mut count);
    count
}

#[allow(dead_code)]
fn walk_orphan(expr: &Expr, scope: &HashSet<u32>, count: &mut usize) {
    match expr {
        Expr::ValUse(vu) => {
            if !scope.contains(&vu.val_id.0) {
                *count += 1;
            }
        }
        Expr::BlockValue(s) => {
            // Items processed in order; each ValDef's rhs sees prior items
            // only. The result sees all items.
            let mut block_scope = scope.clone();
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    walk_orphan(&vd.expr.rhs, &block_scope, count);
                    block_scope.insert(vd.expr.id.0);
                } else {
                    walk_orphan(item, &block_scope, count);
                }
            }
            walk_orphan(&s.expr.result, &block_scope, count);
        }
        Expr::ValDef(s) => {
            // Top-level ValDef (not inside a BlockValue items[] list) — rhs
            // sees current scope; the def's id does NOT enter scope at this
            // level (only its enclosing BlockValue handles that).
            walk_orphan(&s.expr.rhs, scope, count);
        }
        Expr::FuncValue(fv) => {
            let mut body_scope = scope.clone();
            for arg in fv.args() {
                body_scope.insert(arg.idx.0);
            }
            walk_orphan(fv.body(), &body_scope, count);
        }
        other => {
            for child in direct_children(other) {
                walk_orphan(child, scope, count);
            }
        }
    }
}

/// BISECT-A4: split `count_occurrences(scope, target)` into (in_if_cond, outside_if_cond).
/// "in_if_cond" = occurrences anywhere within the `condition` sub-tree of an `If` node.
/// Used by `CSE_TRACE_PRE_EXTRACT` to test whether sigmao's BinOp(Eq, SizeOf, Const)
/// candidates appear ONLY inside If.cond positions (while named-fixture extractions
/// also appear outside If.cond).
fn count_in_if_cond(expr: &Expr, target: &Expr, inside_cond: bool) -> (usize, usize) {
    let here = if expr == target {
        if inside_cond { (1, 0) } else { (0, 1) }
    } else {
        (0, 0)
    };
    let mut acc = here;
    let mut add = |c: (usize, usize)| {
        acc.0 += c.0;
        acc.1 += c.1;
    };
    match expr {
        Expr::If(if_op) => {
            add(count_in_if_cond(&if_op.condition, target, true));
            add(count_in_if_cond(&if_op.true_branch, target, inside_cond));
            add(count_in_if_cond(&if_op.false_branch, target, inside_cond));
        }
        Expr::BinOp(s) => {
            add(count_in_if_cond(&s.expr.left, target, inside_cond));
            add(count_in_if_cond(&s.expr.right, target, inside_cond));
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                add(count_in_if_cond(item, target, inside_cond));
            }
            add(count_in_if_cond(&s.expr.result, target, inside_cond));
        }
        Expr::ValDef(s) => {
            add(count_in_if_cond(&s.expr.rhs, target, inside_cond));
        }
        other => {
            for c in direct_children(other) {
                add(count_in_if_cond(&c, target, inside_cond));
            }
        }
    }
    acc
}

/// BISECT-A4: one-token variant tag for an extracted `pre_extract_from_valdefs` candidate.
/// Used by the `CSE_TRACE_PRE_EXTRACT` arm to characterize the wrapper shape.
fn pre_extract_variant_name(e: &Expr) -> &'static str {
    match e {
        Expr::Const(_) => "Const",
        Expr::ValUse(_) => "ValUse",
        Expr::GlobalVars(_) => "GlobalVars",
        Expr::BinOp(_) => "BinOp",
        Expr::And(_) => "And",
        Expr::Or(_) => "Or",
        Expr::Xor(_) => "Xor",
        Expr::If(_) => "If",
        Expr::SelectField(_) => "SelectField",
        Expr::ByIndex(_) => "ByIndex",
        Expr::OptionGet(_) => "OptionGet",
        Expr::OptionGetOrElse(_) => "OptionGetOrElse",
        Expr::OptionIsDefined(_) => "OptionIsDefined",
        Expr::SizeOf(_) => "SizeOf",
        Expr::ExtractAmount(_) => "ExtractAmount",
        Expr::ExtractScriptBytes(_) => "ExtractScriptBytes",
        Expr::ExtractBytes(_) => "ExtractBytes",
        Expr::ExtractBytesWithNoRef(_) => "ExtractBytesWithNoRef",
        Expr::ExtractCreationInfo(_) => "ExtractCreationInfo",
        Expr::ExtractId(_) => "ExtractId",
        Expr::ExtractRegisterAs(_) => "ExtractRegisterAs",
        Expr::PropertyCall(_) => "PropertyCall",
        Expr::MethodCall(_) => "MethodCall",
        Expr::Apply(_) => "Apply",
        Expr::Tuple(_) => "Tuple",
        Expr::Collection(_) => "Collection",
        Expr::CalcBlake2b256(_) => "CalcBlake2b256",
        Expr::CalcSha256(_) => "CalcSha256",
        Expr::BoolToSigmaProp(_) => "BoolToSigmaProp",
        Expr::SigmaPropBytes(_) => "SigmaPropBytes",
        Expr::CreateProveDlog(_) => "CreateProveDlog",
        Expr::CreateProveDhTuple(_) => "CreateProveDhTuple",
        Expr::DecodePoint(_) => "DecodePoint",
        Expr::Fold(_) => "Fold",
        Expr::Map(_) => "Map",
        Expr::Filter(_) => "Filter",
        Expr::Append(_) => "Append",
        Expr::Slice(_) => "Slice",
        Expr::SigmaAnd(_) => "SigmaAnd",
        Expr::SigmaOr(_) => "SigmaOr",
        Expr::Atleast(_) => "Atleast",
        Expr::Negation(_) => "Negation",
        Expr::Upcast(_) => "Upcast",
        Expr::Downcast(_) => "Downcast",
        Expr::LogicalNot(_) => "LogicalNot",
        _ => "Other",
    }
}

/// BISECT-A3: count occurrences of `Const(target: SInt)` in a MIR `Expr` tree.
/// Used by `trace_slot_shift`'s `CSE_TRACE_CONST_COUNTS` arm to localize the
/// pre-CSE pass where the sigmao `1:SInt`/`3:SInt` multiset diff (NODE 12/4 vs
/// LOCAL 11/3) emerges.
pub(crate) fn count_const_sint(expr: &Expr, target: i32) -> usize {
    let mut all = Vec::new();
    collect_consts(expr, &mut all);
    all.iter()
        .filter(|c| matches!(c, Expr::Const(k)
            if matches!(k.v, ergotree_ir::mir::constant::Literal::Int(v) if v == target)))
        .count()
}

/// Q24-S3 / WS-G QB-HANDOFF-15-OF-15 Session 3 — normalized shape signature
/// for sigmao 9-NODE-only-shape per-stage evolution probe.
/// Mirror of `compiler::expr_shape` (which is `#[cfg(test)]`-gated and not
/// reachable from this non-test trace site). Strips ValIds + concrete Const
/// values; retains tpe + opcode + child structure + per-opcode discriminants.
fn shape_signature(e: &Expr) -> String {
    use ergotree_ir::traversable::Traversable;
    let (name, extra): (&'static str, Option<String>) = match e {
        Expr::Const(c) => return format!("K({})", c.tpe),
        Expr::ConstPlaceholder(c) => return format!("Cph({})", c.tpe),
        Expr::ValUse(_) => return "VU".to_string(),
        Expr::Context => return "Ctx".to_string(),
        Expr::Global => return "Global".to_string(),
        Expr::GlobalVars(g) => {
            use ergotree_ir::mir::global_vars::GlobalVars;
            let nm = match g {
                GlobalVars::Inputs => "Inputs",
                GlobalVars::Outputs => "Outputs",
                GlobalVars::Height => "Height",
                GlobalVars::SelfBox => "Self",
                GlobalVars::MinerPubKey => "MinerPk",
                GlobalVars::GroupGenerator => "GroupGen",
            };
            return nm.to_string();
        }
        Expr::Append(_) => ("Append", None),
        Expr::SubstConstants(_) => ("SubstConsts", None),
        Expr::ByteArrayToLong(_) => ("BAToL", None),
        Expr::ByteArrayToBigInt(_) => ("BAToBI", None),
        Expr::LongToByteArray(_) => ("LtoBA", None),
        Expr::Collection(_) => ("Coll", None),
        Expr::Tuple(_) => ("Tup", None),
        Expr::CalcBlake2b256(_) => ("Blake", None),
        Expr::CalcSha256(_) => ("Sha", None),
        Expr::FuncValue(_) => ("Fn", None),
        Expr::Apply(_) => ("Apply", None),
        Expr::MethodCall(m) => ("MC", Some(format!("{}", m.expr().method.name()))),
        Expr::PropertyCall(p) => ("PC", Some(format!("{}", p.expr().method.name()))),
        Expr::BlockValue(_) => ("Block", None),
        Expr::ValDef(v) => ("VD", Some(format!("id={}", v.expr().id.0))),
        Expr::If(_) => ("If", None),
        Expr::BinOp(b) => ("BO", Some(format!("{}", b.expr().kind))),
        Expr::And(_) => ("And", None),
        Expr::Or(_) => ("Or", None),
        Expr::Xor(_) => ("Xor", None),
        Expr::Atleast(_) => ("AtLeast", None),
        Expr::LogicalNot(_) => ("Not", None),
        Expr::Negation(_) => ("Neg", None),
        Expr::BitInversion(_) => ("BitInv", None),
        Expr::OptionGet(_) => ("OGet", None),
        Expr::OptionIsDefined(_) => ("OIsDef", None),
        Expr::OptionGetOrElse(_) => ("OGetOr", None),
        Expr::ExtractAmount(_) => ("ExAmt", None),
        Expr::ExtractRegisterAs(r) => (
            "ExReg",
            Some(format!("R{}", r.expr().register_id)),
        ),
        Expr::ExtractBytes(_) => ("ExBytes", None),
        Expr::ExtractBytesWithNoRef(_) => ("ExBNoRef", None),
        Expr::ExtractScriptBytes(_) => ("ExScript", None),
        Expr::ExtractCreationInfo(_) => ("ExCreaInfo", None),
        Expr::ExtractId(_) => ("ExId", None),
        Expr::ByIndex(b) => (
            "ByIdx",
            Some(if b.expr().default.is_some() { "or" } else { "raw" }.to_string()),
        ),
        Expr::SizeOf(_) => ("SizeOf", None),
        Expr::Slice(_) => ("Slice", None),
        Expr::Fold(_) => ("Fold", None),
        Expr::Map(_) => ("Map", None),
        Expr::Filter(_) => ("Filter", None),
        Expr::Exists(_) => ("Exists", None),
        Expr::ForAll(_) => ("ForAll", None),
        Expr::SelectField(s) => (
            "Sel",
            Some(format!("#{}", s.expr().field_index.zero_based_index() + 1)),
        ),
        Expr::BoolToSigmaProp(_) => ("BTSP", None),
        Expr::Upcast(u) => ("Up", Some(format!("{}", u.tpe))),
        Expr::Downcast(d) => ("Dn", Some(format!("{}", d.tpe()))),
        Expr::CreateProveDlog(_) => ("PDlog", None),
        Expr::CreateProveDhTuple(_) => ("PDht", None),
        Expr::SigmaPropBytes(_) => ("SPBytes", None),
        Expr::SigmaPropIsProven(_) => ("SPIsProven", None),
        Expr::ZkProofBlock(_) => ("ZkProof", None),
        Expr::DecodePoint(_) => ("DecPt", None),
        Expr::SigmaAnd(_) => ("SAnd", None),
        Expr::SigmaOr(_) => ("SOr", None),
        Expr::GetVar(v) => ("GVar", Some(format!("#{}", v.expr().var_id))),
        Expr::DeserializeRegister(_) => ("DesReg", None),
        Expr::DeserializeContext(_) => ("DesCtx", None),
        Expr::MultiplyGroup(_) => ("MulG", None),
        Expr::Exponentiate(_) => ("ExpG", None),
        Expr::XorOf(_) => ("XorOf", None),
        Expr::TreeLookup(_) => ("AvlLook", None),
        Expr::CreateAvlTree(_) => ("MkAvl", None),
    };
    let children: Vec<String> = e.children().map(shape_signature).collect();
    let head = match extra {
        Some(x) => format!("{}<{}>", name, x),
        None => name.to_string(),
    };
    if children.is_empty() {
        head
    } else {
        format!("{}[{}]", head, children.join(","))
    }
}

/// Q24-S3 — count subtree occurrences of `target` shape anywhere in `expr`.
fn count_shape_tree(expr: &Expr, target: &str) -> usize {
    use ergotree_ir::traversable::Traversable;
    let mut count = if shape_signature(expr) == target { 1 } else { 0 };
    for child in expr.children() {
        count += count_shape_tree(child, target);
    }
    count
}

/// Q24-S3 — count outer `BlockValue.items` ValDef RHS positions whose shape
/// signature matches `target`. Mirrors `print_outer_valdef_shape_diff` outer-VD
/// extraction logic; only the root-level BlockValue is inspected.
fn count_shape_outer_vd(expr: &Expr, target: &str) -> usize {
    if let Expr::BlockValue(bv) = expr {
        let mut c = 0;
        for item in &bv.expr.items {
            if let Expr::ValDef(vd) = item {
                if shape_signature(&vd.expr.rhs) == target {
                    c += 1;
                }
            }
        }
        c
    } else {
        0
    }
}

/// Q24-S3 / QB-SESSION-03 — sigmao 9-NODE-only-shape targets from S23 archive
/// `3a955f1f_*` (re-anchored at HEAD `3a955f1f` SHAPE DIFF). Per-stage
/// `(tree, outer_vd)` count probe localizes each shape to a pipeline stage.
/// Ordered by archive Table §3.1 (DAG-identity shapes first, simple-undercount
/// second, fold-asymmetry last).
const SIGMAO_NODE_ONLY_SHAPES: &[(&str, &str, i32)] = &[
    // (alias, shape signature, NODE outer_vd count)
    ("s1_isMinted_inline", "BO<&&>[BO<==>[VU,ExId[VU]],BO<==>[ExScript[Self],ExScript[VU]]]", 1),
    ("s4_isExercible_unfolded", "If[BO<==>[ByIdx<raw>[VU,K(Int)],K(Long)],BO<&&>[BO<&&>[VU,VU],BO<<>[VU,BO<+>[VU,K(Long)]]],BO<&&>[VU,BO<<=>[VU,VU]]]", 1),
    ("s5_validBasicReplOut0_inline", "If[BO<==>[VU,ExScript[Self]],BO<&&>[BO<&&>[BO<&&>[BO<&&>[BO<>=>[ExAmt[VU],VU],BO<==>[OGet[ExReg<R4>[VU]],OGet[ExReg<R4>[VU]]]],BO<==>[OGet[ExReg<R5>[VU]],VU]],BO<==>[OGet[ExReg<R6>[VU]],OGet[ExReg<R6>[VU]]]],BO<==>[OGet[ExReg<R7>[VU]],VU]],K(Boolean)]", 1),
    ("s7_PDlog_DecPt_ByIdx_chain", "PDlog[DecPt[ByIdx<raw>[VU,K(Int)]]]", 1),
    ("s8_SPBytes_VU_chain_dn", "SPBytes[VU]", 1),
    ("s2_ByIdx_or_VU_K_VU", "ByIdx<or>[VU,K(Int),VU]", 6),
    ("s3_ByIdx_raw_Outputs_K", "ByIdx<raw>[Outputs,K(Int)]", 3),
    ("s6_PC_tokens_VU", "PC<tokens>[VU]", 3),
    ("s9_Sel_1_VU", "Sel<#1>[VU]", 4),
];

#[inline]
fn trace_slot_shift(stage: &str, expr: &Expr) {
    if std::env::var("CSE_TRACE_CONST_COUNTS").is_ok() {
        let c1 = count_const_sint(expr, 1);
        let c3 = count_const_sint(expr, 3);
        eprintln!(
            "[CONST_COUNTS] stage={:36}  1:SInt={:>3}  3:SInt={:>3}",
            stage, c1, c3
        );
    }
    if std::env::var("CSE_TRACE_SHAPE_EVOLUTION").is_ok() {
        let mut parts: Vec<String> = Vec::with_capacity(SIGMAO_NODE_ONLY_SHAPES.len());
        for (alias, target, node_vd) in SIGMAO_NODE_ONLY_SHAPES {
            let tree = count_shape_tree(expr, target);
            let outer_vd = count_shape_outer_vd(expr, target);
            parts.push(format!("{}={}/{}|N_vd={}", alias, tree, outer_vd, node_vd));
        }
        eprintln!("[SHAPE_EVO] stage={:38} {}", stage, parts.join(" "));
    }
    if std::env::var("CSE_TRACE_SLOT_SHIFT").is_ok() {
        let mut orphans: Vec<u32> = Vec::new();
        let scope: HashSet<u32> = HashSet::new();
        collect_orphan_ids(expr, &scope, &mut orphans);
        if !orphans.is_empty() {
            eprintln!("[SLOT_SHIFT] stage={} orphan_ids={:?}", stage, orphans);
        }
    }
    if std::env::var("CSE_TRACE_VU_PATH").is_ok() {
        let mut found: Vec<(u32, ergotree_ir::types::stype::SType, Vec<&'static str>)> = Vec::new();
        let scope: HashSet<u32> = HashSet::new();
        let mut path: Vec<&'static str> = Vec::new();
        collect_orphan_paths(expr, &scope, &mut path, &mut found);
        for (id, tpe, p) in &found {
            eprintln!(
                "[VU_PATH] stage={} orphan_id={} tpe={:?} path={:?}",
                stage, id, tpe, p
            );
        }
        if let Ok(target) = std::env::var("CSE_TRACE_VD_FOR") {
            if let Ok(target_id) = target.parse::<u32>() {
                let mut found2: Vec<(ergotree_ir::types::stype::SType, Vec<&'static str>)> =
                    Vec::new();
                let mut path2: Vec<&'static str> = Vec::new();
                collect_valdef_paths(expr, target_id, &mut path2, &mut found2);
                for (tpe, p) in &found2 {
                    eprintln!(
                        "[VD_PATH] stage={} target_id={} tpe={:?} path={:?}",
                        stage, target_id, tpe, p
                    );
                }
            }
        }
    }
    // QB-SESSION-13 / sig-15 gluon S7 — B2 trajectory probe.
    // Emits a per-stage tally of structurally-equal sibling-only ValDef
    // groups (B2 class) so we can localize WHICH pipeline stage first
    // produces gluon's 8 B2 sibling pairs. Per the S33 reframe, the
    // observed final B2 shape may be created at any of:
    //   - stage 00 (HIR→MIR lowering preserving user-source val replication)
    //   - stage 02 (`cse_expr` root extraction + replace_all sub-tree embedding)
    //   - stage 03 (`apply_cse_within_branches` per-branch extraction)
    //   - post-CSE renumbering / reordering (stages 05a..12)
    // The flag is env-gated (`CSE_TRACE_B2_TRAJECTORY=1`); default
    // behavior unchanged. Output per stage:
    //   [B2_TRAJ] stage=<name> total_vd=<N> b1=<X> b2=<Y> b3=<Z> same=<S>
    //   [B2_TRAJ]   group=<idx> count=<N> sib=<S> anc=<A> shape=<sig> depths=[...]
    if std::env::var("CSE_TRACE_B2_TRAJECTORY").is_ok() {
        let mut entries: Vec<(Vec<String>, u32, Expr)> = Vec::new();
        let mut path: Vec<String> = Vec::new();
        collect_valdefs_with_paths(expr, &mut path, &mut entries);
        let total = entries.len();
        let mut visited = vec![false; total];
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for i in 0..total {
            if visited[i] {
                continue;
            }
            let mut g = vec![i];
            visited[i] = true;
            for j in (i + 1)..total {
                if !visited[j] && entries[i].2 == entries[j].2 {
                    g.push(j);
                    visited[j] = true;
                }
            }
            groups.push(g);
        }
        let mut b1 = 0usize;
        let mut b2 = 0usize;
        let mut b3 = 0usize;
        let mut same = 0usize;
        let mut b2_groups: Vec<(usize, &Vec<usize>, usize, usize)> = Vec::new();
        for (gi, g) in groups.iter().enumerate() {
            if g.len() < 2 {
                continue;
            }
            let mut anc = 0usize;
            let mut sib = 0usize;
            for ai in 0..g.len() {
                for bi in (ai + 1)..g.len() {
                    let pa = &entries[g[ai]].0;
                    let pb = &entries[g[bi]].0;
                    if pa == pb {
                        // same scope (already-dedup'd, ignore)
                    } else if path_is_prefix(pa, pb) || path_is_prefix(pb, pa) {
                        anc += 1;
                    } else {
                        sib += 1;
                    }
                }
            }
            if sib > 0 && anc > 0 {
                b3 += 1;
            } else if sib > 0 {
                b2 += 1;
                b2_groups.push((gi, g, sib, anc));
            } else if anc > 0 {
                b1 += 1;
            } else {
                same += 1;
            }
        }
        eprintln!(
            "[B2_TRAJ] stage={:36} total_vd={:>3} b1={} b2={} b3={} same={}",
            stage, total, b1, b2, b3, same
        );
        for (gi, g, sib, anc) in &b2_groups {
            let shape = shape_signature(&entries[g[0]].2);
            let depths: Vec<usize> = g.iter().map(|&i| entries[i].0.len()).collect();
            eprintln!(
                "[B2_TRAJ]   g={:>2} count={} sib={} anc={} shape={} depths={:?}",
                gi,
                g.len(),
                sib,
                anc,
                shape,
                depths,
            );
        }
        // QB-SESSION-23 / sig-15 gluon S23 — Probe 1.2 hoist-safety extension.
        // For each B2 group, compute the orphan-ID set of the first
        // occurrence's RHS against the ROOT-block outer scope (top-level
        // BlockValue items). Reports whether hoisting that RHS to root scope
        // would create orphan ValUse refs (S31 gate would refuse). Gated on
        // CSE_TRACE_B2_HOIST_SAFETY=1; pure read.
        //
        // S23 empirical finding (35th falsification fingerprint instance, NEW
        // BIMODAL-EMERGENCE-YIELD-FALSIFIED-AT-HOIST-SAFETY subclass): post-S37
        // cascade gluon stage-12 inventory is 4 b2 pairs (down from S36
        // prediction of 6). Of those 4, only 1 is a real pre-renumber sibling
        // pair (g=61 stage-11 vd_ids=[73,76] trampoline rhs=ValUse(68)
        // referring to outer-scope ValDef). The other 3 (stage-12 g=73, g=78,
        // g=79 with vd_ids=[61,61] / [61,61] / [62,62]) are
        // sequential_renumber's per-branch counter-reset artifacts — their
        // inner ValUse refs are to semantically distinct bindings that
        // coincidentally share post-renumber IDs. Hoisting them would violate
        // scope semantics; the S31 orphan-id gate refuses them by construction
        // (orphan refs vs outer scope). Path B post-stage-12 merge yield
        // ceiling thus ~3-4B (1 trampoline pair), NOT the ~22B / 6-pair
        // handoff §0 prediction. Diag-only commit. See cluster archive
        // PENDING_S23_sig15-gluon-path-b-stage12-yield-falsified.md.
        if std::env::var("CSE_TRACE_B2_HOIST_SAFETY").is_ok() && !b2_groups.is_empty() {
            let root_scope: HashSet<u32> = if let Expr::BlockValue(bv) = expr {
                bv.expr
                    .items
                    .iter()
                    .filter_map(|i| match i {
                        Expr::ValDef(vd) => Some(vd.expr.id.0),
                        _ => None,
                    })
                    .collect()
            } else {
                HashSet::new()
            };
            for (gi, g, _sib, _anc) in &b2_groups {
                let rhs = &entries[g[0]].2;
                let mut orphans: Vec<u32> = Vec::new();
                collect_orphan_ids(rhs, &root_scope, &mut orphans);
                orphans.sort();
                orphans.dedup();
                let ids: Vec<u32> = g.iter().map(|&i| entries[i].1).collect();
                let safe = orphans.is_empty();
                eprintln!(
                    "[B2_TRAJ/SAFETY]   g={:>2} stage={} vd_ids={:?} root_orphans={:?} safe_to_root_hoist={} rhs={}",
                    gi,
                    stage,
                    ids,
                    orphans,
                    safe,
                    short_expr(rhs)
                );
            }
        }
    }
}

/// QB-SESSION-13 / sig-15 gluon S7 — durable trajectory-probe helper.
///
/// Walks the expression tree depth-first, recording for every `ValDef` node
/// (a) its structural scope-path (sequence of string fragments naming each
/// container encountered), (b) its declared `id`, and (c) a structural clone
/// of its `rhs`. Used by both `trace_slot_shift`'s `CSE_TRACE_B2_TRAJECTORY`
/// hook (per-stage B2 tally) and any future probe needing sibling-redundancy
/// inventory at intermediate pipeline stages. Mirrors the `walk` helper inside
/// `compiler.rs::probe_sig15_sibling_redundancy` but with a more compact
/// fragment encoding suitable for shipping in the CSE pipeline (the test-
/// only probe uses verbose discriminant strings for debugging output).
#[allow(dead_code)]
fn collect_valdefs_with_paths(
    expr: &Expr,
    path: &mut Vec<String>,
    out: &mut Vec<(Vec<String>, u32, Expr)>,
) {
    use ergotree_ir::traversable::Traversable;
    match expr {
        Expr::ValDef(s) => {
            let vd = &s.expr;
            out.push((path.clone(), vd.id.0, (*vd.rhs).clone()));
            path.push("VD.rhs".to_string());
            collect_valdefs_with_paths(&vd.rhs, path, out);
            path.pop();
        }
        Expr::BlockValue(s) => {
            let bv = &s.expr;
            for (i, it) in bv.items.iter().enumerate() {
                path.push(format!("BV.item[{}]", i));
                collect_valdefs_with_paths(it, path, out);
                path.pop();
            }
            path.push("BV.result".to_string());
            collect_valdefs_with_paths(&bv.result, path, out);
            path.pop();
        }
        Expr::If(s) => {
            path.push("If.cond".to_string());
            collect_valdefs_with_paths(&s.condition, path, out);
            path.pop();
            path.push("If.true".to_string());
            collect_valdefs_with_paths(&s.true_branch, path, out);
            path.pop();
            path.push("If.false".to_string());
            collect_valdefs_with_paths(&s.false_branch, path, out);
            path.pop();
        }
        Expr::FuncValue(fv) => {
            path.push("Fn.body".to_string());
            collect_valdefs_with_paths(fv.body(), path, out);
            path.pop();
        }
        other => {
            let tag = format!("{:?}", std::mem::discriminant(other));
            for (i, c) in <Expr as Traversable>::children(other).enumerate() {
                path.push(format!("{}.c[{}]", tag, i));
                collect_valdefs_with_paths(c, path, out);
                path.pop();
            }
        }
    }
}

#[allow(dead_code)]
fn path_is_prefix(a: &[String], b: &[String]) -> bool {
    a.len() <= b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y)
}

/// QB-SESSION-08 / Cohort B s2 post-stage-03 promotion.
///
/// Operates on the root BlockValue immediately after
/// `apply_cse_within_branches`. Detects sigmao_option's "branch CSE produced
/// a new occurrence of an already-extracted shape" event for shape
/// s2 = `ByIdx<or>[VU,K(Int),VU]`.
///
/// Predicate (Probe 1 / Session 8 P1.1):
/// - Fires only when the root BlockValue's items[] already contains >= 1
///   ValDef whose RHS matches the s2 shape (i.e. LOCAL already treats this
///   shape as extractable at outer scope at stage 02).
/// - For every sub-expression matching the shape that is NOT itself the
///   immediate rhs of an outer ValDef item, extract: append a new outer
///   ValDef and replace the inline occurrence with a fresh ValUse.
///
/// Cross-fixture safety (Probe 1 inventory across 15 sig-15 + ecosystem):
/// - Only sigmao_option satisfies the predicate (outer_vd_s2 = 5 at stage
///   02 with the +1 emergence at stage 03).
/// - paideia_stake_state has inline s2 occurrences but outer_vd_s2 = 0
///   throughout the pipeline — predicate doesn't fire.
/// - All MATCH fixtures (sigmausd / rosen / oracle / chaincash / dexy /
///   duckpools / ergomixer / ergoraffle / phoenix / spectrum / skyharbor)
///   have tree/outer_vd = 0/0 for s2 throughout — predicate doesn't fire.
/// - gluon_box_guard has 0/0 — predicate doesn't fire.
///
/// Skips nested BlockValue + FuncValue subtrees: extracting from those
/// scopes to root would create free-variable / out-of-scope ValUse refs.
///
/// S29 falsification (28th → 29th instance): inserted between stage 05b
/// (`inline_single_use_vals`) and stage 07 (`flatten_nested_blocks`),
/// the gate fires correctly (sigmao s2 outer_vd 5 → 6 = NODE parity 46;
/// SHAPE diff improves common-multiset 38 → 39, N-only 8 → 7) but
/// downstream `dfs_reassign_val_ids → reorder_valdefs → sequential_renumber`
/// pipeline cannot accept the new ValDef body's pre-renumber ValUse refs,
/// breaking constant pool segregation (constants_len 61 → 0; LOCAL falls
/// back to unsegregated 1005B; `probe_sig15_collisions` sigmao OK → FAIL).
/// Same renumbering-pipeline-crash class as S5 broad Fix A (#11) / S6
/// narrow strict-subset gate (#12). Confirms: byte-ADDING direction
/// hits the SAME architectural gate as byte-SHRINKING (S26/S27/S28).
/// Retained as durable artifact; re-enable when renumbering pipeline
/// rewrite (WS-G class) is in place.
#[allow(dead_code)]
fn promote_branch_emerged_s2(expr: Expr, next_id: &mut u32) -> Expr {
    const TARGET_SHAPE: &str = "ByIdx<or>[VU,K(Int),VU]";

    let s = match expr {
        Expr::BlockValue(s) => s,
        other => return other,
    };
    let inner = s.expr;
    let items = inner.items;
    let result = *inner.result;

    // Predicate: count outer ValDef RHS matches.
    let outer_vd_count: usize = items
        .iter()
        .filter(|i| {
            matches!(i, Expr::ValDef(vd) if shape_signature(&vd.expr.rhs) == TARGET_SHAPE)
        })
        .count();
    if outer_vd_count == 0 {
        return Expr::BlockValue(Spanned {
            source_span: s.source_span,
            expr: BlockValue {
                items,
                result: result.into(),
            },
        });
    }

    // S31 / Session 10 Probe 2: scope-soundness gate.
    // Collect outer scope's bound IDs (items[]'s ValDef ids). A hoisted
    // shape's free ValUse refs MUST all be in this set; otherwise the
    // new outer ValDef body references an inner-scope binding that
    // disambig can't resolve at outer scope, producing structurally
    // invalid IR (orphan ValUse at outer scope → segregation FAIL).
    let outer_scope: HashSet<u32> = items
        .iter()
        .filter_map(|i| match i {
            Expr::ValDef(vd) => Some(vd.expr.id.0),
            _ => None,
        })
        .collect();

    // QB-SESSION-11 / S32 Probe 1.2: closure-trace diagnostic. When
    // CSE_TRACE_S32_CLOSURE=1, walk all inner BlockValues to build
    // (id → body) map of inner ValDef bindings, then for each item +
    // result, collect orphan refs the gate would reject and print the
    // recursive closure of orphan-bound ValDef bodies. Pure diagnostic;
    // doesn't change extraction behavior.
    if std::env::var("CSE_TRACE_S32_CLOSURE").is_ok() {
        let mut inner_valdefs: std::collections::HashMap<u32, Expr> = Default::default();
        collect_inner_valdefs(&result, &outer_scope, &mut inner_valdefs);
        for item in &items {
            collect_inner_valdefs(item, &outer_scope, &mut inner_valdefs);
        }
        let mut all_match_orphans: Vec<u32> = Vec::new();
        for item in &items {
            collect_match_orphans(item, TARGET_SHAPE, &outer_scope, &mut all_match_orphans);
        }
        collect_match_orphans(&result, TARGET_SHAPE, &outer_scope, &mut all_match_orphans);
        all_match_orphans.sort();
        all_match_orphans.dedup();
        eprintln!(
            "[S32_CLOSURE] outer_scope_size={} inner_vd_map_size={} match_orphans={:?}",
            outer_scope.len(),
            inner_valdefs.len(),
            all_match_orphans
        );
        let mut closure: Vec<u32> = Vec::new();
        let mut worklist: Vec<u32> = all_match_orphans.clone();
        while let Some(id) = worklist.pop() {
            if closure.contains(&id) {
                continue;
            }
            closure.push(id);
            if let Some(body) = inner_valdefs.get(&id) {
                eprintln!(
                    "[S32_CLOSURE]   ValDef({}).body :: {}",
                    id,
                    short_expr(body)
                );
                let mut deeper: Vec<u32> = Vec::new();
                collect_orphan_ids(body, &outer_scope, &mut deeper);
                deeper.sort();
                deeper.dedup();
                eprintln!(
                    "[S32_CLOSURE]   ValDef({}).orphans_against_outer={:?}",
                    id, deeper
                );
                for d in deeper {
                    worklist.push(d);
                }
            } else {
                eprintln!(
                    "[S32_CLOSURE]   ValDef({}) NOT FOUND in inner_valdefs (deeper than walker can reach)",
                    id
                );
            }
        }
    }

    // QB-SESSION-11 / S32 Strategy I: recursive free-var lift. Under
    // `CSE_S32_LIFT=1`, BEFORE running the extraction pass, walk the outer
    // block to find every TARGET_SHAPE match site's orphan closure (orphan
    // ValDef bodies in inner BlockValue scopes), compute the transitive
    // dependency set, and LIFT those ValDefs to outer scope. Remove them
    // from their inner BlockValue items[]. Inner ValUse refs to lifted IDs
    // will resolve to the outer ValDef via the inner→outer scope-chain
    // walk in `disambiguate_val_ids`. After lift, the previously-orphaned
    // match sites become safe to hoist (their free refs are all in the
    // expanded outer scope).
    // Collect inner_valdefs map for BOTH Strategy I (lift) and Strategy II
    // (inline) so the lift logic and the inline-substitution logic can share
    // it. Idempotent build; pure-read inside the branches below.
    let mut inner_valdefs_for_strategy: std::collections::HashMap<u32, Expr> = Default::default();
    if std::env::var("CSE_S32_LIFT").is_ok() || std::env::var("CSE_S32_INLINE").is_ok() {
        for item in &items {
            collect_inner_valdefs(item, &outer_scope, &mut inner_valdefs_for_strategy);
        }
        collect_inner_valdefs(&result, &outer_scope, &mut inner_valdefs_for_strategy);
    }

    let (items, result, mut extracted, outer_scope) = if std::env::var("CSE_S32_LIFT").is_ok() {
        let inner_valdefs = &inner_valdefs_for_strategy;

        let mut match_orphans: Vec<u32> = Vec::new();
        for item in &items {
            collect_match_orphans(item, TARGET_SHAPE, &outer_scope, &mut match_orphans);
        }
        collect_match_orphans(&result, TARGET_SHAPE, &outer_scope, &mut match_orphans);
        match_orphans.sort();
        match_orphans.dedup();

        let mut closure_set: HashSet<u32> = Default::default();
        let mut worklist: Vec<u32> = match_orphans.clone();
        let mut unreachable = false;
        while let Some(id) = worklist.pop() {
            if closure_set.contains(&id) {
                continue;
            }
            if let Some(body) = inner_valdefs.get(&id) {
                closure_set.insert(id);
                let mut deeper: Vec<u32> = Vec::new();
                collect_orphan_ids(body, &outer_scope, &mut deeper);
                for d in deeper {
                    worklist.push(d);
                }
            } else {
                unreachable = true;
                break;
            }
        }

        if unreachable || closure_set.is_empty() {
            if std::env::var("CSE_TRACE_PROMOTE_S2").is_ok() {
                eprintln!(
                    "[PROMOTE_S2/lift] BAIL unreachable={} closure_size={} match_orphans={:?}",
                    unreachable, closure_set.len(), match_orphans
                );
            }
            (items, result, Vec::new(), outer_scope)
        } else {
            let mut closure_ids: Vec<u32> = closure_set.iter().copied().collect();
            closure_ids.sort();
            let lifted: Vec<Expr> = closure_ids
                .iter()
                .filter_map(|id| {
                    inner_valdefs.get(id).map(|body| {
                        Expr::ValDef(Spanned {
                            source_span: SourceSpan::empty(),
                            expr: ValDef {
                                id: ValId(*id),
                                rhs: body.clone().into(),
                            },
                        })
                    })
                })
                .collect();

            if std::env::var("CSE_TRACE_PROMOTE_S2").is_ok() {
                eprintln!(
                    "[PROMOTE_S2/lift] lifting closure_ids={:?} (size={})",
                    closure_ids,
                    closure_ids.len()
                );
            }

            let items_lifted: Vec<Expr> = items
                .into_iter()
                .map(|i| remove_valdefs_in_blocks(i, &closure_set))
                .collect();
            let result_lifted = remove_valdefs_in_blocks(result, &closure_set);

            let mut new_outer_scope = outer_scope;
            for id in &closure_ids {
                new_outer_scope.insert(*id);
            }
            // Lift goes into pre-extracted accumulator so it lands BEFORE
            // the new s2 ValDef in final items[]. topo_order_valdefs at
            // the bottom reorders to dependency-before-dependent.
            (items_lifted, result_lifted, lifted, new_outer_scope)
        }
    } else {
        (items, result, Vec::new(), outer_scope)
    };

    let processed_items: Vec<Expr> = items
        .into_iter()
        .map(|item| match item {
            Expr::ValDef(vd) => {
                let rhs = *vd.expr.rhs;
                // Leave existing outer extractions intact: walk into their
                // children but skip the rhs root.
                let new_rhs = if shape_signature(&rhs) == TARGET_SHAPE {
                    rhs
                } else {
                    extract_inline_shape(rhs, TARGET_SHAPE, next_id, &mut extracted, &outer_scope)
                };
                Expr::ValDef(Spanned {
                    source_span: vd.source_span,
                    expr: ValDef {
                        id: vd.expr.id,
                        rhs: new_rhs.into(),
                    },
                })
            }
            other => extract_inline_shape(other, TARGET_SHAPE, next_id, &mut extracted, &outer_scope),
        })
        .collect();
    let new_result = extract_inline_shape(result, TARGET_SHAPE, next_id, &mut extracted, &outer_scope);

    if extracted.is_empty() {
        return Expr::BlockValue(Spanned {
            source_span: s.source_span,
            expr: BlockValue {
                items: processed_items,
                result: new_result.into(),
            },
        });
    }

    let mut combined = processed_items;
    combined.extend(extracted);
    let final_items = topo_order_valdefs(combined);

    Expr::BlockValue(Spanned {
        source_span: s.source_span,
        expr: BlockValue {
            items: final_items,
            result: new_result.into(),
        },
    })
}

/// Helper for `promote_branch_emerged_s2`. Recursively walks `expr`. When a
/// sub-expression's shape signature matches `target_shape`, replace it with
/// a fresh `ValUse(new_id, T)` and append the original as a new ValDef to
/// `extracted`. Otherwise recurse into children.
///
/// Skips nested BlockValue / FuncValue boundaries — those are separate
/// scopes; extracting across them would break ValUse scoping.
#[allow(dead_code)]
fn extract_inline_shape(
    expr: Expr,
    target_shape: &str,
    next_id: &mut u32,
    extracted: &mut Vec<Expr>,
    outer_scope: &HashSet<u32>,
) -> Expr {
    if shape_signature(&expr) == target_shape {
        // S31 / Session 10 Probe 2: scope-soundness gate. Refuse to hoist
        // a shape whose free ValUse refs are not all bound in outer_scope —
        // hoisting would create an orphan ValUse at outer scope (disambig
        // can't resolve inner-scope bindings from outer scope chain →
        // segregation FAIL). Falsifies the S29 "renumbering-pipeline-crash"
        // framing: the failure is at the extraction predicate level, not
        // the renumbering pipeline.
        let mut orphans: Vec<u32> = Vec::new();
        collect_orphan_ids(&expr, outer_scope, &mut orphans);
        if !orphans.is_empty() {
            if std::env::var("CSE_TRACE_PROMOTE_S2").is_ok() {
                eprintln!(
                    "[PROMOTE_S2/unsafe] refused shape={} orphans={:?} expr={}",
                    target_shape,
                    orphans,
                    short_expr(&expr)
                );
            }
            // Fall through: still recurse into children so deeper safe
            // matches can be extracted.
            return match expr {
                Expr::FuncValue(_) => expr,
                other => map_children_extract_shape(other, target_shape, next_id, extracted, outer_scope),
            };
        }
        if std::env::var("CSE_TRACE_PROMOTE_S2").is_ok() {
            eprintln!(
                "[PROMOTE_S2/safe] extracting shape={} expr={}",
                target_shape,
                short_expr(&expr)
            );
        }
        let tpe = expr.tpe();
        let new_id = ValId(*next_id);
        *next_id += 1;
        extracted.push(Expr::ValDef(Spanned {
            source_span: SourceSpan::empty(),
            expr: ValDef {
                id: new_id,
                rhs: expr.into(),
            },
        }));
        return Expr::ValUse(ValUse {
            val_id: new_id,
            tpe,
        });
    }
    match expr {
        // FuncValue: don't cross lambda boundary (free var concerns).
        Expr::FuncValue(_) => expr,
        other => map_children_extract_shape(other, target_shape, next_id, extracted, outer_scope),
    }
}

/// Mirror of `map_children_with_id_mut` but threading the additional
/// `target_shape: &str` + `extracted: &mut Vec<Expr>` captures needed by
/// `extract_inline_shape`. Hand-coded because fn-pointer-based
/// `map_children_with_id_mut` can't capture additional state.
#[allow(dead_code)]
fn map_children_extract_shape(
    expr: Expr,
    target_shape: &str,
    next_id: &mut u32,
    extracted: &mut Vec<Expr>,
    outer_scope: &HashSet<u32>,
) -> Expr {
    macro_rules! r {
        ($e:expr) => {
            extract_inline_shape($e, target_shape, next_id, extracted, outer_scope)
        };
    }
    match expr {
        Expr::BlockValue(s) => Expr::BlockValue(Spanned {
            source_span: s.source_span,
            expr: BlockValue {
                items: s.expr.items.into_iter().map(|i| r!(i)).collect(),
                result: r!(*s.expr.result).into(),
            },
        }),
        Expr::ValDef(s) => Expr::ValDef(Spanned {
            source_span: s.source_span,
            expr: ValDef {
                id: s.expr.id,
                rhs: r!(*s.expr.rhs).into(),
            },
        }),
        Expr::BinOp(s) => Expr::BinOp(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::bin_op::BinOp {
                kind: s.expr.kind,
                left: r!(*s.expr.left).into(),
                right: r!(*s.expr.right).into(),
            },
        }),
        Expr::BoolToSigmaProp(bts) => {
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: r!(*bts.input).into(),
            })
        }
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: r!(*if_op.condition).into(),
            true_branch: r!(*if_op.true_branch).into(),
            false_branch: r!(*if_op.false_branch).into(),
        }),
        Expr::SizeOf(so) => Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
            input: r!(*so.input).into(),
        }),
        Expr::ExtractAmount(ea) => {
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: r!(*ea.input).into(),
            })
        }
        Expr::PropertyCall(s) => Expr::PropertyCall(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::property_call::PropertyCall {
                obj: r!(*s.expr.obj).into(),
                method: s.expr.method,
            },
        }),
        Expr::SigmaAnd(sa) => {
            let items: Vec<Expr> = sa.items.into_iter().map(|i| r!(i)).collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: items.try_into().expect("SigmaAnd >= 2"),
            })
        }
        Expr::SigmaOr(so) => {
            let items: Vec<Expr> = so.items.into_iter().map(|i| r!(i)).collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: items.try_into().expect("SigmaOr >= 2"),
            })
        }
        Expr::And(a) => Expr::And(Spanned {
            source_span: a.source_span,
            expr: ergotree_ir::mir::and::And {
                input: r!(*a.expr.input).into(),
            },
        }),
        Expr::Or(o) => Expr::Or(Spanned {
            source_span: o.source_span,
            expr: ergotree_ir::mir::or::Or {
                input: r!(*o.expr.input).into(),
            },
        }),
        Expr::Collection(c) => match c {
            ergotree_ir::mir::collection::Collection::Exprs { elem_tpe, items } => {
                let new_items: Vec<Expr> = items.into_iter().map(|i| r!(i)).collect();
                Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs {
                    elem_tpe,
                    items: new_items,
                })
            }
            other => Expr::Collection(other),
        },
        Expr::Tuple(t) => {
            let items: Vec<Expr> = t.items.into_iter().map(|i| r!(i)).collect();
            Expr::Tuple(ergotree_ir::mir::tuple::Tuple {
                items: items.try_into().expect("Tuple >= 2"),
            })
        }
        Expr::ByIndex(s) => {
            let input = r!(*s.expr.input);
            let index = r!(*s.expr.index);
            let default = s.expr.default.map(|d| Box::new(r!(*d)));
            ergotree_ir::mir::coll_by_index::ByIndex::new(input, index, default)
                .map(|bi| {
                    Expr::ByIndex(Spanned {
                        source_span: s.source_span,
                        expr: bi,
                    })
                })
                .expect("ByIndex::new in extract_inline_shape")
        }
        Expr::OptionGet(og) => {
            let input = r!(*og.expr.input);
            ergotree_ir::mir::option_get::OptionGet::try_build(input)
                .map(|x| {
                    Expr::OptionGet(Spanned {
                        source_span: og.source_span,
                        expr: x,
                    })
                })
                .expect("OptionGet in extract_inline_shape")
        }
        Expr::OptionGetOrElse(s) => {
            let input = r!(*s.expr.input);
            let default = r!(*s.expr.default);
            ergotree_ir::mir::option_get_or_else::OptionGetOrElse::new(input, default)
                .map(|x| {
                    Expr::OptionGetOrElse(Spanned {
                        source_span: s.source_span,
                        expr: x,
                    })
                })
                .expect("OptionGetOrElse in extract_inline_shape")
        }
        Expr::OptionIsDefined(s) => {
            let input = r!(*s.expr.input);
            ergotree_ir::mir::option_is_defined::OptionIsDefined::try_build(input)
                .map(|x| {
                    Expr::OptionIsDefined(Spanned {
                        source_span: s.source_span,
                        expr: x,
                    })
                })
                .expect("OptionIsDefined in extract_inline_shape")
        }
        Expr::ExtractRegisterAs(s) => {
            let input = r!(*s.expr.input);
            ergotree_ir::mir::extract_reg_as::ExtractRegisterAs::new(
                input,
                s.expr.register_id,
                ergotree_ir::types::stype::SType::SOption(s.expr.elem_tpe),
            )
            .map(|x| {
                Expr::ExtractRegisterAs(Spanned {
                    source_span: s.source_span,
                    expr: x,
                })
            })
            .expect("ExtractRegisterAs in extract_inline_shape")
        }
        Expr::ExtractScriptBytes(es) => {
            Expr::ExtractScriptBytes(ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes {
                input: r!(*es.input).into(),
            })
        }
        Expr::ExtractBytes(es) => {
            Expr::ExtractBytes(ergotree_ir::mir::extract_bytes::ExtractBytes {
                input: r!(*es.input).into(),
            })
        }
        Expr::ExtractBytesWithNoRef(es) => Expr::ExtractBytesWithNoRef(
            ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef {
                input: r!(*es.input).into(),
            },
        ),
        Expr::ExtractId(es) => {
            Expr::ExtractId(ergotree_ir::mir::extract_id::ExtractId {
                input: r!(*es.input).into(),
            })
        }
        Expr::ExtractCreationInfo(es) => {
            Expr::ExtractCreationInfo(ergotree_ir::mir::extract_creation_info::ExtractCreationInfo {
                input: r!(*es.input).into(),
            })
        }
        Expr::SelectField(s) => {
            let input = r!(*s.expr.input);
            ergotree_ir::mir::select_field::SelectField::new(input, s.expr.field_index)
                .map(|sf| {
                    Expr::SelectField(Spanned {
                        source_span: s.source_span,
                        expr: sf,
                    })
                })
                .expect("SelectField in extract_inline_shape")
        }
        Expr::LogicalNot(ln) => Expr::LogicalNot(Spanned {
            source_span: ln.source_span,
            expr: ergotree_ir::mir::logical_not::LogicalNot {
                input: r!(*ln.expr.input).into(),
            },
        }),
        Expr::Negation(n) => Expr::Negation(Spanned {
            source_span: n.source_span,
            expr: ergotree_ir::mir::negation::Negation {
                input: r!(*n.expr.input).into(),
            },
        }),
        Expr::CreateProveDlog(cpd) => {
            Expr::CreateProveDlog(ergotree_ir::mir::create_provedlog::CreateProveDlog {
                input: r!(*cpd.input).into(),
            })
        }
        Expr::SigmaPropBytes(sp) => {
            Expr::SigmaPropBytes(ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes {
                input: r!(*sp.input).into(),
            })
        }
        Expr::DecodePoint(dp) => Expr::DecodePoint(ergotree_ir::mir::decode_point::DecodePoint {
            input: r!(*dp.input).into(),
        }),
        Expr::CalcSha256(c) => Expr::CalcSha256(ergotree_ir::mir::calc_sha256::CalcSha256 {
            input: r!(*c.input).into(),
        }),
        Expr::CalcBlake2b256(c) => {
            Expr::CalcBlake2b256(ergotree_ir::mir::calc_blake2b256::CalcBlake2b256 {
                input: r!(*c.input).into(),
            })
        }
        Expr::Append(s) => Expr::Append(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_append::Append {
                input: r!(*s.expr.input).into(),
                col_2: r!(*s.expr.col_2).into(),
            },
        }),
        Expr::Slice(s) => Expr::Slice(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_slice::Slice {
                input: r!(*s.expr.input).into(),
                from: r!(*s.expr.from).into(),
                until: r!(*s.expr.until).into(),
            },
        }),
        Expr::Upcast(u) => {
            let tpe = u.tpe.clone();
            Expr::Upcast(
                ergotree_ir::mir::upcast::Upcast::new(r!(*u.input), tpe).expect("Upcast in extract"),
            )
        }
        Expr::Downcast(d) => {
            let tpe = d.tpe.clone();
            Expr::Downcast(
                ergotree_ir::mir::downcast::Downcast::new(r!(*d.input), tpe)
                    .expect("Downcast in extract"),
            )
        }
        // Leaves and unhandled compound types: leave intact. The hand-coded
        // recursion above covers every variant reachable from sigmao's
        // post-stage-03 root; expansion welcome when other fixtures land.
        leaf => leaf,
    }
}

fn collect_valdef_paths(
    expr: &Expr,
    target_id: u32,
    path: &mut Vec<&'static str>,
    out: &mut Vec<(ergotree_ir::types::stype::SType, Vec<&'static str>)>,
) {
    if let Expr::ValDef(s) = expr {
        if s.expr.id.0 == target_id {
            out.push((s.expr.rhs.tpe(), path.clone()));
        }
    }
    match expr {
        Expr::BlockValue(s) => {
            path.push("block");
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    if vd.expr.id.0 == target_id {
                        out.push((vd.expr.rhs.tpe(), path.clone()));
                    }
                    path.push("vd_rhs");
                    collect_valdef_paths(&vd.expr.rhs, target_id, path, out);
                    path.pop();
                } else {
                    collect_valdef_paths(item, target_id, path, out);
                }
            }
            path.push("result");
            collect_valdef_paths(&s.expr.result, target_id, path, out);
            path.pop();
            path.pop();
        }
        Expr::If(if_op) => {
            path.push("if_cond");
            collect_valdef_paths(&if_op.condition, target_id, path, out);
            path.pop();
            path.push("if_true");
            collect_valdef_paths(&if_op.true_branch, target_id, path, out);
            path.pop();
            path.push("if_false");
            collect_valdef_paths(&if_op.false_branch, target_id, path, out);
            path.pop();
        }
        Expr::FuncValue(fv) => {
            path.push("fn_body");
            collect_valdef_paths(fv.body(), target_id, path, out);
            path.pop();
        }
        Expr::ValDef(s) => {
            path.push("vd_rhs");
            collect_valdef_paths(&s.expr.rhs, target_id, path, out);
            path.pop();
        }
        other => {
            for child in direct_children(other) {
                collect_valdef_paths(child, target_id, path, out);
            }
        }
    }
}

fn collect_orphan_paths<'a>(
    expr: &'a Expr,
    scope: &HashSet<u32>,
    path: &mut Vec<&'static str>,
    out: &mut Vec<(u32, ergotree_ir::types::stype::SType, Vec<&'static str>)>,
) {
    match expr {
        Expr::ValUse(vu) => {
            if !scope.contains(&vu.val_id.0) {
                out.push((vu.val_id.0, vu.tpe.clone(), path.clone()));
            }
        }
        Expr::BlockValue(s) => {
            let mut block_scope = scope.clone();
            path.push("block");
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    path.push("vd_rhs");
                    collect_orphan_paths(&vd.expr.rhs, &block_scope, path, out);
                    path.pop();
                    block_scope.insert(vd.expr.id.0);
                } else {
                    collect_orphan_paths(item, &block_scope, path, out);
                }
            }
            path.push("result");
            collect_orphan_paths(&s.expr.result, &block_scope, path, out);
            path.pop();
            path.pop();
        }
        Expr::ValDef(s) => {
            path.push("vd_rhs");
            collect_orphan_paths(&s.expr.rhs, scope, path, out);
            path.pop();
        }
        Expr::FuncValue(fv) => {
            let mut body_scope = scope.clone();
            for arg in fv.args() {
                body_scope.insert(arg.idx.0);
            }
            path.push("fn_body");
            collect_orphan_paths(fv.body(), &body_scope, path, out);
            path.pop();
        }
        Expr::If(if_op) => {
            path.push("if_cond");
            collect_orphan_paths(&if_op.condition, scope, path, out);
            path.pop();
            path.push("if_true");
            collect_orphan_paths(&if_op.true_branch, scope, path, out);
            path.pop();
            path.push("if_false");
            collect_orphan_paths(&if_op.false_branch, scope, path, out);
            path.pop();
        }
        other => {
            let kind: &'static str = match other {
                Expr::OptionGet(_) => "OptionGet",
                Expr::SelectField(_) => "SelectField",
                Expr::SigmaAnd(_) => "SigmaAnd",
                Expr::SigmaOr(_) => "SigmaOr",
                Expr::BoolToSigmaProp(_) => "BoolToSigmaProp",
                Expr::BinOp(_) => "BinOp",
                Expr::Apply(_) => "Apply",
                Expr::MethodCall(_) => "MC",
                Expr::PropertyCall(_) => "PC",
                Expr::ExtractRegisterAs(_) => "ExtractReg",
                Expr::Filter(_) => "Filter",
                Expr::Map(_) => "Map",
                Expr::Fold(_) => "Fold",
                Expr::Exists(_) => "Exists",
                Expr::ForAll(_) => "ForAll",
                Expr::Collection(_) => "Collection",
                Expr::Tuple(_) => "Tuple",
                Expr::ByIndex(_) => "ByIndex",
                Expr::SizeOf(_) => "SizeOf",
                Expr::Append(_) => "Append",
                Expr::Slice(_) => "Slice",
                Expr::Upcast(_) => "Upcast",
                Expr::Downcast(_) => "Downcast",
                Expr::Negation(_) => "Negation",
                Expr::CreateProveDlog(_) => "CreateProveDlog",
                Expr::CreateProveDhTuple(_) => "CreateProveDhT",
                Expr::DecodePoint(_) => "DecodePoint",
                Expr::SigmaPropBytes(_) => "SigmaPropBytes",
                Expr::ExtractScriptBytes(_) => "ExtractScript",
                Expr::ExtractBytes(_) => "ExtractBytes",
                Expr::ExtractAmount(_) => "ExtractAmount",
                Expr::OptionGetOrElse(_) => "OptionGetOrElse",
                Expr::OptionIsDefined(_) => "OptionIsDefined",
                Expr::And(_) => "And",
                Expr::Or(_) => "Or",
                Expr::LogicalNot(_) => "LogicalNot",
                Expr::Atleast(_) => "Atleast",
                Expr::CalcBlake2b256(_) => "Blake2b",
                Expr::CalcSha256(_) => "Sha256",
                Expr::ByteArrayToBigInt(_) => "BAtoBigInt",
                Expr::ByteArrayToLong(_) => "BAtoLong",
                Expr::LongToByteArray(_) => "LongToBA",
                Expr::SubstConstants(_) => "SubstConstants",
                Expr::TreeLookup(_) => "TreeLookup",
                Expr::CreateAvlTree(_) => "CreateAvlTree",
                Expr::GetVar(_) => "GetVar",
                Expr::Const(_) => "Const",
                Expr::ConstPlaceholder(_) => "CP",
                Expr::GlobalVars(_) => "GV",
                _ => "_",
            };
            path.push(kind);
            for child in direct_children(other) {
                collect_orphan_paths(child, scope, path, out);
            }
            path.pop();
        }
    }
}

/// QB-SESSION-11 / S32 Strategy I helper. Recursively walks `expr` and
/// removes any `Expr::ValDef` whose id is in `to_remove` from BlockValue
/// items[] (and from any further nested BlockValues). If a BlockValue's
/// items becomes empty after removal, it is replaced by its result.
/// Used to lift inner-scope orphan ValDefs to outer scope.
#[allow(dead_code)]
fn remove_valdefs_in_blocks(expr: Expr, to_remove: &HashSet<u32>) -> Expr {
    match expr {
        Expr::BlockValue(s) => {
            let new_items: Vec<Expr> = s
                .expr
                .items
                .into_iter()
                .filter_map(|item| match &item {
                    Expr::ValDef(vd) if to_remove.contains(&vd.expr.id.0) => None,
                    _ => Some(remove_valdefs_in_blocks(item, to_remove)),
                })
                .collect();
            let new_result = remove_valdefs_in_blocks(*s.expr.result, to_remove);
            if new_items.is_empty() {
                new_result
            } else {
                Expr::BlockValue(Spanned {
                    source_span: s.source_span,
                    expr: BlockValue {
                        items: new_items,
                        result: new_result.into(),
                    },
                })
            }
        }
        Expr::ValDef(s) => Expr::ValDef(Spanned {
            source_span: s.source_span,
            expr: ValDef {
                id: s.expr.id,
                rhs: remove_valdefs_in_blocks(*s.expr.rhs, to_remove).into(),
            },
        }),
        other => map_children_remove(other, to_remove),
    }
}

/// Hand-coded child-map for `remove_valdefs_in_blocks` (captures `to_remove`,
/// so the fn-pointer-based `map_children_with_id_mut` is unsuitable).
/// Recurses into shapes that may carry nested BlockValue.
#[allow(dead_code)]
fn map_children_remove(expr: Expr, to_remove: &HashSet<u32>) -> Expr {
    macro_rules! r {
        ($e:expr) => {
            remove_valdefs_in_blocks($e, to_remove)
        };
    }
    match expr {
        Expr::BinOp(s) => Expr::BinOp(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::bin_op::BinOp {
                kind: s.expr.kind,
                left: r!(*s.expr.left).into(),
                right: r!(*s.expr.right).into(),
            },
        }),
        Expr::BoolToSigmaProp(bts) => {
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: r!(*bts.input).into(),
            })
        }
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: r!(*if_op.condition).into(),
            true_branch: r!(*if_op.true_branch).into(),
            false_branch: r!(*if_op.false_branch).into(),
        }),
        Expr::SizeOf(so) => Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
            input: r!(*so.input).into(),
        }),
        Expr::ExtractAmount(ea) => {
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: r!(*ea.input).into(),
            })
        }
        Expr::PropertyCall(s) => Expr::PropertyCall(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::property_call::PropertyCall {
                obj: r!(*s.expr.obj).into(),
                method: s.expr.method,
            },
        }),
        Expr::SigmaAnd(sa) => {
            let items: Vec<Expr> = sa.items.into_iter().map(|i| r!(i)).collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: items.try_into().expect("SigmaAnd >= 2"),
            })
        }
        Expr::SigmaOr(so) => {
            let items: Vec<Expr> = so.items.into_iter().map(|i| r!(i)).collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: items.try_into().expect("SigmaOr >= 2"),
            })
        }
        Expr::And(a) => Expr::And(Spanned {
            source_span: a.source_span,
            expr: ergotree_ir::mir::and::And {
                input: r!(*a.expr.input).into(),
            },
        }),
        Expr::Or(o) => Expr::Or(Spanned {
            source_span: o.source_span,
            expr: ergotree_ir::mir::or::Or {
                input: r!(*o.expr.input).into(),
            },
        }),
        Expr::Collection(c) => match c {
            ergotree_ir::mir::collection::Collection::Exprs { elem_tpe, items } => {
                let new_items: Vec<Expr> = items.into_iter().map(|i| r!(i)).collect();
                Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs {
                    elem_tpe,
                    items: new_items,
                })
            }
            other => Expr::Collection(other),
        },
        Expr::Tuple(t) => {
            let items: Vec<Expr> = t.items.into_iter().map(|i| r!(i)).collect();
            Expr::Tuple(ergotree_ir::mir::tuple::Tuple {
                items: items.try_into().expect("Tuple >= 2"),
            })
        }
        Expr::ByIndex(s) => {
            let input = r!(*s.expr.input);
            let index = r!(*s.expr.index);
            let default = s.expr.default.map(|d| Box::new(r!(*d)));
            ergotree_ir::mir::coll_by_index::ByIndex::new(input, index, default)
                .map(|bi| Expr::ByIndex(Spanned { source_span: s.source_span, expr: bi }))
                .expect("ByIndex in remove_valdefs_in_blocks")
        }
        Expr::OptionGet(og) => {
            let input = r!(*og.expr.input);
            ergotree_ir::mir::option_get::OptionGet::try_build(input)
                .map(|x| Expr::OptionGet(Spanned { source_span: og.source_span, expr: x }))
                .expect("OptionGet in remove_valdefs_in_blocks")
        }
        Expr::OptionGetOrElse(s) => {
            let input = r!(*s.expr.input);
            let default = r!(*s.expr.default);
            ergotree_ir::mir::option_get_or_else::OptionGetOrElse::new(input, default)
                .map(|x| Expr::OptionGetOrElse(Spanned { source_span: s.source_span, expr: x }))
                .expect("OptionGetOrElse in remove_valdefs_in_blocks")
        }
        Expr::OptionIsDefined(s) => {
            let input = r!(*s.expr.input);
            ergotree_ir::mir::option_is_defined::OptionIsDefined::try_build(input)
                .map(|x| Expr::OptionIsDefined(Spanned { source_span: s.source_span, expr: x }))
                .expect("OptionIsDefined in remove_valdefs_in_blocks")
        }
        Expr::ExtractRegisterAs(s) => {
            let input = r!(*s.expr.input);
            ergotree_ir::mir::extract_reg_as::ExtractRegisterAs::new(
                input,
                s.expr.register_id,
                ergotree_ir::types::stype::SType::SOption(s.expr.elem_tpe),
            )
            .map(|x| Expr::ExtractRegisterAs(Spanned { source_span: s.source_span, expr: x }))
            .expect("ExtractRegisterAs in remove_valdefs_in_blocks")
        }
        Expr::ExtractScriptBytes(es) => {
            Expr::ExtractScriptBytes(ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes {
                input: r!(*es.input).into(),
            })
        }
        Expr::ExtractBytes(es) => {
            Expr::ExtractBytes(ergotree_ir::mir::extract_bytes::ExtractBytes {
                input: r!(*es.input).into(),
            })
        }
        Expr::ExtractBytesWithNoRef(es) => Expr::ExtractBytesWithNoRef(
            ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef {
                input: r!(*es.input).into(),
            },
        ),
        Expr::ExtractId(es) => Expr::ExtractId(ergotree_ir::mir::extract_id::ExtractId {
            input: r!(*es.input).into(),
        }),
        // For other shapes that may carry nested BlockValues, fall back
        // via map_children_with_id_mut with an identity transform (which
        // descends into structures and lets us not propagate `to_remove`
        // into shapes that don't host BlockValue items[] anyway). The
        // shapes hand-coded above cover the sigmao_option AST surface
        // observed at S29; if additional fixtures trigger the lift path
        // we'll widen this dispatch.
        other => other,
    }
}

/// Diagnostic helper for `CSE_TRACE_S32_CLOSURE`. Walks `expr` and collects
/// every nested ValDef whose id is NOT in `outer_scope` (i.e. an inner-scope
/// binding) into `out[id → body]`. Children of FuncValue/BlockValue are
/// walked too. Pure-diagnostic; not used by extraction logic.
#[allow(dead_code)]
fn collect_inner_valdefs(
    expr: &Expr,
    outer_scope: &HashSet<u32>,
    out: &mut std::collections::HashMap<u32, Expr>,
) {
    match expr {
        Expr::ValDef(s) => {
            let id = s.expr.id.0;
            if !outer_scope.contains(&id) {
                out.insert(id, (*s.expr.rhs).clone());
            }
            collect_inner_valdefs(&s.expr.rhs, outer_scope, out);
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_inner_valdefs(item, outer_scope, out);
            }
            collect_inner_valdefs(&s.expr.result, outer_scope, out);
        }
        Expr::FuncValue(fv) => {
            collect_inner_valdefs(fv.body(), outer_scope, out);
        }
        other => {
            for child in direct_children(other) {
                collect_inner_valdefs(child, outer_scope, out);
            }
        }
    }
}

/// Diagnostic helper for `CSE_TRACE_S32_CLOSURE`. Walks `expr` looking for
/// sub-expressions whose shape signature matches `target_shape`. For each
/// match, computes free ValUse refs against `outer_scope` and appends them
/// to `out`. Pure-diagnostic.
#[allow(dead_code)]
fn collect_match_orphans(
    expr: &Expr,
    target_shape: &str,
    outer_scope: &HashSet<u32>,
    out: &mut Vec<u32>,
) {
    if shape_signature(expr) == target_shape {
        collect_orphan_ids(expr, outer_scope, out);
        return;
    }
    match expr {
        Expr::FuncValue(_) => {}
        Expr::ValDef(s) => collect_match_orphans(&s.expr.rhs, target_shape, outer_scope, out),
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_match_orphans(item, target_shape, outer_scope, out);
            }
            collect_match_orphans(&s.expr.result, target_shape, outer_scope, out);
        }
        other => {
            for child in direct_children(other) {
                collect_match_orphans(child, target_shape, outer_scope, out);
            }
        }
    }
}

fn collect_orphan_ids(expr: &Expr, scope: &HashSet<u32>, out: &mut Vec<u32>) {
    match expr {
        Expr::ValUse(vu) => {
            if !scope.contains(&vu.val_id.0) {
                out.push(vu.val_id.0);
            }
        }
        Expr::BlockValue(s) => {
            let mut block_scope = scope.clone();
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    collect_orphan_ids(&vd.expr.rhs, &block_scope, out);
                    block_scope.insert(vd.expr.id.0);
                } else {
                    collect_orphan_ids(item, &block_scope, out);
                }
            }
            collect_orphan_ids(&s.expr.result, &block_scope, out);
        }
        Expr::ValDef(s) => {
            collect_orphan_ids(&s.expr.rhs, scope, out);
        }
        Expr::FuncValue(fv) => {
            let mut body_scope = scope.clone();
            for arg in fv.args() {
                body_scope.insert(arg.idx.0);
            }
            collect_orphan_ids(fv.body(), &body_scope, out);
        }
        other => {
            for child in direct_children(other) {
                collect_orphan_ids(child, scope, out);
            }
        }
    }
}


// -----------------------------------------------------------------------
// Graph IR: processAstGraph port
// -----------------------------------------------------------------------

/// Build a DFS schedule of unique sub-expressions.
/// Mirrors the Scala compiler's `depthFirstOrderFrom(rootIds, neighbours)`.
/// Each unique expression appears exactly once, in dependency order
/// (children before parents).
fn dfs_schedule(expr: &Expr) -> Vec<Expr> {
    let mut visited: Vec<Expr> = Vec::new();
    let mut schedule: Vec<Expr> = Vec::new();
    dfs_visit(expr, &mut visited, &mut schedule);
    schedule
}

fn dfs_visit(expr: &Expr, visited: &mut Vec<Expr>, schedule: &mut Vec<Expr>) {
    // Skip if already visited (hash-consing: structurally equal = same node)
    if visited.iter().any(|v| v == expr) {
        return;
    }
    visited.push(expr.clone());

    // Visit children first (dependencies before dependents)
    for child in direct_children(expr) {
        dfs_visit(child, visited, schedule);
    }

    // Add this node to schedule after its dependencies
    schedule.push(expr.clone());
}

// -----------------------------------------------------------------------
// Scope-aware DAG helpers for branch-level CSE
// -----------------------------------------------------------------------
//
// Scala's `processAstGraph` runs once per ThunkDef scope. Inner If branches
// and And/Or right arms are separate ThunkDefs — their contents are opaque
// to the outer processAstGraph's DAG. These scope-aware helpers stop
// recursion at ThunkDef boundaries so dag counting matches Scala's model.

/// Like `collect_subexprs` but stops at inner-If branches and And/Or right arms.
#[allow(dead_code)]
fn collect_subexprs_scope(expr: &Expr, out: &mut Vec<Expr>) {
    if is_collectible(expr) {
        out.push(expr.clone());
    }
    match expr {
        // ThunkDef boundary: only recurse into the condition (eager), not branches
        Expr::If(if_op) => {
            collect_subexprs_scope(&if_op.condition, out);
        }
        // And/Or right arm is a ThunkDef: only recurse into left (eager)
        Expr::BinOp(s)
            if matches!(
                s.expr.kind,
                ergotree_ir::mir::bin_op::BinOpKind::Logical(
                    ergotree_ir::mir::bin_op::LogicalOp::And
                        | ergotree_ir::mir::bin_op::LogicalOp::Or
                )
            ) =>
        {
            collect_subexprs_scope(&s.expr.left, out);
        }
        Expr::BinOp(s) => {
            collect_subexprs_scope(&s.expr.left, out);
            collect_subexprs_scope(&s.expr.right, out);
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_subexprs_scope(item, out);
            }
            collect_subexprs_scope(&s.expr.result, out);
        }
        Expr::ValDef(s) => {
            collect_subexprs_scope(&s.expr.rhs, out);
        }
        Expr::BoolToSigmaProp(bts) => {
            collect_subexprs_scope(&bts.input, out);
        }
        Expr::PropertyCall(s) => {
            collect_subexprs_scope(&s.expr.obj, out);
        }
        Expr::MethodCall(s) => {
            collect_subexprs_scope(&s.expr.obj, out);
            for arg in &s.expr.args {
                collect_subexprs_scope(arg, out);
            }
        }
        Expr::ExtractAmount(ea) => collect_subexprs_scope(&ea.input, out),
        Expr::ExtractRegisterAs(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::ExtractScriptBytes(esb) => collect_subexprs_scope(&esb.input, out),
        Expr::ExtractBytes(eb) => collect_subexprs_scope(&eb.input, out),
        Expr::ExtractId(ei) => collect_subexprs_scope(&ei.input, out),
        Expr::ExtractCreationInfo(eci) => collect_subexprs_scope(&eci.input, out),
        Expr::SizeOf(so) => collect_subexprs_scope(&so.input, out),
        Expr::ByIndex(s) => {
            collect_subexprs_scope(&s.expr.input, out);
            collect_subexprs_scope(&s.expr.index, out);
            if let Some(ref d) = s.expr.default {
                collect_subexprs_scope(d, out);
            }
        }
        Expr::SelectField(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::OptionGet(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::OptionIsDefined(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::OptionGetOrElse(s) => {
            collect_subexprs_scope(&s.expr.input, out);
            collect_subexprs_scope(&s.expr.default, out);
        }
        Expr::Filter(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::Exists(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::ForAll(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::Map(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::Fold(s) => {
            collect_subexprs_scope(&s.expr.input, out);
            collect_subexprs_scope(&s.expr.zero, out);
        }
        Expr::Slice(s) => {
            collect_subexprs_scope(&s.expr.input, out);
            collect_subexprs_scope(&s.expr.from, out);
            collect_subexprs_scope(&s.expr.until, out);
        }
        Expr::LogicalNot(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::Negation(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::SigmaPropBytes(spb) => collect_subexprs_scope(&spb.input, out),
        Expr::Upcast(uc) => collect_subexprs_scope(&uc.input, out),
        Expr::Downcast(dc) => collect_subexprs_scope(&dc.input, out),
        Expr::CalcBlake2b256(cb) => collect_subexprs_scope(&cb.input, out),
        Expr::ByteArrayToBigInt(s) => collect_subexprs_scope(&s.expr.input, out),
        Expr::SigmaAnd(sa) => {
            for item in sa.items.iter() {
                collect_subexprs_scope(item, out);
            }
        }
        Expr::SigmaOr(so) => {
            for item in so.items.iter() {
                collect_subexprs_scope(item, out);
            }
        }
        Expr::Tuple(t) => {
            for item in t.items.iter() {
                collect_subexprs_scope(item, out);
            }
        }
        Expr::TreeLookup(s) => {
            collect_subexprs_scope(&s.expr.tree, out);
            collect_subexprs_scope(&s.expr.key, out);
            collect_subexprs_scope(&s.expr.proof, out);
        }
        Expr::Apply(app) => {
            collect_subexprs_scope(&app.func, out);
            for arg in &app.args {
                collect_subexprs_scope(arg, out);
            }
        }
        Expr::Append(s) => {
            collect_subexprs_scope(&s.expr.input, out);
            collect_subexprs_scope(&s.expr.col_2, out);
        }
        Expr::CreateProveDlog(cpd) => collect_subexprs_scope(&cpd.input, out),
        Expr::CreateProveDhTuple(cpd) => {
            collect_subexprs_scope(&cpd.g, out);
            collect_subexprs_scope(&cpd.h, out);
            collect_subexprs_scope(&cpd.u, out);
            collect_subexprs_scope(&cpd.v, out);
        }
        Expr::Atleast(s) => {
            collect_subexprs_scope(&s.bound, out);
            collect_subexprs_scope(&s.input, out);
        }
        Expr::And(a) => collect_subexprs_scope(&a.expr.input, out),
        Expr::Or(o) => collect_subexprs_scope(&o.expr.input, out),
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                collect_subexprs_scope(item, out);
            }
        }
        Expr::FuncValue(_) => {}
        Expr::Const(_)
        | Expr::ConstPlaceholder(_)
        | Expr::GlobalVars(_)
        | Expr::ValUse(_)
        | Expr::Context
        | Expr::Global => {}
        _ => {}
    }
}

/// Like `count_occurrences` but stops at inner-Thunk boundaries
/// (If branches, And/Or right arms). Mirrors Scala's
/// `subG.schedule` membership: a sym created inside a sibling Thunk is
/// not in the outer scope's schedule, so its uses there don't drive
/// outer-scope extraction decisions.
fn count_occurrences_scope(expr: &Expr, target: &Expr) -> usize {
    let mut count = if expr == target { 1 } else { 0 };
    for child in direct_children_scope(expr) {
        count += count_occurrences_scope(child, target);
    }
    count
}

/// True iff there exists a BlockValue strictly nested inside this scope's
/// items or result whose subtree contains ≥2 occurrences of `target`.
/// Such an inner BlockValue is its own scope where `extract_if_cond_shared`
/// will fire when `pre_extract_from_valdefs` recurses — extracting at THIS
/// scope would hijack the inner pass, producing a ValDef at the wrong scope.
///
/// In Scala terms: when all global occurrences of a candidate sit inside an
/// inner ThunkDef's BlockValue body, the sym's `bodyIds` is that inner
/// BlockValue's, not this scope's. Suppress rescue extraction here so the
/// inner pass can place the ValDef where Scala does.
///
/// Used to close OpenOrders 2B trailing ByIndex extract: at the outer scope,
/// `ByIndex(VU(15), Const(0))` appears 2× inside the outer If's true_branch
/// BlockValue and 0× elsewhere — defer to that branch's pass.
/// Preserves ProxyBorrow's PropertyCall(VU(N), tokens) extraction: that
/// candidate's occurrences are inside the right arm of `&&`, which has no
/// BlockValue body — there's no deeper scope to defer to, so extract here.
fn deeper_block_with_ge_two_occurrences(items: &[Expr], result: &Expr, target: &Expr) -> bool {
    // Compute total count over this scope's tree (excluding the synthetic
    // ValDef wrappers — which would double-count if the wrapper itself
    // matched, but in practice ValDef is never a CSE target).
    let total: usize = items
        .iter()
        .map(|i| {
            if let Expr::ValDef(vd) = i {
                count_occurrences(&vd.expr.rhs, target)
            } else {
                count_occurrences(i, target)
            }
        })
        .sum::<usize>()
        + count_occurrences(result, target);

    fn walk(expr: &Expr, target: &Expr, total: usize) -> bool {
        if let Expr::BlockValue(_) = expr {
            let c = count_occurrences(expr, target);
            // Only defer when ALL global occurrences are concentrated in
            // this single deeper block. If uses are split across this scope
            // and a deeper block, extracting HERE allows the inner block
            // to share via outer ValUse (Scala's LCA placement).
            if c >= 2 && c == total {
                return true;
            }
        }
        for c in direct_children(expr) {
            if walk(c, target, total) {
                return true;
            }
        }
        false
    }
    for item in items {
        if let Expr::ValDef(vd) = item {
            if walk(&vd.expr.rhs, target, total) {
                return true;
            }
        } else if walk(item, target, total) {
            return true;
        }
    }
    walk(result, target, total)
}

/// Whether `expr` contains any `ValUse(id)` for an id in `local_ids`.
/// Used as the "capture-set reaches this scope's local items" signal:
/// a candidate that references a local ValDef is anchored to this
/// scope in Scala's IR (the dep edge keeps the candidate's sym in
/// scope when free-vars are resolved).
fn expr_references_any_local(expr: &Expr, local_ids: &std::collections::HashSet<u32>) -> bool {
    match expr {
        Expr::ValUse(vu) => local_ids.contains(&vu.val_id.0),
        _ => direct_children(expr)
            .iter()
            .any(|c| expr_references_any_local(c, local_ids)),
    }
}

/// Like `direct_children` but stops at ThunkDef boundaries.
/// For If: returns only `condition`. For And/Or BinOp: returns only `left`.
#[allow(dead_code)]
fn direct_children_scope(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::If(ite) => vec![&ite.condition],
        Expr::BinOp(s)
            if matches!(
                s.expr.kind,
                ergotree_ir::mir::bin_op::BinOpKind::Logical(
                    ergotree_ir::mir::bin_op::LogicalOp::And
                        | ergotree_ir::mir::bin_op::LogicalOp::Or
                )
            ) =>
        {
            vec![&s.expr.left]
        }
        _ => direct_children(expr),
    }
}

/// Like `count_dag_usages` but uses scope-restricted traversal.
/// Stops at inner-If branches and And/Or right arms so references inside
/// sub-ThunkDefs don't inflate dag counts at the current ThunkDef level.
#[allow(dead_code)]
fn count_dag_usages_scope(expr: &Expr) -> Vec<(Expr, usize)> {
    let mut all_subexprs: Vec<Expr> = Vec::new();
    collect_subexprs_scope(expr, &mut all_subexprs);

    let mut unique: Vec<Expr> = Vec::new();
    for sub in &all_subexprs {
        if !unique.iter().any(|u| u == sub) {
            unique.push(sub.clone());
        }
    }

    let mut parent_sets: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); unique.len()];

    let root_children = direct_children_scope(expr);
    for child in &root_children {
        if let Some(child_idx) = unique.iter().position(|u| u == *child) {
            parent_sets[child_idx].insert(unique.len());
        }
    }

    for (parent_idx, parent) in unique.iter().enumerate() {
        let children = direct_children_scope(parent);
        for child in children {
            if let Some(child_idx) = unique.iter().position(|u| u == child) {
                parent_sets[child_idx].insert(parent_idx);
            }
        }
    }

    if std::env::var("CSE_DEBUG_PARENTS").is_ok() {
        for (idx, e) in unique.iter().enumerate() {
            let parents = &parent_sets[idx];
            if parents.len() >= 2 {
                eprintln!(
                    "  PARENTS of unique[{}] (count={}) {:.180?}",
                    idx,
                    parents.len(),
                    e
                );
                let mut sorted_parents: Vec<usize> = parents.iter().copied().collect();
                sorted_parents.sort();
                for p in &sorted_parents {
                    if *p == unique.len() {
                        eprintln!("    parent=ROOT");
                    } else {
                        eprintln!("    parent[{}] {:.220?}", p, unique[*p]);
                    }
                }
            }
        }
    }

    unique
        .into_iter()
        .zip(parent_sets)
        .map(|(expr, parents)| (expr, parents.len()))
        .collect()
}

/// Like `dfs_schedule` but uses scope-restricted traversal.
#[allow(dead_code)]
fn dfs_schedule_scope(expr: &Expr) -> Vec<Expr> {
    let mut visited: Vec<Expr> = Vec::new();
    let mut schedule: Vec<Expr> = Vec::new();
    dfs_visit_scope(expr, &mut visited, &mut schedule);
    schedule
}

#[allow(dead_code)]
fn dfs_visit_scope(expr: &Expr, visited: &mut Vec<Expr>, schedule: &mut Vec<Expr>) {
    if visited.iter().any(|v| v == expr) {
        return;
    }
    visited.push(expr.clone());
    for child in direct_children_scope(expr) {
        dfs_visit_scope(child, visited, schedule);
    }
    schedule.push(expr.clone());
}

/// Can this expression be extracted as a ValDef?
/// Mirrors the Scala compiler's filters in processAstGraph:
///   !IsContextProperty && !IsInternalDef && !IsConstantDef
fn is_extractable(expr: &Expr) -> bool {
    // S54: SBigInt Const literals are extractable. Other Const types
    // (SInt/SLong/SBoolean/...) are still inline-only — `is_collectible`
    // already gates collection to SBigInt, so this matches that gate.
    if let Expr::Const(c) = expr {
        return matches!(c.tpe, ergotree_ir::types::stype::SType::SBigInt);
    }
    !matches!(
        expr,
        Expr::ConstPlaceholder(_)
        // IsContextProperty: HEIGHT, INPUTS, OUTPUTS, SELF are always inline
        | Expr::GlobalVars(_)
        // Leaf references
        | Expr::ValUse(_) | Expr::Context | Expr::Global
        // Structural nodes not extracted as ValDefs
        | Expr::FuncValue(_) | Expr::BlockValue(_) | Expr::ValDef(_)
    )
}

/// Recursively rebuild an expression, replacing env entries in children.
fn build_value_recurse(expr: &Expr, env: &[(Expr, u32)]) -> Expr {
    // For each child, check env first, then recurse
    let resolve = |e: &Expr| -> Expr {
        for (env_expr, val_id) in env {
            if e == env_expr {
                return Expr::ValUse(ValUse {
                    val_id: ValId(*val_id),
                    tpe: e.tpe(),
                });
            }
        }
        build_value_recurse(e, env)
    };

    match expr {
        Expr::PropertyCall(s) => {
            let new_obj = resolve(&s.expr.obj);
            Expr::PropertyCall(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::property_call::PropertyCall {
                    obj: new_obj.into(),
                    method: s.expr.method.clone(),
                },
            })
        }
        Expr::MethodCall(s) => {
            let new_obj = resolve(&s.expr.obj);
            let new_args: Vec<Expr> = s.expr.args.iter().map(&resolve).collect();
            Expr::MethodCall(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::method_call::MethodCall {
                    obj: new_obj.into(),
                    method: s.expr.method.clone(),
                    args: new_args,
                    explicit_type_args: s.expr.explicit_type_args.clone(),
                },
            })
        }
        Expr::BinOp(s) => {
            let new_left = resolve(&s.expr.left);
            let new_right = resolve(&s.expr.right);
            Expr::BinOp(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::bin_op::BinOp {
                    kind: s.expr.kind,
                    left: new_left.into(),
                    right: new_right.into(),
                },
            })
        }
        Expr::ExtractAmount(ea) => {
            let new_input = resolve(&ea.input);
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: new_input.into(),
            })
        }
        Expr::ExtractRegisterAs(s) => {
            let new_input = resolve(&s.expr.input);
            Expr::ExtractRegisterAs(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::extract_reg_as::ExtractRegisterAs {
                    input: new_input.into(),
                    register_id: s.expr.register_id,
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        Expr::ExtractScriptBytes(esb) => {
            let new_input = resolve(&esb.input);
            Expr::ExtractScriptBytes(ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes {
                input: new_input.into(),
            })
        }
        Expr::ExtractBytes(eb) => {
            let new_input = resolve(&eb.input);
            Expr::ExtractBytes(ergotree_ir::mir::extract_bytes::ExtractBytes {
                input: new_input.into(),
            })
        }
        Expr::ExtractId(ei) => {
            let new_input = resolve(&ei.input);
            Expr::ExtractId(ergotree_ir::mir::extract_id::ExtractId {
                input: new_input.into(),
            })
        }
        Expr::ExtractCreationInfo(eci) => {
            let new_input = resolve(&eci.input);
            Expr::ExtractCreationInfo(
                ergotree_ir::mir::extract_creation_info::ExtractCreationInfo {
                    input: new_input.into(),
                },
            )
        }
        Expr::SizeOf(so) => {
            let new_input = resolve(&so.input);
            Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
                input: new_input.into(),
            })
        }
        Expr::ByIndex(s) => {
            let new_input = resolve(&s.expr.input);
            let new_index = resolve(&s.expr.index);
            let new_default = s.expr.default.as_ref().map(|d| resolve(d).into());
            ergotree_ir::mir::coll_by_index::ByIndex::new(new_input, new_index, new_default)
                .map(|bi| {
                    Expr::ByIndex(Spanned {
                        source_span: s.source_span,
                        expr: bi,
                    })
                })
                .unwrap_or_else(|_| expr.clone())
        }
        Expr::SelectField(s) => {
            let new_input = resolve(&s.expr.input);
            ergotree_ir::mir::select_field::SelectField::new(new_input, s.expr.field_index)
                .map(|sf| {
                    Expr::SelectField(Spanned {
                        source_span: s.source_span,
                        expr: sf,
                    })
                })
                .unwrap_or_else(|_| expr.clone())
        }
        Expr::OptionGet(s) => {
            let new_input = resolve(&s.expr.input);
            <ergotree_ir::mir::option_get::OptionGet as OneArgOpTryBuild>::try_build(new_input)
                .map(|og| {
                    Expr::OptionGet(Spanned {
                        source_span: s.source_span,
                        expr: og,
                    })
                })
                .unwrap_or_else(|_| expr.clone())
        }
        Expr::OptionIsDefined(s) => {
            let new_input = resolve(&s.expr.input);
            Expr::OptionIsDefined(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::option_is_defined::OptionIsDefined {
                    input: new_input.into(),
                },
            })
        }
        Expr::LogicalNot(s) => {
            let new_input = resolve(&s.expr.input);
            Expr::LogicalNot(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::logical_not::LogicalNot {
                    input: new_input.into(),
                },
            })
        }
        Expr::Negation(s) => {
            let new_input = resolve(&s.expr.input);
            Expr::Negation(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::negation::Negation {
                    input: new_input.into(),
                },
            })
        }
        Expr::BoolToSigmaProp(bsp) => {
            let new_input = resolve(&bsp.input);
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: new_input.into(),
            })
        }
        Expr::Upcast(uc) => {
            let new_input = resolve(&uc.input);
            Expr::Upcast(ergotree_ir::mir::upcast::Upcast {
                input: new_input.into(),
                tpe: uc.tpe.clone(),
            })
        }
        Expr::CalcBlake2b256(cb) => {
            let new_input = resolve(&cb.input);
            Expr::CalcBlake2b256(ergotree_ir::mir::calc_blake2b256::CalcBlake2b256 {
                input: new_input.into(),
            })
        }
        Expr::SigmaPropBytes(spb) => {
            let new_input = resolve(&spb.input);
            Expr::SigmaPropBytes(ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes {
                input: new_input.into(),
            })
        }
        Expr::CreateProveDlog(cpd) => {
            let new_input = resolve(&cpd.input);
            Expr::CreateProveDlog(ergotree_ir::mir::create_provedlog::CreateProveDlog {
                input: new_input.into(),
            })
        }
        Expr::If(ite) => {
            let new_cond = resolve(&ite.condition);
            let new_then = resolve(&ite.true_branch);
            let new_else = resolve(&ite.false_branch);
            Expr::If(ergotree_ir::mir::if_op::If {
                condition: new_cond.into(),
                true_branch: new_then.into(),
                false_branch: new_else.into(),
            })
        }
        Expr::SigmaAnd(sa) => {
            let new_items: Vec<Expr> = sa.items.iter().map(&resolve).collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: new_items.try_into().expect("SigmaAnd items"),
            })
        }
        Expr::SigmaOr(so) => {
            let new_items: Vec<Expr> = so.items.iter().map(&resolve).collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: new_items.try_into().expect("SigmaOr items"),
            })
        }
        Expr::Map(s) => {
            let new_input = resolve(&s.expr.input);
            // Don't resolve inside mapper (it's a FuncValue / lambda)
            Expr::Map(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_map::Map {
                    input: new_input.into(),
                    mapper: s.expr.mapper.clone(),
                    mapper_sfunc: s.expr.mapper_sfunc.clone(),
                },
            })
        }
        Expr::Filter(s) => {
            let new_input = resolve(&s.expr.input);
            Expr::Filter(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_filter::Filter {
                    input: new_input.into(),
                    condition: s.expr.condition.clone(),
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        Expr::Fold(s) => {
            let new_input = resolve(&s.expr.input);
            let new_zero = resolve(&s.expr.zero);
            Expr::Fold(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_fold::Fold {
                    input: new_input.into(),
                    zero: new_zero.into(),
                    fold_op: s.expr.fold_op.clone(),
                },
            })
        }
        Expr::Exists(s) => {
            let new_input = resolve(&s.expr.input);
            Expr::Exists(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_exists::Exists {
                    input: new_input.into(),
                    condition: s.expr.condition.clone(),
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        Expr::ForAll(s) => {
            let new_input = resolve(&s.expr.input);
            Expr::ForAll(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_forall::ForAll {
                    input: new_input.into(),
                    condition: s.expr.condition.clone(),
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        // Leaf nodes and structural nodes — return as-is
        _ => expr.clone(),
    }
}

/// Apply `process_ast_graph` independently to each If-branch in the tree.
///
/// Scala's `processAstGraph` runs once per ThunkDef scope: the root scope
/// plus each If branch (and &&/|| right arms). This means expressions that
/// appear 2+ times *within a single branch* get extracted as ValDefs inside
/// that branch. Our root-level CSE pass only handles the root scope; branch-
/// only expressions are blocked by the `appears_in_main_scope` check.
///
/// This pass walks the post-root-CSE tree, finds every `If` node, and runs
/// `process_ast_graph` on each branch independently with the current
/// `global_max` to avoid ID conflicts with outer-scope vals.
fn apply_cse_within_branches(expr: Expr, global_max: u32) -> Expr {
    match expr {
        Expr::If(if_op) => {
            // Apply branch-level CSE to each branch using scope-aware dag counting
            // so inner-If contents don't inflate counts at this ThunkDef level.
            let true_cse = process_ast_graph_branch(*if_op.true_branch, global_max);
            let false_cse = process_ast_graph_branch(*if_op.false_branch, global_max);
            // Recurse into results for nested Ifs, bumping global_max past
            // any IDs assigned in both branches.
            let new_max = global_max
                .max(find_max_val_id(&true_cse))
                .max(find_max_val_id(&false_cse));
            let true_final = apply_cse_within_branches(true_cse, new_max);
            let false_final = apply_cse_within_branches(false_cse, new_max);
            let cond_final = apply_cse_within_branches(*if_op.condition, new_max);
            Expr::If(ergotree_ir::mir::if_op::If {
                condition: cond_final.into(),
                true_branch: true_final.into(),
                false_branch: false_final.into(),
            })
        }
        // QB1 Phase 3.4b / S27 — Sub-scope dispatch on &&/|| right arms when
        // v2 hash-cons enabled. Mirrors Scala's processAstGraph recursion
        // into the right-arm ThunkDef of And/Or (left arm is eager, same
        // scope as parent). v2's per-sym `count >= 2 AND sym_home == root`
        // gate at the branch dispatch's fresh SymTable extracts syms whose
        // sym_home is that thunk's bodyDefs — what S26 metals identified as
        // Scala's behaviour (hasManyUsagesGlobal per-sym at sub-scope
        // BlockValue).
        //
        // Gated to v2: HC=0 default sig-15 12/15 must stay sacred. v1's
        // existing branch-CSE only dispatches into If arms; extending it
        // unconditionally would re-shape HC=0 extraction.
        Expr::BinOp(s)
            if hash_cons_v2_enabled()
                && matches!(
                    s.expr.kind,
                    ergotree_ir::mir::bin_op::BinOpKind::Logical(
                        ergotree_ir::mir::bin_op::LogicalOp::And
                            | ergotree_ir::mir::bin_op::LogicalOp::Or
                    )
                ) =>
        {
            let span = s.source_span;
            let kind = s.expr.kind;
            let left = *s.expr.left;
            let right = *s.expr.right;
            let left_final = apply_cse_within_branches(left, global_max);
            let right_cse = process_ast_graph_branch(right, global_max);
            let new_max = global_max.max(find_max_val_id(&right_cse));
            let right_final = apply_cse_within_branches(right_cse, new_max);
            Expr::BinOp(Spanned {
                source_span: span,
                expr: ergotree_ir::mir::bin_op::BinOp {
                    kind,
                    left: left_final.into(),
                    right: right_final.into(),
                },
            })
        }
        // QB1 Phase 3.4b / S28 Arc A — FuncValue body sub-scope dispatch when
        // v2 hash-cons enabled. Mirrors Scala's `buildValue` Lambda arm which
        // invokes `processAstGraph(...lam...)` on the Lambda's sub-AstGraph,
        // emitting ValDefs at the FuncValue body's BlockValue for syms first-
        // constructed inside the body (sym_home == this_lambda_subscope) with
        // hasManyUsagesGlobal ≥ 2. sigma-rust FuncValue.body: Box<Expr>
        // accepts BlockValue (verified via Probe 0 metals); no IR invariant
        // blocks the wrap.
        //
        // S28 Probe 1.2 finding: dexy / paideia / duckpools — the three v2
        // residual fixtures — contain ZERO FuncValue nodes (validated by
        // grepping their .es sources for fold/forall/exists/map/=>). Arc A
        // is therefore null-target for the S27 residuals. Its function in
        // S28 is soundness validation across lambda-containing fixtures
        // (chaincash / ergomixer / ergoraffle / oracle / rosen / sigmausd /
        // skyharbor — 7 of 8 MATCH-held; phoenix has no lambda). MATCH-held
        // preservation under FuncValue dispatch documents the architectural
        // primitive as sound at the lambda boundary; S29 can then layer
        // recursive sub-sub-scope dispatch on it (paideia +6B class).
        Expr::FuncValue(fv) if hash_cons_v2_enabled() => {
            let args = fv.args().to_vec();
            let body = fv.body().clone();
            let body_cse = process_ast_graph_branch(body, global_max);
            let new_max = global_max.max(find_max_val_id(&body_cse));
            let body_final = apply_cse_within_branches(body_cse, new_max);
            Expr::FuncValue(FuncValue::new(args, body_final))
        }
        // QB1 Phase 3.4b / S29 — DIAG-ONLY (43rd falsification fingerprint).
        // Handoff §0 hypothesised paideia +6B / dexy +18B / duckpools +2B
        // need a B.i-A2 "recursive sub-sub-scope" arm (re-invoking
        // process_ast_graph_branch INTO nested ThunkDefs of each per-arm
        // fresh SymTable). Probe 0 metals on TreeBuilding.processAstGraph
        // confirmed Scala does (a) recursive invocation (subG → buildValue →
        // processAstGraph on nested Lambda / ThunkDef) — so the architectural
        // shape is sound. BUT Probe 1.2 + Probe 1.3 (CSE_TRACE_CANONICAL +
        // CSE_TRACE_HC_V2 on paideia/dexy at HEAD c3112b34) EMPIRICALLY
        // PRE-FALSIFY the under-extraction framing:
        //
        //   - 9 of 15 dispatches per paideia run already fire recursively
        //     via the If / BinOp(Logical) / FuncValue arms above; each
        //     per-thunk fresh SymTable extracts the candidate set at
        //     sym_home=0 of that sub-dispatch with `[HCv2/Branch] extract`
        //     trace lines confirming actual ValDef emission.
        //   - Paideia's csym=12/14/15/19 land at sym_home=1/2 in the OUTER
        //     BinOp right-arm dispatch (skipped by S24 gate) AND at sym_home=0
        //     with extracted_root_gate=true in Dispatch 4 + Dispatch 5
        //     (the inner If true/false branch sub-dispatches). They are
        //     emitted TWICE — once per sibling sub-scope — instead of ONCE
        //     at the cross-sibling LCA (Dispatch 2 root).
        //   - Dexy's csym=22-26 chain (sym_home=3, scopes=[3,4,6,7]) has
        //     the same shape across 4-5 sibling sub-scopes → ~4× ValDef
        //     replication → +18B accounts.
        //   - Duckpools's csym=8/9/15 (S28 Probe 1.3) is the smaller cousin.
        //
        // Class synthesis: dexy/paideia/duckpools v2 residuals are a UNIFIED
        // OVER-extraction class — per-sub-scope dispatch emits one ValDef
        // per sibling sub-scope where Scala's mainG.hasManyUsagesGlobal +
        // findGlobalDefinition emits ONE ValDef at LCA-of-uses (the cross-
        // sibling common ancestor). NOT under-extraction; NOT recursive-
        // dispatch under-coverage. The β.1 outer-LCA promotion attempt
        // (S25, 42nd falsification) is the right direction architecturally
        // but its cross-fixture cost is large (5-of-8 MATCH-held regressed)
        // — closure requires Scala's full first-DFS-construction-scope
        // hash-cons (WS-G QB1 full migration), not an incremental gate.
        //
        // S30+ disposition: NO B.i-A2 arm landed (premise falsified at probe).
        // Pivot to (i) characterising the cross-sibling LCA-of-uses set
        // empirically per fixture without coupling to MATCH-held regressions,
        // or (ii) accepting the three v2 residuals (+6 / +18 / +2 = +26B
        // total across 15 fixtures) as a permanent plateau under v2 until
        // the full hash-cons migration. The metals envelope (5 reads
        // through TreeBuilding/Thunks/AstGraphs/Base) is exhaustive for
        // sub-scope dispatch; further metals adds nothing without a
        // distinct mechanism question.
        //
        // Recurse into all other expression types using the generic child mapper
        other => map_children_with_id(other, global_max, apply_cse_within_branches),
    }
}

/// True if `expr` depends on any runtime context — a `ValUse`, a
/// `GlobalVars` (HEIGHT, INPUTS, OUTPUTS, SELF), `Context`, `Global`, or a
/// `GetVar`. Pure-constant expressions (only `Const`/`ConstPlaceholder`
/// transitively) return false; those are safe to hoist to root because
/// their value doesn't depend on where they're evaluated.
fn touches_context(expr: &Expr) -> bool {
    match expr {
        Expr::ValUse(_) | Expr::GlobalVars(_) | Expr::Context | Expr::Global | Expr::GetVar(_) => {
            true
        }
        _ => direct_children(expr).into_iter().any(touches_context),
    }
}

/// Like `touches_context` but treats the `Global` singleton as pure (it is a
/// hash-consed singleton sym in Scala's graph IR — its uses are tracked
/// globally across ThunkDef scopes). Used to recognise candidates whose only
/// "context" touch is `Global` (e.g. `proveDlog(groupGenerator)`), so they
/// can be hoisted to root scope under the permissive `appears_outside_if_branches`
/// check rather than strict `appears_in_main_scope`.
fn touches_runtime_context(expr: &Expr) -> bool {
    match expr {
        Expr::ValUse(_) | Expr::GlobalVars(_) | Expr::Context | Expr::GetVar(_) => true,
        Expr::Global => false,
        _ => direct_children(expr).into_iter().any(touches_runtime_context),
    }
}

/// S19: pure-const aggregate shape — `Tuple` or `Collection` whose every
/// leaf is `Const | ConstPlaceholder | Collection` (recursively). Used to
/// gate HC=1 Branch over-extractions where Scala's first-DFS-construction
/// scope places the aggregate inline at branch use sites.
fn is_pure_const_shape(expr: &Expr) -> bool {
    match expr {
        Expr::Tuple(t) => t.items.iter().all(is_pure_const_shape),
        Expr::Collection(c) => match c {
            ergotree_ir::mir::collection::Collection::BoolConstants(_) => true,
            ergotree_ir::mir::collection::Collection::Exprs { items, .. } => {
                items.iter().all(is_pure_const_shape)
            }
        },
        Expr::Const(_) | Expr::ConstPlaceholder(_) => true,
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ScopeMode {
    /// Outermost ThunkDef — extracted ValDefs land at root scope.
    Root,
    /// Inside an If true/false branch — extracted ValDefs land at branch scope.
    Branch,
}

/// Compact debug-string for a candidate expression. Truncated for trace output.
fn short_expr(e: &Expr) -> String {
    let s = format!("{:?}", e);
    if s.len() > 160 {
        format!("{}…(len={})", &s[..160], s.len())
    } else {
        s
    }
}

/// Port of the Scala compiler's `processAstGraph` — full-tree variant.
/// Used for root-scope CSE where the entire tree is one ThunkDef.
fn process_ast_graph(expr: Expr, global_max_id: u32) -> Expr {
    let dag_usages = count_dag_usages(&expr);
    let schedule = dfs_schedule(&expr);
    if hash_cons_v2_enabled() {
        process_ast_graph_hash_cons_v2(expr, global_max_id, dag_usages, schedule, ScopeMode::Root)
    } else if hash_cons_enabled() {
        process_ast_graph_hash_cons(expr, global_max_id, dag_usages, schedule, ScopeMode::Root)
    } else {
        process_ast_graph_impl(expr, global_max_id, dag_usages, schedule, ScopeMode::Root)
    }
}

/// Branch-level variant of `process_ast_graph`.
///
/// Base counts are scope-restricted (don't see across deeper-If ThunkDef
/// boundaries) — same as before — but augmented with a "cross-branch
/// dominator" pass: for each inner-If at this branch scope, expressions
/// present in BOTH arms get count≥2 here, so they get extracted at this
/// scope rather than living inline in each arm. This mirrors Scala's
/// behaviour: `mainG.hasManyUsagesGlobal` sees the cross-arm shared symbol
/// as having ≥2 uses, and TreeBuilding extracts it at the surrounding
/// `processAstGraph` scope.
///
/// `branch_local_ids` filtering inside `process_ast_graph_impl` rejects any
/// cross-branch candidate whose RHS references a ValDef defined inside a
/// deeper-If's arm (forward-ref guard).
fn process_ast_graph_branch(expr: Expr, global_max_id: u32) -> Expr {
    // Schedule = scope-restricted: only expressions first-created at this
    // scope's main level (mirroring Scala's `subG.flatSchedule`).
    let mut schedule = dfs_schedule_scope(&expr);

    // Counts: scope-restricted base, with cross-branch dominators bumped
    // to ≥ 2. A cross-branch dominator is a sub-expression appearing in
    // BOTH arms of an inner-If at this scope; extracting it here turns
    // two inline copies into one ValDef + two ValUses (typically a net
    // byte saving, and matches Scala's behaviour of hash-consing across
    // sibling ThunkDefs into the surrounding scope).
    //
    // We only bump entries already present in the scope-restricted
    // schedule — adding new schedule entries for cross-branch-only
    // expressions over-extracts compared to node (Scala's
    // `subG.flatSchedule` doesn't contain such expressions).
    let mut dag_usages = count_dag_usages_scope(&expr);
    let raw_cross = collect_cross_branch_dominators(&expr);
    // Filter: in Scala's hash-consed graph, a sub-expression c that is shared
    // across arms inherits its sym-parent count from the unified scope. If c
    // has exactly one dominator-set parent X (i.e. every appearance of c in
    // the arms is wrapped by X, which is itself shared), then c's sym has
    // only ONE sym-parent (X) — count=1 in Scala — so it should NOT be
    // extracted at this scope. X handles the sharing. Only keep c when its
    // dominator-parent count is 0 (c is "top-level" within the arms relative
    // to the dom set) or >= 2 (c has multiple distinct sym parents, all of
    // which are themselves cross-shared).
    let cross: Vec<Expr> = raw_cross
        .iter()
        .filter(|c| {
            let dom_parent_count = raw_cross
                .iter()
                .filter(|p| *p != *c)
                .filter(|p| direct_children(p).contains(c))
                .count();
            dom_parent_count != 1
        })
        .cloned()
        .collect();
    for cb_expr in cross {
        if let Some(entry) = dag_usages.iter_mut().find(|(e, _)| *e == cb_expr) {
            if entry.1 < 2 {
                entry.1 = 2;
            }
        }
    }

    // Global bump (S40):
    //
    // Scala's `mainG.hasManyUsagesGlobal(s)` checks the GLOBAL parent count
    // of any sym in this scope's `bodyIds`. Our scope-restricted
    // `count_dag_usages_scope` undercounts when a candidate is also referenced
    // inside sibling Thunks (which Scala's global walk would see).
    //
    // For each `dag_usages` entry already reachable in scope, bump count to
    // max(scope_count, global_occurrence_count) when global_occ >= 2 — but
    // skip BinOps. BinOps are deduped separately by Phase 3 (`rescue=true`),
    // and re-bumping them here over-extracts in cases where the BinOp
    // references a local ValDef that node leaves inline (e.g. SaleLP's
    // `BinOp(Minus, ValUse(10), 1)` at global_occ=2). Non-BinOp candidates
    // at global_occ=2 (OptionGet, ExtractRegisterAs, PropertyCall, ByIndex)
    // do match node's extraction decisions and close ~25B on SigUSDV1.
    //
    // The earlier `>= 3` threshold (S39) was overly conservative — it left
    // 25B of SigUSDV1 extractions unrealized because most of its
    // global_occ=2 candidates are non-BinOp and node DOES extract them.
    for (cand, count) in dag_usages.iter_mut() {
        if matches!(cand, Expr::BinOp(_)) {
            continue;
        }
        // Use count_occurrences_no_inner_if so that:
        // - Occurrences inside &&/|| right arms ARE counted (skyharbor OUTPUTS(2)
        //   appears in both arms of the royalty && — Scala counts this globally).
        // - Occurrences inside nested If true/false branches are NOT counted.
        //   Those branches are child ThunkDef scopes that get their own
        //   process_ast_graph_branch pass; counting into them here inflates the
        //   count for the surrounding scope (e.g. SaleLP OUTPUTS(4) appearing
        //   inside an inlined If(isLastSale,...) nested in the false branch).
        let global_occ = count_occurrences_no_inner_if(&expr, cand);
        if global_occ >= 2 && global_occ > *count {
            *count = global_occ;
        }
    }

    // S59 — Cross-condition-branch seeding (§2b SelectField fix).
    //
    // Scala's graph IR places sub-expressions used in BOTH an inner If's
    // condition (eager part, evaluated at the surrounding scope) AND that
    // If's true/false branch (a ThunkDef) into the SURROUNDING scope's
    // bodyIds. `findOrCreateDefinition` finds the sym in parent scope when
    // the inner Thunk references it, so the sym's hash-cons collapses both
    // uses to one Sym whose hasManyUsagesGlobal == true → extracted at the
    // surrounding scope (THIS branch).
    //
    // Our `count_dag_usages_scope` stops at If branches and `&&`/`||` right
    // arms, so sub-expressions whose only structural occurrences are inside
    // those Thunks never enter dag_usages. The S40 global bump only updates
    // existing entries — it can't help here. Seed missing entries explicitly.
    //
    // Safety: we only seed candidates that span across an If's
    // condition/branch boundary (a "cross-cond-branch" pattern). Sub-exprs
    // confined to a single Thunk (e.g. all uses inside one true_branch) are
    // NOT seeded here — those are handled by the inner If's own
    // process_ast_graph_branch pass. The branch_local_ids check inside
    // process_ast_graph_impl still rejects any candidate referencing a
    // ValDef defined inside a deeper If arm, so forward-ref hoisting is
    // impossible.
    let cond_branch_shared = collect_cond_branch_shared(&expr);
    // Reverse pre-order = post-order ⇒ children appear before parents in
    // the schedule, matching dfs_schedule's topo order.
    for cb in cond_branch_shared.into_iter().rev() {
        if !is_collectible(&cb) || !is_extractable(&cb) {
            continue;
        }
        if matches!(cb, Expr::BinOp(_)) {
            continue;
        }
        let global_occ = count_occurrences(&expr, &cb);
        if global_occ < 2 {
            continue;
        }
        if dag_usages.iter().any(|(e, _)| *e == cb) {
            continue;
        }
        dag_usages.push((cb.clone(), global_occ));
        if !schedule.iter().any(|s| s == &cb) {
            schedule.push(cb);
        }
    }

    if std::env::var("CSE_DEBUG").is_ok() {
        eprintln!(
            "\n=== process_ast_graph_branch (global_max_id={}) ===",
            global_max_id
        );
        eprintln!("--- branch root expr (full Debug):\n{:?}", expr);
        eprintln!("--- scope schedule ({} entries):", schedule.len());
        for (i, n) in schedule.iter().enumerate() {
            eprintln!("  [{}] {:.220?}", i, n);
        }
        eprintln!("--- dag_usages_scope ({} entries):", dag_usages.len());
        for (e, c) in &dag_usages {
            eprintln!("  count={} {:.220?}", c, e);
        }
        eprintln!("=== end branch dump ===\n");
    }

    // S12 PROBE — env-gated narrow-predicate falsification (QB Session 12).
    // Tests whether suppressing branch-local CSE extraction of
    // `Expr::BinOp(_, Relation(_), _)` and `Expr::ValUse(_)` RHS shapes
    // (the two unique-to-gluon B2 RHS classes per S5/S12 cross-fixture
    // inventory) moves gluon Δ +102 without regressing 12 MATCH fixtures.
    //
    // Per Probe 0 (re-anchored at HEAD): Scala has NO `apply_cse_within_branches`
    // analog; sibling-equal sym extraction-vs-inline is decided at graph-construction
    // time by `findGlobalDefinition` (first-DFS-construction scope claim) — not
    // observable from post-CSE Expr tree. Per S22, HC=1 makes identical decisions
    // on gluon's 8 B2 groups, ruling out the scope-chain layer.
    //
    // This filter removes the matching shapes from `dag_usages` before
    // `process_ast_graph_impl` would extract them. If MATCH fixtures hold
    // → narrow predicate works. If MATCH fixtures regress → P2.3
    // (same architectural class as sigmao, confirmed; surgical surface empty).
    if std::env::var("CSE_PROBE_S12").is_ok() {
        dag_usages.retain(|(cand, _)| {
            !matches!(cand, Expr::ValUse(_))
                && !matches!(
                    cand,
                    Expr::BinOp(b) if matches!(b.expr.kind,
                        ergotree_ir::mir::bin_op::BinOpKind::Relation(_))
                )
        });
    }

    if hash_cons_v2_enabled() {
        process_ast_graph_hash_cons_v2(expr, global_max_id, dag_usages, schedule, ScopeMode::Branch)
    } else if hash_cons_enabled() {
        process_ast_graph_hash_cons(expr, global_max_id, dag_usages, schedule, ScopeMode::Branch)
    } else {
        process_ast_graph_impl(expr, global_max_id, dag_usages, schedule, ScopeMode::Branch)
    }
}

/// Collect sub-expressions that appear in BOTH arms of some inner-If reachable
/// from `expr` without crossing another scope boundary along the way to that
/// If. Each If contributes its own intersection of true_branch ⋂ false_branch
/// sub-expressions; nested Ifs inside a branch contribute their own
/// intersections recursively (they live at the same outer scope as far as
/// `processAstGraph` is concerned, since both arms execute at the same
/// scope-relative position).
#[allow(dead_code)]
fn collect_cross_branch_dominators(expr: &Expr) -> Vec<Expr> {
    let mut out: Vec<Expr> = Vec::new();
    fn walk(expr: &Expr, out: &mut Vec<Expr>) {
        if let Expr::If(if_op) = expr {
            let mut true_subs: Vec<Expr> = Vec::new();
            collect_subexprs(&if_op.true_branch, &mut true_subs);
            let mut false_subs: Vec<Expr> = Vec::new();
            collect_subexprs(&if_op.false_branch, &mut false_subs);
            for e in &true_subs {
                if false_subs.iter().any(|f| f == e) && !out.iter().any(|o| o == e) {
                    out.push(e.clone());
                }
            }
            // Recurse: an If nested inside true/false branch is also a
            // cross-branch source, but its candidates land at the *inner*
            // branch's scope when that branch is processed via the branch
            // recursion in `apply_cse_within_branches` — so don't double-count
            // by walking deeper here.
            return;
        }
        for c in direct_children(expr) {
            walk(c, out);
        }
    }
    walk(expr, &mut out);
    out
}

/// Collect sub-expressions that appear in BOTH an inner-If's CONDITION and
/// at least one of its branches (true_branch or false_branch). The walk
/// stops at If branches (so nested Ifs inside a branch belong to that
/// branch's own scope and aren't reported here), but recurses into
/// conditions to handle nested Ifs in conditions. Walking uses
/// `direct_children` (not scope-restricted) so the walker reaches Ifs
/// nested inside `&&`/`||` right arms — Scala places those Ifs at the
/// surrounding scope when the surrounding `&&` is itself just a sym
/// reference (the common case for inlined right arms post-CSE).
///
/// Used by `process_ast_graph_branch` to seed dag_usages with candidates
/// that should be extracted at this scope (matching Scala's
/// eager-condition + branch-thunk model). See §2b in S55/S58 handoffs.
fn collect_cond_branch_shared(expr: &Expr) -> Vec<Expr> {
    let mut out: Vec<Expr> = Vec::new();
    fn walk(expr: &Expr, out: &mut Vec<Expr>) {
        if let Expr::If(if_op) = expr {
            let mut cond_subs: Vec<Expr> = Vec::new();
            collect_subexprs(&if_op.condition, &mut cond_subs);
            let mut true_subs: Vec<Expr> = Vec::new();
            collect_subexprs(&if_op.true_branch, &mut true_subs);
            let mut false_subs: Vec<Expr> = Vec::new();
            collect_subexprs(&if_op.false_branch, &mut false_subs);
            for e in &cond_subs {
                let in_true = true_subs.iter().any(|t| t == e);
                let in_false = false_subs.iter().any(|f| f == e);
                if (in_true || in_false) && !out.iter().any(|o| o == e) {
                    out.push(e.clone());
                }
            }
            // Recurse into the condition to handle nested Ifs in cond.
            // Don't recurse into branches — those are inner scope.
            walk(&if_op.condition, out);
            return;
        }
        for c in direct_children(expr) {
            walk(c, out);
        }
    }
    walk(expr, &mut out);
    out
}

// S58 — Pre-v3 ergotree Upcast(Const, _) handling note
// =====================================================
// Pre-v3 ErgoTrees (header version 0..=2) emit Upcast(Const, _) ValDef RHSs in a
// post-round-trip shape, NOT the "natural" shape produced by processAstGraph.
// Specifically:
//
//   PRE-segregation SValue:    val v9 = Upcast(Const(100000:SLong), SBigInt)  tpe SBigInt
//                              site uses: ValUse(9, SBigInt)
//
//   POST-segregation SValue:   val v9 = Const(100000:SLong)                   tpe SLong
//                              site uses: Upcast(ValUse(9, SLong), SBigInt)
//
// The transform happens during ErgoTree::new's serialize+reparse round-trip
// (ergotree-ir/src/ergo_tree.rs ErgoTree::new, mirror of Scala
// ErgoTree.withSegregation), driven by:
//
//   1. ValueSerializer.serializable() (Scala data/.../ValueSerializer.scala:154-166):
//      strips Upcast wrapper when serializing a Value — but ONLY effective when
//      the Upcast wraps a Constant (the strip + the `case c: Constant` arm in
//      ValueSerializer.serialize together emit just the ConstantPlaceholder bytes;
//      Upcasts wrapping ValUse / MethodCall / etc. fall through to `case _ =>`
//      which uses the ORIGINAL v.opCode and preserves the wrapper).
//
//   2. TransformingSigmaBuilder.applyUpcast (Scala data/.../SigmaBuilder.scala:751,
//      and DeserializationSigmaBuilder override): on parse, when an
//      arith/comparison op's operands have mismatched numeric types, inserts
//      Upcast at the smaller-typed operand. Disabled for v3+.
//
// Rust mirror: sigma_serialize for Expr::Upcast(Const, _) emits bare Const
// placeholder bytes (Site 1, ergotree-ir/src/serialization/expr.rs);
// bin_op_sigma_parse re-inserts Upcast at use-site arith/comparison ops
// (Site 2, ergotree-ir/src/serialization/bin_op.rs). CSE itself does NOT
// need to know about this — keep extracting Upcast(Const, T) as the candidate;
// the round-trip in ErgoTree::new reshapes the tree before any final byte emission.
//
// If a new contract diverges from node bytes with a similar wrapper-vs-bare
// issue, suspect another pre-v3 transform not yet mirrored: audit
// ByIndexSerializer (default-arg shape), MethodCallSerializer (typeSubst).

/// Shared implementation: given pre-computed dag_usages and schedule,
/// select multi-use nodes for extraction and build the result BlockValue.
//
// WS-G.1 diagnosis (2026-05-10) — placement strategy diverges from Scala.
//
// Scala's `processAstGraph` (sigma.compiler.ir.TreeBuilding) iterates each
// AstGraph's *local* `subG.schedule` and emits a ValDef at THIS scope only
// when (a) `mainG.hasManyUsagesGlobal(s)` (flat schedule, `.syms`) AND (b) the
// sym lives in this scope's `domain`. The sym's owning scope is fixed at
// graph-construction time by `ThunkScope.findDef → findGlobalDefinition`
// (sigma.compiler.ir.primitives.Thunks): the FIRST ThunkScope that constructs
// an equivalent Def claims it; sibling thunks reach the sym via
// `findGlobalDefinition` but do NOT add it to their own `bodyDefs`. At
// `processAstGraph` time, sibling scopes' `curEnv` therefore doesn't bind
// the shared sym; their `buildValue` env-misses and inline-rebuilds.
//
// Net Scala behaviour: a cross-scope shared sym emits as ValDef in its
// FIRST-DFS-CONSTRUCTION SCOPE (deeper than the LCA of all uses). Other
// scopes inline-rebuild; `ConstantStore::put` then dedups any literal
// `Const` children of the inline copies, partially compensating size.
//
// Rust here uses two separable scope modes:
//
//   - `ScopeMode::Root`: extract globally-shared candidates at the
//     OUTERMOST eligible scope (root BlockValue), then propagate ValUse
//     refs inward via `replace_all`. Cross-thunk shared candidates that
//     pass the scope checks become a single ValDef + N ValUses.
//
//   - `ScopeMode::Branch` (called via `apply_cse_within_branches`): each
//     If branch runs an INDEPENDENT CSE pass over its own subtree. No
//     cross-branch coordination — sibling branches that share a candidate
//     each extract it locally, producing N ValDefs for one Scala graph-IR
//     sym.
//
// Empirical divergence at HEAD (G.1 archive
// `<commit>_ws-g1-paideia-gluon-shared-mechanism-diagnosis.md`):
//
//   - paideia +2: `[PAG/Root] extract id=82 dag_count=3 :: Tuple([Coll[Byte](),
//     0L])` hoists the cross-branch-shared Tuple to root (LOCAL val 33);
//     Scala-NODE places at val 48 inside `if_true` (first-DFS scope). Δ +2
//     from constant-pool dedup pattern (LOCAL 77 / NODE 79 entries).
//
//   - gluon +102: zero Root extractions; many `[PAG/Branch]` extractions
//     for the SAME shape across nested If arms (id=238 BinOp 9×; id=242
//     Slice 5×). NODE has ONE graph-level sym per shared shape.
//
// Both fixtures share an underlying architectural cause: Rust's MIR uses
// tree-position CSE with retroactive dedup; Scala's IR uses DAG-identity
// hash-cons at construction. Closing both requires graph-IR migration of
// `process_ast_graph_*` (~200-400 LOC), NOT a single LCA-aware placement
// decision — Scala has no LCA primitive in AstGraphs/ProgramGraphs/Thunks/
// TreeBuilding, so an "LCA placement" mechanism on the Rust side wouldn't
// match what NODE actually does. See the WS-G.2 reframing in
// `WORKSTREAM-G-HANDOFF.md` for the migration design space.
//
// WS-G.2.1 design probe (2026-05-10) — Scala sym-table data model + migration
// target.  Probe 0 (metals: Base, Thunks, AstGraphs, TreeBuilding) confirmed
// the following data model; G.2.2 must build a Rust analogue:
//
// === Scala data model ===
//
//   1. Equivalence relation (sigma.compiler.ir.Base.Node):
//      `Def.equals = Arrays.deepEquals(elements, other.elements)` where
//      `elements = [getClass, productElement(0), productElement(1), ...]`.
//      Two separately-constructed Defs with identical structure are equals()
//      and hash to the same value. Rust analogue: `Expr: PartialEq` (already
//      structural) + `Expr: Hash` (to be added in G.2.2).
//
//   2. Global hash-cons table (Base._globalDefs: AVHashMap[Def, Def]):
//      `findGlobalDefinition(d)` returns an existing sym if one with equal
//      structure exists. `createDefinition` without a scope adds to
//      `_globalDefs`. All `reifyObject` calls OUTSIDE any ThunkScope go
//      through this global table — every structurally-unique Def gets ONE sym
//      at root scope. Rust analogue: `HashMap<ExprKey, SymId>` as the
//      `SymTable`'s root tier.
//
//   3. ThunkScope chain (Thunks.ThunkScope):
//      Each ThunkDef (if-branch, &&/|| right operand, lambda body) gets its
//      own `ThunkScope { parent, bodyDefs: AVHashMap[Def, Def], bodyIds }`.
//      `findDef(d)` walks: current.bodyDefs → parent.findDef → ... →
//      `findGlobalDefinition`. Creating a new sym inside scope S adds it to
//      S.bodyDefs only, NOT to any ancestor or `_globalDefs`. Sibling scopes
//      A and B do NOT share bodyDefs; equivalent Defs first-constructed in A
//      and B become SEPARATE syms, each living in its own domain.
//
//   4. First-DFS-construction-scope ownership:
//      When `reifyObject(d)` is called during graph-building, `ThunkScope.findDef`
//      searches upward. If found in an ancestor scope (including global) →
//      the existing sym is returned; no new sym created. If NOT found → a new
//      sym is created and added to the CURRENT scope's bodyDefs (or
//      `_globalDefs` if no ThunkScope active). Therefore: the owning scope of
//      sym S = the FIRST scope (in source-order graph-building DFS) that
//      constructed the equivalent Def.
//      - paideia Tuple: first constructed inside if_true's ThunkDef body (source
//        order: if_true branch precedes if_false). Sym lives in if_true.bodyDefs.
//        `processAstGraph` for root's `subG.schedule` does NOT include this sym
//        (not in root's domain). `processAstGraph` for if_true sees it, emits
//        ONE ValDef at val 48.
//      - gluon BinOp/Slice: first constructed at ROOT scope (referenced from a
//        val declaration evaluated before the If-branch ThunkDefs). Sym in
//        `_globalDefs`. Root `processAstGraph` sees it, emits ONE ValDef.
//        All branches reference via curEnv; no per-branch re-extraction.
//
//   5. `processAstGraph` per-scope iteration:
//      Iterates `subG.schedule` (LOCAL scope, not flatSchedule). Counts via
//      `mainG.hasManyUsagesGlobal` (global flat-schedule, `.syms`). A sym emits
//      ValDef in scope T iff: (a) `hasManyUsagesGlobal == true`, AND (b) sym
//      is in T's domain (was first-constructed in T). `curEnv` threading
//      propagates parent ValDef bindings into nested scopes; sibling scopes do
//      NOT share curEnv.
//
// === Rust current structure ===
//
//   `count_dag_usages`: CORRECT for global edge-incidence counting (S9
//   confirmed). Equivalent to flatSchedule+.syms buildUsageMap. NOT replaced
//   by migration — provides the usage threshold check.
//
//   `process_ast_graph_impl(ScopeMode::Root)`: candidates with dag_count>=2
//   are extracted at OUTERMOST eligible scope. For pure-const candidates (no
//   ValUse/GlobalVars), `needs_check=false` bypasses scope gate entirely →
//   paideia Tuple hoisted to root instead of staying at if_true.
//
//   `apply_cse_within_branches`: processes each If-branch INDEPENDENTLY via
//   separate `process_ast_graph_branch` calls. No cross-branch coordination.
//   Shared BinOps/Slices with local dag_count>=2 get extracted PER-BRANCH →
//   gluon's 9× / 5× redundant ValDef overhead.
//
// === G.2.2 migration target ===
//
//   Required: `struct SymTable { scope_parents: Vec<Option<usize>>,
//   table: HashMap<ExprKey, (ScopeId, SymId)>, next_sym: u32 }` and
//   `impl ExprKey` (wrapping `Expr` with a custom `Hash`). The unified DFS
//   pass: (1) at scope boundaries push new ScopeId; (2) on first encounter of
//   E in scope S → record `(E → (S, sym_id))`; (3) on second encounter in
//   ANY scope → if scope S is ancestor of owning scope, use parent's sym_id;
//   if S is sibling/descendant, the new sym was first-constructed in S' —
//   count the use but don't re-extract. The extraction fires in the scope that
//   OWNS the sym (S'). See G2.2-HASH-CONS-PRIMITIVE-HANDOFF.md.
//
//   Preserved through migration: `replace_all` (S7, walker-completeness),
//   `contains_val_use` (S4, walker-completeness), `direct_children` (used by
//   both), renumbering pipeline (`dfs_reassign_val_ids → reorder_valdefs →
//   sequential_renumber` confirmed correct at S7). Hash-cons does NOT replace
//   these — it replaces the extraction-scope decision only.
fn process_ast_graph_impl(
    expr: Expr,
    global_max_id: u32,
    dag_usages: Vec<(Expr, usize)>,
    schedule: Vec<Expr>,
    mode: ScopeMode,
) -> Expr {
    // ValIds defined only inside deeper-If arms within `expr`. A candidate
    // whose RHS references any of these cannot be hoisted to this scope —
    // doing so creates a forward `ValUse(id)` referencing a ValDef that
    // isn't visible at this scope. (Same class as S32 Bug B.)
    let branch_local_ids = collect_branch_local_val_ids(&expr);
    let mut env: Vec<(Expr, u32)> = Vec::new();
    let mut next_id = find_max_val_id(&expr).max(global_max_id) + 1;

    let trace = std::env::var("CSE_TRACE_EXTRACT").is_ok();
    for node in &schedule {
        if !is_extractable(node) {
            continue;
        }
        if !is_graph_shared(node) {
            if trace {
                eprintln!("[PAG/{:?}] skip-not-shared :: {}", mode, short_expr(node));
            }
            continue;
        }
        let dag_count = dag_usages
            .iter()
            .find(|(e, _)| e == node)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        // sig-15 sigmao S27 (Cohort A3 / Session 6 of QB-HANDOFF-15-OF-15,
        // 2026-05-12, FALSIFIED at Probe 2 with 27th falsification fingerprint
        // instance — SECOND CONSECUTIVE compensating-extraction class after
        // S26 / Cohort A1 at HEAD `a1c9b8c0`). Probed shape-narrow override
        //     if matches!(node, Expr::CreateProveDlog(cpd)
        //         if matches!(&*cpd.input, Expr::DecodePoint(_)))
        //         && dag_count < 2
        //     { dag_count = count_occurrences(&expr, node); }
        // to mirror Scala's `globalUsagesOf` semantics for the chain shape.
        // Cross-fixture pre-flight CLEAN (12 MATCH sig-15 + F.2 563/575 + lib
        // 251/251 + Lilium SaleLP MATCH + ecosystem 11/14 all preserved; the
        // shape predicate is structurally inert for all non-sigmao fixtures).
        // SHAPE diff IMPROVES (common-multiset 38→40, L-only 7→5, N-only 8→6)
        // — LOCAL emits `PDlog[DecPt[ByIdx]]` (matches NODE d9) + `SPBytes[VU]`
        // (matches NODE d13) as separate outer ValDefs, exactly the structural
        // split Scala's bottom-up sym construction produces.
        // BUT sigmao BYTES REGRESS Δ -24 → -26 (LOCAL 1124 → 1122). Root cause
        // per `feedback_falsification_fingerprint` rule #1: LOCAL's pre-fix
        // shape `SPBytes[PDlog[DecPt[VU]]]` (one nested ValDef + one inline at
        // line 239 inside SigmaAnd thunk) was a compensating-extraction —
        // larger nested inline at the &&-chain held bytes UP toward NODE size.
        // Splitting into two ValDefs saves ~2B inline (smaller ValUse refs at
        // both sites) but the SAVED inline bytes were exactly what was
        // OFFSETTING the gap. Same byte-direction failure as S26 / Cohort A1
        // `ExtractScriptBytes(SELF)` suppression. Reverted. Closure of sigmao
        // -24 plateau requires the A3 chain split COUPLED with a partner fix
        // that ADDS LOCAL bytes (one of: A1 ExScript[Self] suppression coupled
        // with B Cohort, OR Cohort B extractions s2/s3/s6/s9 standalone, OR
        // architectural WS-G per-thunk-distinct sym construction). Per
        // fingerprint rule #1: "if SHAPE-correct fix BYTE-regresses, the
        // existing extraction is empirically correct for current LOCAL state
        // — the fix can only land coupled with the partner extraction that
        // closes the offsetting gap." F1-F7 axes all green during the probe.
        //
        // Sigmao S28 (2026-05-13, Cohort A1+A3 COUPLED / Session 7 of
        // QB-HANDOFF-15-OF-15). Re-applied this A3 override simultaneously
        // with S26's A1 narrow `is_graph_shared` suppression. Result: sigmao
        // LOCAL 1124 → 1118 (Δ -24 → -30) — EXACT simple sum (no cascade).
        // SHAPE diff dramatic improvement (common 38→42, L-only 7→2, N-only
        // 8→4) but byte-direction wrong. Cross-fixture clean (12 MATCH +
        // F.2 563/575 + plateaus held + ecosystem 11/14). Empirically closes
        // the single-and-coupled Scala-faithful structural fix class for
        // sigmao; Session 8 pivots to Cohort B byte-ADDING attacks. 28th
        // falsification fingerprint instance. Reverted; doc-comment durable.
        if dag_count >= 2 {
            // Reject candidates whose RHS references a ValId defined only
            // inside a deeper-If arm — hoisting them here would create a
            // forward ValUse to a ValDef that isn't visible at this scope.
            if references_locally_defined(node, &branch_local_ids) {
                if trace {
                    eprintln!(
                        "[PAG/{:?}] reject reason=branch_local_ids dag_count={} :: {}",
                        mode,
                        dag_count,
                        short_expr(node)
                    );
                }
                continue;
            }

            // Scope checks differ by mode:
            //
            // - Root: in Scala, && / || wrap the right operand in a
            //   ThunkDef and `If` true/false branches are also ThunkDefs.
            //   Expressions that ONLY appear inside such sub-ThunkDefs
            //   create per-thunk graph symbols rather than hash-consing
            //   into the outer program graph, so they should not be
            //   extracted at root. Reject when not present in the main
            //   scope. Exception: `GlobalVars`-input ExtractId / ExtractAmount
            //   can be present in &&/|| right arms (Scala still counts
            //   those references) — use the more permissive
            //   `appears_outside_if_branches` for them.
            //
            // - Branch: rejection by branch_local_ids above already
            //   prevents the analogue (forward references into deeper
            //   sub-thunks). Don't apply the main-scope check here — the
            //   whole point of branch mode is to extract at this branch
            //   scope.
            if mode == ScopeMode::Root {
                // WS-F cluster 010 B3: `proveDlog(groupGenerator)` — i.e.
                // `CreateProveDlog` whose only context touch is the singleton
                // `Global` receiver — is hash-consed in Scala's graph IR
                // exactly like ExtractId(SELF). `mainG.hasManyUsagesGlobal`
                // counts its usages across &&/|| ThunkDefs, so when both
                // occurrences sit inside an `||` right-arm thunk Scala still
                // extracts the ValDef at the surrounding (root) scope. Without
                // this carve-out, `appears_in_main_scope` (strict) rejects the
                // candidate and Rust inlines twice, diverging from Scala.
                // Empirical fixtures: composition_143, composition_189.
                let is_global_only_provedlog = matches!(node, Expr::CreateProveDlog(_))
                    && !touches_runtime_context(node);
                // sig-15 sigmao_option (`168acaf2`) and paideia_stake_state S3 (`d6ba8d82`):
                // SelectField is hash-consed by Scala's TreeBuilding on (input_sym, field_index)
                // — the SelectField sym depends only on the input sym. When the input is
                // shareable at root, Scala's `mainG.hasManyUsagesGlobal` counts SelectField uses
                // across &&/|| ThunkDefs and emits the ValDef at root. The strict
                // `appears_in_main_scope` rejects whenever all uses are inside any deferred
                // thunk (And/Or right arm OR If branch), so it's too aggressive for SelectField.
                //
                // Two input-stability classes need different scope-check semantics:
                //
                // (A) `Expr::ValUse(_)` input — the ValUse is already root-bound (upstream
                //     `references_locally_defined` upstream rejects branch-local ValUses, so
                //     reaching this point implies root-binding). The SelectField sym only
                //     depends on this root-bound ValUse; Scala's `hasManyUsagesGlobal` will
                //     hoist regardless of where the uses physically sit, including when ALL
                //     uses are inside If-arm thunks. Bypass the scope check entirely (S1
                //     `168acaf2` invariant). Sigmao_option's
                //     `val tup = box.tokens.getOrElse(...)` + many `tup._N` references inside
                //     `validMintOption || validDeliverOption || validExerciseOption` If arms
                //     fits this case.
                //
                // (B) Dag-shared non-ValUse input (eg `ByIndex(GlobalVars, Const)` itself shared
                //     across siblings) — the input WILL be extracted in this same pass, so the
                //     resolved SelectField RHS becomes `SelectField(ValUse(...), n)`, but until
                //     resolution the candidate sym still references the unresolved input chain.
                //     Scala's segregation places it at the LCA of its sibling occurrences. Use
                //     the LCA-aware permissive check (S3 paideia: `appears_outside_if_branches`
                //     OR `count_distinct_top_level_containers >= 2`).
                //
                // Splitting (A) vs (B) is required: sigmao's `tup._N` uses are all inside If
                // arms (no use is "outside" any If, and the SelectField appears only inside the
                // disjunction's body, not in 2+ distinct top-level container positions of the
                // outer block) — neither LCA predicate fires, so a unified `is_select_field_on_stable`
                // gate would (and did, post-`d6ba8d82`) reject sigmao's hoist.
                let is_select_field_on_val_use = matches!(node,
                    Expr::SelectField(s) if matches!(&*s.expr.input, Expr::ValUse(_))
                );
                let is_select_field_on_dag_shared = matches!(node,
                    Expr::SelectField(s)
                        if !matches!(&*s.expr.input, Expr::ValUse(_))
                        && dag_usages.iter().any(|(e, c)| *c >= 2 && e == &*s.expr.input)
                );
                let use_if_branch_check = matches!(node, Expr::ExtractId(ei) if matches!(&*ei.input, Expr::GlobalVars(_)))
                    || matches!(node, Expr::ExtractAmount(ea) if matches!(&*ea.input, Expr::GlobalVars(_) | Expr::ByIndex(_)))
                    || is_global_only_provedlog;
                // Pure-constant Upcast wrappers must be scope-checked even though
                // they don't touch context: Scala's ThunkScope.findDef chain
                // creates a separate sym per Thunk for these, so a wrapper used
                // only inside If branches must extract at branch scope (where the
                // ValDef RHS contributes 1 pool entry per branch), not at root.
                // Without this check, root mode would hoist (1 ValDef + N ValUses)
                // and segregation would emit fewer pool entries than Scala. (S45)
                let is_pure_const_upcast = matches!(node,
                    Expr::Upcast(uc)
                        if matches!(&*uc.input, Expr::Const(_) | Expr::ConstPlaceholder(_))
                );
                // S55: bare Const literals (post-S54: SBigInt) need the same
                // ThunkScope-aware extraction as pure-const Upcast wrappers.
                // Scala creates a per-Thunk sym for the Const node, so a Const
                // used only inside If branches must extract at the branch
                // scope, not at root. Without this check, OpenOrderERG hoisted
                // BigInt(0) to outer Lambda items[] — shifting its constant
                // pool position from index 10 (node) to index 7 (local).
                let is_bare_const = matches!(node, Expr::Const(_));
                // Pure-constant candidates (no transitive ValUse / GlobalVars /
                // Context / SelfBox / GetVar) can always be hoisted to root —
                // their value doesn't depend on context, so eager evaluation
                // at root is semantically equivalent to inline evaluation.
                // Only context-touching and pure-const-Upcast/bare-Const
                // candidates need the scope check.
                //
                // S30 paideia +2 surgical-probe falsification (30th instance,
                // diag-only commit): adding `is_pure_const_tuple` predicate
                // here to suppress the Tup `[Coll[Byte](), 0L)]` root ValDef
                // at dag_count=3 regresses paideia 1470→1477B (Δ +2 → +9)
                // because `apply_cse_within_branches` then extracts the Tup
                // independently per validStakeTx/validUnstakeTx sibling thunk
                // (each sees dag_count≥2 locally), yielding 2 sibling ValDefs
                // whose aggregate header overhead exceeds the saved root-inline
                // body bytes. Compensating-extraction class (S7/S8 1477 / 1483
                // exact replay at this HEAD; cascade-stable across S7→S30).
                // Architectural mechanism = Scala's first-DFS-construction-scope
                // ownership (per Probe 0 metals on TreeBuilding.processAstGraph
                // + GraphBuilding hash-cons), NOT a single-predicate gate.
                // DO NOT re-attempt; closure requires WS-G hash-cons migration.
                let needs_check = (use_if_branch_check
                    || touches_context(node)
                    || is_pure_const_upcast
                    || is_bare_const
                    || is_select_field_on_dag_shared)
                    && !is_select_field_on_val_use;
                if needs_check {
                    let in_scope = if is_select_field_on_dag_shared {
                        // SelectField on dag-shared non-ValUse input: LCA-aware check.
                        // Hoist when either (a) at least one use is literally outside
                        // an If branch, or (b) the candidate appears in 2+ distinct
                        // top-level sibling containers (paideia's `validStakeTx ||
                        // validEmitTx || validUnstakeTx` arms, each containing uses
                        // inside their own If cascades). Both cases force the LCA to
                        // the outer scope. (S3 paideia `d6ba8d82`)
                        appears_outside_if_branches(&expr, node)
                            || count_distinct_top_level_containers(&expr, node) >= 2
                    } else if use_if_branch_check {
                        appears_outside_if_branches(&expr, node)
                    } else {
                        appears_in_main_scope(&expr, node)
                    };
                    if !in_scope {
                        if trace {
                            eprintln!(
                                "[PAG/Root] reject reason=not-in-main-scope dag_count={} :: {}",
                                dag_count,
                                short_expr(node)
                            );
                        }
                        continue;
                    }
                }
            }

            if trace {
                eprintln!(
                    "[PAG/{:?}] extract id={} dag_count={} :: {}",
                    mode,
                    next_id,
                    dag_count,
                    short_expr(node)
                );
            }
            env.push((node.clone(), next_id));
            next_id += 1;
        } else if trace {
            eprintln!(
                "[PAG/{:?}] skip dag_count={} :: {}",
                mode,
                dag_count,
                short_expr(node)
            );
        }
    }

    if env.is_empty() {
        return expr;
    }

    // Apply extractions iteratively with candidate updating.
    // After extracting node X, update remaining env entries that contained
    // X as a sub-expression. This handles chained extraction:
    // e.g., extract INPUTS(0), then PropertyCall(ValUse, tokens) still matches.
    let mut result = expr;
    let mut final_env: Vec<(Expr, u32)> = Vec::new();
    let mut env = env;
    let mut i = 0;
    while i < env.len() {
        let (ref node, val_id) = env[i];
        // Re-check: does this node still appear 2+ times in the current tree?
        let tree_count = count_occurrences(&result, node);
        if tree_count < 2 {
            i += 1;
            continue;
        }
        let node = node.clone();
        let val_use = Expr::ValUse(ValUse {
            val_id: ValId(val_id),
            tpe: expr_type(&node),
        });
        result = replace_all(&result, &node, &val_use);
        // Update remaining env entries to reflect this extraction
        for entry in env.iter_mut().skip(i + 1) {
            let (ref mut env_expr, _) = entry;
            *env_expr = replace_all(env_expr, &node, &val_use);
        }
        final_env.push((node, val_id));
        i += 1;
    }
    let env = final_env;

    if env.is_empty() {
        return result;
    }

    // Step 5: Build ValDef RHS expressions by resolving children through env
    let val_defs: Vec<Expr> = env
        .iter()
        .map(|(node, val_id)| {
            let rhs = build_value_recurse(node, &env);
            Expr::ValDef(Spanned {
                source_span: SourceSpan::empty(),
                expr: ValDef {
                    id: ValId(*val_id),
                    rhs: rhs.into(),
                },
            })
        })
        .collect();

    // Lambda-rescue post-pass: if a sub-expression of one of these new
    // ValDef RHSes also appears in a lambda body in `result`, extract it
    // as a separate ValDef. Mirrors Scala's `mainG.hasManyUsagesGlobal`
    // behaviour through ThunkDef boundaries (sig-15 rosen_event_trigger).
    let post_next_id = val_defs
        .iter()
        .filter_map(|vd| {
            if let Expr::ValDef(s) = vd {
                Some(s.expr.id.0)
            } else {
                None
            }
        })
        .max()
        .map(|m| m + 1)
        .unwrap_or(next_id);
    let (val_defs, result, _) = lambda_rescue_post_pass(val_defs, result, post_next_id);

    // Wrap in BlockValue. New ValDefs are prepended to existing items, but
    // their RHSs may ValUse existing item ids and existing items may ValUse
    // newly-extracted ids. Topo-sort to ensure every ValUse comes after its
    // ValDef (same class as S32 Bug A).
    match result {
        Expr::BlockValue(spanned) => {
            let mut items = val_defs;
            items.extend(spanned.expr.items);
            let items = topo_order_valdefs(items);
            Expr::BlockValue(Spanned {
                source_span: SourceSpan::empty(),
                expr: BlockValue {
                    items,
                    result: spanned.expr.result,
                },
            })
        }
        other => Expr::BlockValue(Spanned {
            source_span: SourceSpan::empty(),
            expr: BlockValue {
                items: topo_order_valdefs(val_defs),
                result: other.into(),
            },
        }),
    }
}

/// WS-G.2.3 — runtime gate for the parallel hash-cons driver.
/// Default OFF; the existing `process_ast_graph_impl` path is preserved
/// for every fixture currently at MATCH. G.2.4 toggles per-fixture for
/// cross-fixture comparison.
fn hash_cons_enabled() -> bool {
    std::env::var("CSE_HASH_CONS").as_deref() == Ok("1")
}

/// WS-G.2.3 — parallel hash-cons driver. Mirrors the SHAPE of
/// `process_ast_graph_impl` (schedule + dag_count threshold → env →
/// replace_all → build_value_recurse → BlockValue wrap) but replaces the
/// scope-gate hierarchy (`appears_in_main_scope` /
/// `count_distinct_top_level_containers` / GlobalVars carve-outs) with a
/// single hash-cons ownership check derived from a structural-equality
/// SymTable walk.
///
/// Algorithm (per G.2.1 + G.2.3 handoff):
///   1. Pre-order DFS of `expr` interning every sub-expression into a
///      SymTable. Each If true/false branch, each &&/|| right operand, and
///      each FuncValue body opens a new scope (parent = current). This
///      mirrors Scala's `ThunkScope.findDef → findGlobalDefinition` chain
///      and produces sibling-scope independence (Thunks.scala parity).
///   2. For each candidate node in `schedule` with `dag_count >= 2` and
///      `!references_locally_defined`, gate extraction by:
///         `sym_table.find(node, ROOT).is_some()`
///      — i.e., the sub-expression was first-constructed at root scope (or
///      an ancestor of root, which is just root itself). Sub-expressions
///      whose first construction was inside a child ThunkDef scope live
///      there and are NOT hoisted here.
///   3. Identical replace_all / build_value_recurse / topo-sort tail as
///      `process_ast_graph_impl`.
///
/// Pre-stated falsification (per handoff): hash-cons is necessary but
/// probably NOT sufficient. The four known unknowns (perf, walker
/// completeness, DFS-construction-order vs LCA-of-uses, downstream
/// renumbering interface) may surface as blockers. Diag-only commit
/// posture: this driver lands as a flag-gated parallel path; default
/// behaviour unchanged.
///
/// Instrumentation: `CSE_TRACE_HASH_CONS=1` logs every `find_or_intern`,
/// every new_scope, and every extract / skip decision.
fn process_ast_graph_hash_cons(
    expr: Expr,
    global_max_id: u32,
    dag_usages: Vec<(Expr, usize)>,
    schedule: Vec<Expr>,
    mode: ScopeMode,
) -> Expr {
    use sym_table::{ScopeId, SymTable};

    let trace = std::env::var("CSE_TRACE_HASH_CONS").is_ok();
    let mut st = SymTable::new();
    let root: ScopeId = 0;

    fn is_logical_and_or(kind: &ergotree_ir::mir::bin_op::BinOpKind) -> bool {
        matches!(
            kind,
            ergotree_ir::mir::bin_op::BinOpKind::Logical(
                ergotree_ir::mir::bin_op::LogicalOp::And
                    | ergotree_ir::mir::bin_op::LogicalOp::Or
            )
        )
    }

    fn intern_walk(e: &Expr, st: &mut SymTable, scope: sym_table::ScopeId, trace: bool) {
        let (sym, is_new) = st.find_or_intern(e, scope);
        if trace {
            eprintln!(
                "[HC/intern] scope={} sym={} new={} :: {}",
                scope,
                sym,
                is_new,
                short_expr(e)
            );
        }
        match e {
            Expr::If(if_op) => {
                intern_walk(&if_op.condition, st, scope, trace);
                let s_t = st.new_scope(scope);
                if trace {
                    eprintln!("[HC/new_scope] parent={} child={} (If.true)", scope, s_t);
                }
                intern_walk(&if_op.true_branch, st, s_t, trace);
                let s_f = st.new_scope(scope);
                if trace {
                    eprintln!("[HC/new_scope] parent={} child={} (If.false)", scope, s_f);
                }
                intern_walk(&if_op.false_branch, st, s_f, trace);
            }
            Expr::BinOp(s) if is_logical_and_or(&s.expr.kind) => {
                intern_walk(&s.expr.left, st, scope, trace);
                let s_r = st.new_scope(scope);
                if trace {
                    eprintln!(
                        "[HC/new_scope] parent={} child={} (&&/|| right)",
                        scope, s_r
                    );
                }
                intern_walk(&s.expr.right, st, s_r, trace);
            }
            Expr::FuncValue(fv) => {
                let s_b = st.new_scope(scope);
                if trace {
                    eprintln!("[HC/new_scope] parent={} child={} (FuncValue body)", scope, s_b);
                }
                intern_walk(fv.body(), st, s_b, trace);
            }
            _ => {
                for c in direct_children(e) {
                    intern_walk(c, st, scope, trace);
                }
            }
        }
    }

    // Canonicalize the intern-walk input the same way ExprKey expects
    // (consumers MUST strip source spans). `apply_cse` already stripped
    // spans at pipeline entry; this is defensive in case the driver is
    // ever called from a different path.
    let expr_for_walk = strip_source_spans(expr.clone());
    intern_walk(&expr_for_walk, &mut st, root, trace);

    let branch_local_ids = collect_branch_local_val_ids(&expr);
    let mut env: Vec<(Expr, u32)> = Vec::new();
    let mut next_id = find_max_val_id(&expr).max(global_max_id) + 1;

    for node in &schedule {
        if !is_extractable(node) {
            continue;
        }
        if !is_graph_shared(node) {
            if trace {
                eprintln!(
                    "[HC/{:?}] skip-not-shared :: {}",
                    mode,
                    short_expr(node)
                );
            }
            continue;
        }
        let dag_count = dag_usages
            .iter()
            .find(|(e, _)| e == node)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        if dag_count < 2 {
            if trace {
                eprintln!(
                    "[HC/{:?}] skip dag_count={} :: {}",
                    mode,
                    dag_count,
                    short_expr(node)
                );
            }
            continue;
        }
        if references_locally_defined(node, &branch_local_ids) {
            if trace {
                eprintln!(
                    "[HC/{:?}] reject reason=branch_local_ids dag_count={} :: {}",
                    mode,
                    dag_count,
                    short_expr(node)
                );
            }
            continue;
        }

        // Hash-cons scope gate (replaces appears_in_main_scope hierarchy):
        // node is eligible for extraction at this scope iff some sym for
        // it lives at root (the current driver scope is `root` per our
        // intern walk; sub-thunks created during the walk are unreachable
        // from `st.find(_, root)`).
        //
        // S18 duckpools D-B reconciliation: strict `owned_at_root` rejects
        // candidates whose occurrences all sit inside `&&`/`||` right-arm
        // sub-thunks even when the LEFT arm of an inner `&&` (which Scala
        // counts as "eager / main-scope-for-this-Thunk") contains one of
        // the uses. HC=0's `appears_in_main_scope` walker resets the
        // `in_and_thunk` flag on every recursive left-arm descent, so it
        // returns true whenever any path reaches the target through a
        // left-arm-only suffix. The strict scope-id walker in
        // `intern_walk` instead inherits the parent scope on left-arm
        // descent — same scope-id passes through, so the sym is stamped
        // with the OUTER (sub-thunk) scope rather than root.
        //
        // Surgical fallback (mode == Root only): when `owned_at_root` is
        // false but `appears_in_main_scope` returns true, the candidate is
        // semantically root-eligible per HC=0 semantics. Admit. Branch mode
        // keeps the strict ownership check unchanged.
        //
        // Empirical signature: duckpools_child_interest `SizeOf(VU(13))`
        // dag_count=3 — all 3 occurrences inside `&&` right-arm scopes
        // 5/6/7, but at least one sits in an inner left-arm reachable via
        // a left-reset path. HC=0 extracts at id=53; HC=1 (pre-fallback)
        // rejected → −2B regression vs NODE.
        let owned_at_root = st.find(node, root).is_some();
        let main_scope_ok = !owned_at_root
            && mode == ScopeMode::Root
            && appears_in_main_scope(&expr, node);
        // S19 D-A paideia reconciliation — pure-const Root fallback.
        //
        // When HC=1 Root strict-rejects a pure-const aggregate (Tuple /
        // Collection over Const | ConstPlaceholder leaves) via the
        // `owned_at_root` ownership gate, branches see the aggregate
        // locally and each extracts an independent ValDef — yielding
        // sibling per-branch duplicates whose aggregate header overhead
        // drives paideia_stake_state's HC=1 +27 regression. Probe 1
        // trace-diff (CSE_TRACE_HASH_CONS=1) at HEAD `07b12b1b` showed
        // `Tuple([Collection(SByte:empty), Const(0:SLong)])` extracted
        // at HC=1 Branch in BOTH validStakeTx + validUnstakeTx arms.
        //
        // Pure-const aggregates are semantically root-safe (no context
        // / ValUse / GlobalVars dependencies — touches_runtime_context
        // returns false), so admitting at Root is sound. Empirical
        // outcome (paideia HC=1 1495 → 1488, -7B). Cross-fixture clean
        // (12 HC=1 MATCH preserved; sigmao HC=1 -28 preserved; gluon
        // HC=1 +62 preserved; HC=0 sacred untouched).
        //
        // Falsified alternative (S19 Probe A, 31st falsification):
        // rejecting pure-const at HC=1 Branch caused paideia 1495 →
        // 1498 (+3B regression) — compensating-extraction class (S30
        // root-rejection replay at HC=1 Branch surface). Per-branch
        // pure-const ValDef is byte-efficient locally; inlining
        // duplicates costs more bytes than the header. Direction is
        // ADMIT-AT-ROOT, not REJECT-AT-BRANCH.
        let pure_const_root_ok =
            !owned_at_root && mode == ScopeMode::Root && is_pure_const_shape(node);
        // S22 D-C paideia HC=1 — `!appears_in_main_scope` guard on
        // `pure_const_root_ok` FALSIFIED (Path A per QB-HANDOFF-15-OF-15 /
        // S21 reframe). Probe 2 at HEAD `0db5b483`:
        // paideia HC=1 1470 → **1477B** (Δ +2 → +9, **+7B REGRESSION**) —
        // EXACT BYTE-ARITHMETIC REPLAY of S30's HC=0 surface
        // (`is_pure_const_tuple` predicate added to `needs_check`,
        // `029a9376` 2026-05-13). Two distinct code paths (HC=0
        // `process_ast_graph_impl::needs_check` vs HC=1
        // `process_ast_graph_hash_cons::pure_const_root_ok`); same
        // downstream branch-CSE driver re-extracts `Tup[Coll[Byte](),0L]`
        // independently in validStakeTx + validUnstakeTx sibling thunks
        // (each sees local `dag_count >= 2` post-suppression), yielding
        // 2 sibling per-branch ValDefs whose aggregate header overhead
        // (~7B) exceeds the saved root-inline body bytes.
        //
        // Cross-fixture pre-flight (Probe 1 at HEAD): 0 admissions of
        // `pure-const-root-fallback` on the 12 currently-MATCH HC=1
        // fixtures + sigmao + gluon — guard is shape-orthogonal
        // cross-fixture, but the paideia-narrow regression alone forces
        // revert. Class label: cross-mode compensating-extraction
        // confirmation (6th cumulative compensating-extraction instance,
        // 34th falsification fingerprint).
        //
        // Reframe for Session 23: per-thunk-scope `count_dag_usages`
        // (Path B, ~80-120 LOC architectural rewrite) is the only
        // remaining surgical surface for paideia +2 closure. The S30+S22
        // pair structurally proves that local gate-tuning at the
        // pure_const-admit-decision layer cannot net-zero without
        // coupled per-thunk branch-CSE count semantics.
        // S20 D-A paideia continuation — SelectField-on-dag-shared LCA Root
        // fallback.
        //
        // Probe 1 (Session 20, HEAD `6976d446`) trace-diff revealed that
        // HC=0's 18 paideia Root extracts vs HC=1's 14 (S19 post-pure-const
        // fallback) differ at exactly 4 ids: HC=0 ids 79/80/81/83 are all
        // `SelectField(ByIndex(...))` shapes (dag_count 4/4/2/3) admitted
        // by HC=0's `is_select_field_on_dag_shared` LCA-aware predicate
        // (`appears_outside_if_branches || count_distinct_top_level_containers >= 2`).
        // Strict `appears_in_main_scope` rejects them at HC=1 Root because
        // all uses sit inside `validStakeTx || validEmitTx || validUnstakeTx`
        // arms; the LCA-permissive check admits them because they appear
        // in 2+ distinct top-level container positions.
        //
        // Mirror HC=0's gate at HC=1 Root only, restricted to the dag-shared
        // non-ValUse SelectField class. The companion `is_select_field_on_val_use`
        // case is the sigmao HC=1 -28 driver (val_ids 5/7 SelectField on
        // VU(STuple([SColl(SByte), SColl(SByte)]))) — its admission is
        // already handled by Scala-mirrored branch-extraction and must NOT
        // be touched by this fallback.
        //
        // Cross-fixture pre-flight (Probe 1): sigmao's SelectField root
        // candidates have ValUse input (not dag-shared non-VU), so this
        // predicate is shape-orthogonal to sigmao's -28 dependency.
        let is_select_field_on_dag_shared_root = !owned_at_root
            && mode == ScopeMode::Root
            && matches!(node, Expr::SelectField(s)
                if !matches!(&*s.expr.input, Expr::ValUse(_))
                && dag_usages.iter().any(|(e, c)| *c >= 2 && e == &*s.expr.input))
            && (appears_outside_if_branches(&expr, node)
                || count_distinct_top_level_containers(&expr, node) >= 2);
        if !owned_at_root && !main_scope_ok && !pure_const_root_ok && !is_select_field_on_dag_shared_root {
            if trace {
                eprintln!(
                    "[HC/{:?}] reject reason=not-owned-at-root dag_count={} :: {}",
                    mode,
                    dag_count,
                    short_expr(node)
                );
            }
            continue;
        }
        if trace && !owned_at_root && main_scope_ok {
            eprintln!(
                "[HC/{:?}] admit reason=main-scope-fallback dag_count={} :: {}",
                mode,
                dag_count,
                short_expr(node)
            );
        }
        if trace && !owned_at_root && !main_scope_ok && pure_const_root_ok {
            eprintln!(
                "[HC/{:?}] admit reason=pure-const-root-fallback dag_count={} :: {}",
                mode,
                dag_count,
                short_expr(node)
            );
        }
        if trace
            && !owned_at_root
            && !main_scope_ok
            && !pure_const_root_ok
            && is_select_field_on_dag_shared_root
        {
            eprintln!(
                "[HC/{:?}] admit reason=select-field-on-dag-shared-root-fallback dag_count={} :: {}",
                mode,
                dag_count,
                short_expr(node)
            );
        }

        if trace {
            eprintln!(
                "[HC/{:?}] extract id={} dag_count={} :: {}",
                mode,
                next_id,
                dag_count,
                short_expr(node)
            );
        }
        env.push((node.clone(), next_id));
        next_id += 1;
    }

    if env.is_empty() {
        return expr;
    }

    // Replacement tail — structurally identical to process_ast_graph_impl
    // (lines 6486–6580). Preserved verbatim so byte-divergence under
    // CSE_HASH_CONS=1 is attributable to the extraction-gate change, not
    // a different rewrite shape.
    let mut result = expr;
    let mut final_env: Vec<(Expr, u32)> = Vec::new();
    let mut env = env;
    let mut i = 0;
    while i < env.len() {
        let (ref node, val_id) = env[i];
        let tree_count = count_occurrences(&result, node);
        if tree_count < 2 {
            i += 1;
            continue;
        }
        let node = node.clone();
        let val_use = Expr::ValUse(ValUse {
            val_id: ValId(val_id),
            tpe: expr_type(&node),
        });
        result = replace_all(&result, &node, &val_use);
        for entry in env.iter_mut().skip(i + 1) {
            let (ref mut env_expr, _) = entry;
            *env_expr = replace_all(env_expr, &node, &val_use);
        }
        final_env.push((node, val_id));
        i += 1;
    }
    let env = final_env;

    if env.is_empty() {
        return result;
    }

    let val_defs: Vec<Expr> = env
        .iter()
        .map(|(node, val_id)| {
            let rhs = build_value_recurse(node, &env);
            Expr::ValDef(Spanned {
                source_span: SourceSpan::empty(),
                expr: ValDef {
                    id: ValId(*val_id),
                    rhs: rhs.into(),
                },
            })
        })
        .collect();

    let post_next_id = val_defs
        .iter()
        .filter_map(|vd| {
            if let Expr::ValDef(s) = vd {
                Some(s.expr.id.0)
            } else {
                None
            }
        })
        .max()
        .map(|m| m + 1)
        .unwrap_or(next_id);
    let (val_defs, result, _) = lambda_rescue_post_pass(val_defs, result, post_next_id);

    match result {
        Expr::BlockValue(spanned) => {
            let mut items = val_defs;
            items.extend(spanned.expr.items);
            let items = topo_order_valdefs(items);
            Expr::BlockValue(Spanned {
                source_span: SourceSpan::empty(),
                expr: BlockValue {
                    items,
                    result: spanned.expr.result,
                },
            })
        }
        other => Expr::BlockValue(Spanned {
            source_span: SourceSpan::empty(),
            expr: BlockValue {
                items: topo_order_valdefs(val_defs),
                result: other.into(),
            },
        }),
    }
}

/// WS-G QB1 Phase 3.1 — Session 23 — runtime gate for the construction-time
/// hash-cons driver (`process_ast_graph_hash_cons_v2`). Default OFF.
///
/// QB0 (S17–S22) plateau-confirmation: D-C count-arithmetic surface is
/// empirically exhausted (S30 + S22 cross-mode +7B byte-arithmetic exact
/// replay). The `_v2` driver replaces the top-down `intern_walk` +
/// schedule-walk + per-fallback gate decisions of v1 with bottom-up
/// per-thunk construction-time hash-cons, mirroring Scala's
/// `findOrCreateDefinition` + `Thunks::ThunkScope` semantics.
fn hash_cons_v2_enabled() -> bool {
    std::env::var("CSE_HC_V2").as_deref() == Ok("1")
}

/// WS-G QB1 Phase 3.1 — bottom-up construction-time hash-cons driver.
/// Parallel-path implementation behind `CSE_HC_V2=1`. v1
/// (`process_ast_graph_hash_cons`) and `process_ast_graph_impl` are
/// preserved untouched — F1 (HC=0 default sig-15 ≥ 12/15) is sacred until
/// the final flip.
///
/// Algorithm (per QB1 §2 + §3.1):
///   1. Bottom-up DFS of `expr`: visit children before parents, with a
///      `thunk_stack` push at every Scala-equivalent thunk boundary
///      (If true/false branch, &&/|| right arm, FuncValue body).
///   2. For each sub-expression `e` (post-order): `find_or_intern(e,
///      current_scope)` against the existing `SymTable` primitive
///      (`find` walks current → parent → root, never visiting sibling
///      scopes — produces per-thunk-distinct syms by construction).
///      - First encounter: fresh sym in current scope's bodyDefs; record
///        (sym → expr) in `sym_expr` for emission, push onto
///        `sym_order`, count = 1.
///      - Subsequent encounter in same or descendant scope: returns the
///        existing sym; increment `sym_counts[sym]`.
///   3. After full traversal, walk `sym_order` (post-order ⇒ children
///      before parents — same topo invariant `dfs_schedule` provides
///      v1's schedule). For each sym with `count > 1` AND `is_extractable`
///      AND `is_graph_shared` AND `!references_locally_defined`:
///        push `(expr, val_id)` into env.
///   4. Identical replace_all / build_value_recurse / topo-sort tail as
///      v1 (verbatim).
///
/// Differences from v1 (`process_ast_graph_hash_cons`):
///   - v1 does a top-down intern walk producing one global `SymTable`,
///     then walks an externally-computed `schedule` and uses gate
///     predicates (`owned_at_root` + `appears_in_main_scope` fallback +
///     `is_pure_const_shape` fallback + `is_select_field_on_dag_shared`
///     fallback) to decide extraction. The construction is correct but
///     the schedule + count come from the global `dag_usages` (parent-
///     edge walk) — same as `process_ast_graph_impl`. v1 IS gate-tuning
///     over the existing tree-position CSE architecture.
///   - v2 derives counts FROM the bottom-up construction itself:
///     `sym_counts[sym]` is the number of times `find_or_intern` resolved
///     to `sym` from any scope in its visibility chain. Sibling scopes
///     can't see each other's syms (the SymTable enforces this in
///     `find`), so a pure-const aggregate appearing in two sibling
///     thunks intern as two distinct syms (each count 1) rather than
///     one sym (count 2). This is the construction-time per-thunk-
///     distinct sym semantics the QB1 handoff calls for.
///
/// Stop condition for Session 23 (per QB1 §3.1): v2 compiles and runs on
/// at least 1 fixture without panic. Diag-only OK. No correctness
/// target yet. Sessions 24–25 iterate on MATCH parity + plateau closure.
///
/// Instrumentation: `CSE_TRACE_HC_V2=1` logs every find_or_intern visit
/// (scope, sym, new/found, post-count) and every extract / skip
/// decision. `CSE_TRACE_SCOPE_CHAIN` from SymTable still works.
///
/// Pre-stated falsification: the construction-time semantics may produce
/// fewer extractions than Scala if our scope-push set is incomplete (e.g.
/// Scala wraps additional bodies in Thunks that we don't), or more
/// extractions if `find_or_intern` under-collapses (unlikely — SymTable
/// has 8 passing unit tests). v2 emitting on a fixture without panic
/// satisfies the S23 stop condition regardless of byte parity.
#[allow(clippy::too_many_arguments)]
fn process_ast_graph_hash_cons_v2(
    expr: Expr,
    global_max_id: u32,
    _dag_usages: Vec<(Expr, usize)>,
    _schedule: Vec<Expr>,
    mode: ScopeMode,
) -> Expr {
    use sym_table::{ScopeId, SymId, SymTable};

    let trace = std::env::var("CSE_TRACE_HC_V2").is_ok();
    let mut st = SymTable::new();
    let root: ScopeId = 0;

    // Per-sym use count. SymId N is interned exactly once (at first
    // encounter); subsequent finds in descendant scopes resolve to N and
    // bump `sym_counts[N]`. v2's gate is `count > 1` (Scala's
    // `hasManyUsagesGlobal`).
    let mut sym_counts: HashMap<SymId, u32> = HashMap::new();
    // First-construction expr per sym (for ValDef emission).
    let mut sym_expr: HashMap<SymId, Expr> = HashMap::new();
    // Post-order interning order — children before parents — preserves
    // the topo invariant `dfs_schedule` provides to v1.
    let mut sym_order: Vec<SymId> = Vec::new();

    #[allow(clippy::too_many_arguments)]
    fn visit(
        e: &Expr,
        st: &mut SymTable,
        scope: ScopeId,
        sym_counts: &mut HashMap<SymId, u32>,
        sym_expr: &mut HashMap<SymId, Expr>,
        sym_order: &mut Vec<SymId>,
        trace: bool,
    ) {
        // Bottom-up: visit children first with the appropriate scope
        // push at every Scala-equivalent thunk boundary.
        match e {
            Expr::If(if_op) => {
                visit(
                    &if_op.condition,
                    st,
                    scope,
                    sym_counts,
                    sym_expr,
                    sym_order,
                    trace,
                );
                let s_t = st.new_scope(scope);
                if trace {
                    eprintln!("[HCv2/scope] parent={} child={} (If.true)", scope, s_t);
                }
                visit(
                    &if_op.true_branch,
                    st,
                    s_t,
                    sym_counts,
                    sym_expr,
                    sym_order,
                    trace,
                );
                let s_f = st.new_scope(scope);
                if trace {
                    eprintln!("[HCv2/scope] parent={} child={} (If.false)", scope, s_f);
                }
                visit(
                    &if_op.false_branch,
                    st,
                    s_f,
                    sym_counts,
                    sym_expr,
                    sym_order,
                    trace,
                );
            }
            Expr::BinOp(s)
                if matches!(
                    s.expr.kind,
                    ergotree_ir::mir::bin_op::BinOpKind::Logical(
                        ergotree_ir::mir::bin_op::LogicalOp::And
                            | ergotree_ir::mir::bin_op::LogicalOp::Or
                    )
                ) =>
            {
                visit(
                    &s.expr.left,
                    st,
                    scope,
                    sym_counts,
                    sym_expr,
                    sym_order,
                    trace,
                );
                let s_r = st.new_scope(scope);
                if trace {
                    eprintln!(
                        "[HCv2/scope] parent={} child={} (&&/|| right)",
                        scope, s_r
                    );
                }
                visit(
                    &s.expr.right,
                    st,
                    s_r,
                    sym_counts,
                    sym_expr,
                    sym_order,
                    trace,
                );
            }
            Expr::FuncValue(fv) => {
                let s_b = st.new_scope(scope);
                if trace {
                    eprintln!(
                        "[HCv2/scope] parent={} child={} (FuncValue body)",
                        scope, s_b
                    );
                }
                visit(fv.body(), st, s_b, sym_counts, sym_expr, sym_order, trace);
            }
            _ => {
                for c in direct_children(e) {
                    visit(c, st, scope, sym_counts, sym_expr, sym_order, trace);
                }
            }
        }
        // Post-order: intern THIS expression after its children.
        let (sym, is_new) = st.find_or_intern(e, scope);
        if is_new {
            sym_expr.insert(sym, e.clone());
            sym_order.push(sym);
            sym_counts.insert(sym, 1);
        } else {
            *sym_counts.entry(sym).or_insert(0) += 1;
        }
        if trace {
            eprintln!(
                "[HCv2/visit] scope={} sym={} new={} count={} :: {}",
                scope,
                sym,
                is_new,
                sym_counts[&sym],
                short_expr(e)
            );
        }
    }

    // Span-strip the intern-walk input (ExprKey relies on PartialEq on
    // span-stripped Expr). `apply_cse` strips spans at pipeline entry;
    // this is defensive parity with v1.
    let expr_for_walk = strip_source_spans(expr.clone());
    visit(
        &expr_for_walk,
        &mut st,
        root,
        &mut sym_counts,
        &mut sym_expr,
        &mut sym_order,
        trace,
    );

    let branch_local_ids = collect_branch_local_val_ids(&expr);
    let mut env: Vec<(Expr, u32)> = Vec::new();
    let mut next_id = find_max_val_id(&expr).max(global_max_id) + 1;

    for sym in &sym_order {
        let count = *sym_counts.get(sym).unwrap_or(&0);
        if count < 2 {
            if trace {
                eprintln!(
                    "[HCv2/{:?}] skip count={} sym={} :: {}",
                    mode,
                    count,
                    sym,
                    short_expr(&sym_expr[sym])
                );
            }
            continue;
        }
        // QB1 Phase 3.2 / S24 architectural gate: only emit syms whose
        // creation scope is the dispatch's entry scope (root = 0). Syms
        // created inside a thunk (If branch / &&/|| right arm / FuncValue
        // body) belong to that sub-scope's bodyDefs in Scala's mechanism;
        // emitting them at this dispatch level over-extracts. The Branch-
        // mode dispatch (via `apply_cse_within_branches`) re-traverses
        // each thunk subtree with its own SymTable and extracts those
        // syms at branch scope.
        //
        // Empirical falsification at S23 cross-fixture baseline: WITHOUT
        // this gate, v2 over-extracts on phoenix (+1 ValDef) /
        // spectrum_n2t (+4B) / spectrum_t2t (+4B) / ergoraffle (-69) /
        // paideia (-67) / sigmao (-30) / gluon (-185). With the gate,
        // these syms get filtered at root and (modulo branch-mode
        // re-traversal) approach HC=0 byte parity.
        //
        // QB1 Phase 3.3 / S25 — β.1 canonical-count + LCA-at-root gate
        // attempted, FALSIFIED (42nd cumulative fingerprint). Replacing
        // this filter with `canonical_count >= 2 AND LCA(canonical_scopes)
        // == root` recovered dexy MATCH (+44 → MATCH) + sigmao MATCH
        // (-6 → MATCH) + paideia direction-flip (+10 → -86) BUT broke 5
        // S24-MATCH-held fixtures: chaincash +7 / ergoraffle -116 /
        // phoenix +2 / rosen -10 / skyharbor -16. Class: aggregate
        // canonical_count over-counts cross-sibling occurrences relative
        // to Scala's per-sym `usageMap` counting. Scala creates DISTINCT
        // syms per sibling thunk (Thunks.scala) and counts uses per-sym;
        // structurally-equal cross-sibling shapes have separate counts
        // and do NOT aggregate under `hasManyUsagesGlobal`. SymTable's
        // `canonical` / `canonical_counts` / `canonical_scopes` fields
        // remain in tree as durable diagnostic infrastructure for S26+
        // (likely candidate α — Scala-faithful `_globalDefs` fallback —
        // pending metals re-read of Thunks.scala `findOrCreateDefinition`).
        let sym_home = st.scope_of(*sym);
        if sym_home != root {
            if trace {
                eprintln!(
                    "[HCv2/{:?}] skip-sub-scope count={} sym={} sym_scope={} :: {}",
                    mode,
                    count,
                    sym,
                    sym_home,
                    short_expr(&sym_expr[sym])
                );
            }
            continue;
        }
        let node = &sym_expr[sym];
        if !is_extractable(node) {
            if trace {
                eprintln!(
                    "[HCv2/{:?}] skip-not-extractable count={} sym={} :: {}",
                    mode,
                    count,
                    sym,
                    short_expr(node)
                );
            }
            continue;
        }
        if !is_graph_shared(node) {
            if trace {
                eprintln!(
                    "[HCv2/{:?}] skip-not-shared count={} sym={} :: {}",
                    mode,
                    count,
                    sym,
                    short_expr(node)
                );
            }
            continue;
        }
        if references_locally_defined(node, &branch_local_ids) {
            if trace {
                eprintln!(
                    "[HCv2/{:?}] skip-branch-local count={} sym={} :: {}",
                    mode,
                    count,
                    sym,
                    short_expr(node)
                );
            }
            continue;
        }
        if trace {
            eprintln!(
                "[HCv2/{:?}] extract id={} sym={} count={} :: {}",
                mode,
                next_id,
                sym,
                count,
                short_expr(node)
            );
        }
        env.push((node.clone(), next_id));
        next_id += 1;
    }

    // QB Session 26 — read-only canonical dump (Probe 1.2). Consumes the
    // S25-preserved SymTable.canonical / canonical_counts / canonical_scopes
    // accessors. For each canonical SymId observed across the whole graph,
    // emit `csym / count / sym_home / LCA(scopes) / scopes / extracted?`
    // so dexy/paideia/chaincash/ergoraffle can be cross-referenced against
    // HC=0 ValDefs. Independently confirms the metals finding that Scala
    // counts per-sym (not aggregate-canonical) via `mainG.usageMap`.
    if std::env::var("CSE_TRACE_CANONICAL").is_ok() {
        let extracted: std::collections::HashSet<SymId> = sym_order
            .iter()
            .copied()
            .filter(|s| {
                let cnt = *sym_counts.get(s).unwrap_or(&0);
                cnt >= 2 && st.scope_of(*s) == root
            })
            .collect();
        for c_sym in st.canonical_syms() {
            let count = st.canonical_count(c_sym);
            let scopes = st.canonical_scopes_for(c_sym);
            let lca = st.lca_of_scopes(scopes);
            let sym_home = st.scope_of(c_sym);
            let local_count = *sym_counts.get(&c_sym).unwrap_or(&0);
            let expr_dbg = sym_expr
                .get(&c_sym)
                .map(|e| short_expr(e))
                .unwrap_or_else(|| "<not-in-this-dispatch>".to_string());
            eprintln!(
                "[HCv2/{:?}/canonical] csym={} ccount={} local_count={} sym_home={} lca={} scopes={:?} extracted_root_gate={} :: {}",
                mode,
                c_sym,
                count,
                local_count,
                sym_home,
                lca,
                scopes,
                extracted.contains(&c_sym),
                expr_dbg
            );
        }
    }

    if env.is_empty() {
        return expr;
    }

    // Replacement tail — verbatim from `process_ast_graph_hash_cons` /
    // `process_ast_graph_impl`. Preserving this verbatim keeps any
    // byte-divergence under `CSE_HC_V2=1` attributable to the
    // extraction-decision change (the v2 algorithm), not to a different
    // rewrite shape.
    let mut result = expr;
    let mut final_env: Vec<(Expr, u32)> = Vec::new();
    let mut env = env;
    let mut i = 0;
    while i < env.len() {
        let (ref node, val_id) = env[i];
        let tree_count = count_occurrences(&result, node);
        if tree_count < 2 {
            i += 1;
            continue;
        }
        let node = node.clone();
        let val_use = Expr::ValUse(ValUse {
            val_id: ValId(val_id),
            tpe: expr_type(&node),
        });
        result = replace_all(&result, &node, &val_use);
        for entry in env.iter_mut().skip(i + 1) {
            let (ref mut env_expr, _) = entry;
            *env_expr = replace_all(env_expr, &node, &val_use);
        }
        final_env.push((node, val_id));
        i += 1;
    }
    let env = final_env;

    if env.is_empty() {
        return result;
    }

    let val_defs: Vec<Expr> = env
        .iter()
        .map(|(node, val_id)| {
            let rhs = build_value_recurse(node, &env);
            Expr::ValDef(Spanned {
                source_span: SourceSpan::empty(),
                expr: ValDef {
                    id: ValId(*val_id),
                    rhs: rhs.into(),
                },
            })
        })
        .collect();

    let post_next_id = val_defs
        .iter()
        .filter_map(|vd| {
            if let Expr::ValDef(s) = vd {
                Some(s.expr.id.0)
            } else {
                None
            }
        })
        .max()
        .map(|m| m + 1)
        .unwrap_or(next_id);
    let (val_defs, result, _) = lambda_rescue_post_pass(val_defs, result, post_next_id);

    match result {
        Expr::BlockValue(spanned) => {
            let mut items = val_defs;
            items.extend(spanned.expr.items);
            let items = topo_order_valdefs(items);
            Expr::BlockValue(Spanned {
                source_span: SourceSpan::empty(),
                expr: BlockValue {
                    items,
                    result: spanned.expr.result,
                },
            })
        }
        other => Expr::BlockValue(Spanned {
            source_span: SourceSpan::empty(),
            expr: BlockValue {
                items: topo_order_valdefs(val_defs),
                result: other.into(),
            },
        }),
    }
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

/// Check if `target` appears anywhere in `tree` that is NOT exclusively
/// inside the right arm of a logical &&/|| chain. In Scala, the right arm
/// of &&/|| is wrapped in a ThunkDef — expressions first created there
/// are scope-local and not shared with the parent scope.
///
/// If-branches are ALSO ThunkDef scopes. Importantly, once inside an If
/// branch, And/Or left-arm "resets" do NOT escape the if-branch scope —
/// the outer processAstGraph never sees those expressions at root scope.
fn appears_in_main_scope(tree: &Expr, target: &Expr) -> bool {
    appears_in_main_scope_inner(tree, target, false, false)
}

fn appears_in_main_scope_inner(
    expr: &Expr,
    target: &Expr,
    in_and_thunk: bool, // true = inside the right arm of && / ||
    in_if_branch: bool, // true = inside an If true/false branch (cannot be escaped by And/Or)
) -> bool {
    if expr == target {
        return !in_and_thunk && !in_if_branch;
    }
    match expr {
        Expr::BinOp(s)
            if matches!(
                s.expr.kind,
                ergotree_ir::mir::bin_op::BinOpKind::Logical(
                    ergotree_ir::mir::bin_op::LogicalOp::And
                        | ergotree_ir::mir::bin_op::LogicalOp::Or
                )
            ) =>
        {
            // Left arm: reset in_and_thunk (left arm is eager = main scope for this ThunkDef).
            // BUT in_if_branch stays: being in an And-left inside an If branch does NOT
            // make the expression visible at the outer (root) processAstGraph scope.
            appears_in_main_scope_inner(&s.expr.left, target, false, in_if_branch)
                || appears_in_main_scope_inner(&s.expr.right, target, true, in_if_branch)
        }
        // In Scala's graph IR, If branches are ThunkDefs (lazy evaluation).
        // The condition is eagerly evaluated (same scope as enclosing), but
        // true/false branches are separate ThunkDef scopes that cannot be escaped.
        Expr::If(if_op) => {
            appears_in_main_scope_inner(&if_op.condition, target, in_and_thunk, in_if_branch)
                || appears_in_main_scope_inner(&if_op.true_branch, target, false, true)
                || appears_in_main_scope_inner(&if_op.false_branch, target, false, true)
        }
        _ => direct_children(expr)
            .into_iter()
            .any(|child| appears_in_main_scope_inner(child, target, in_and_thunk, in_if_branch)),
    }
}

/// Check if `target` appears anywhere outside of If branches.
/// Unlike `appears_in_main_scope`, this does NOT treat And/Or right arms
/// as separate scopes. In Scala's graph IR, root-scope expressions
/// (those with GlobalVars input) have their references counted across
/// And/Or ThunkDef scopes, but NOT across If branches.
fn appears_outside_if_branches(tree: &Expr, target: &Expr) -> bool {
    appears_outside_if_inner(tree, target, false)
}

fn appears_outside_if_inner(expr: &Expr, target: &Expr, in_if_branch: bool) -> bool {
    if expr == target {
        return !in_if_branch;
    }
    match expr {
        // If branches are separate scopes — expressions only inside
        // If branches don't get root-level ValDefs.
        Expr::If(if_op) => {
            appears_outside_if_inner(&if_op.condition, target, in_if_branch)
                || appears_outside_if_inner(&if_op.true_branch, target, true)
                || appears_outside_if_inner(&if_op.false_branch, target, true)
        }
        // And/Or right arms are NOT separate scopes for root-scope expressions.
        // References from And/Or ThunkDefs count toward root scope.
        _ => direct_children(expr)
            .into_iter()
            .any(|child| appears_outside_if_inner(child, target, in_if_branch)),
    }
}

/// Get the type of an expression.
fn expr_type(expr: &Expr) -> SType {
    expr.tpe()
}

/// Count distinct top-level siblings of `tree` that contain at least one
/// occurrence of `target`. For a `BlockValue` outer scope, "siblings" are
/// the items[] entries plus the result expression. The LCA of target's uses
/// is the outer scope iff at least 2 distinct siblings contain the target —
/// then Scala's segregation emits the ValDef at the outer scope (root).
///
/// sig-15 paideia_stake_state S3: SelectField on a stable input
/// (`box.tokens(i)._f`) used inside If branches across 2+ sibling val RHSs
/// or sibling thunks of an outer Or-chain (`validStakeTx || validEmitTx ||
/// validUnstakeTx`) has its LCA at root despite no use being outside any If.
/// `appears_outside_if_branches` would reject — it requires a use literally
/// outside any If — but Scala hoists because the per-thunk syms unify at
/// the outer scope when ≥2 sibling thunks reference the same input sym.
fn count_distinct_top_level_containers(tree: &Expr, target: &Expr) -> usize {
    // Step into wrapping unary nodes (`sigmaProp` / `BoolToSigmaProp`) so the
    // outer Or/And chain inside is reached. HIR inlines single-use vals, so
    // `sigmaProp(validStakeTx || validEmitTx || validUnstakeTx)` arrives here
    // as a single result expression rather than three sibling items[]; we need
    // to walk into the chain to count the original sibling thunks.
    let inner = match tree {
        Expr::BoolToSigmaProp(bsp) => &*bsp.input,
        _ => tree,
    };
    let siblings: Vec<&Expr> = match inner {
        Expr::BlockValue(bv) => {
            let mut v: Vec<&Expr> = bv.expr.items.iter().collect();
            // If the block's result is itself a logical chain, expand its arms
            // as additional sibling containers (arms post-HIR-inline).
            let result_inner: &Expr = match &*bv.expr.result {
                Expr::BoolToSigmaProp(bsp) => &bsp.input,
                other => other,
            };
            match result_inner {
                Expr::BinOp(s)
                    if matches!(
                        s.expr.kind,
                        ergotree_ir::mir::bin_op::BinOpKind::Logical(
                            ergotree_ir::mir::bin_op::LogicalOp::Or
                                | ergotree_ir::mir::bin_op::LogicalOp::And
                        )
                    ) =>
                {
                    collect_logical_chain_arms(result_inner, s.expr.kind, &mut v);
                }
                _ => v.push(&bv.expr.result),
            }
            v
        }
        Expr::BinOp(s)
            if matches!(
                s.expr.kind,
                ergotree_ir::mir::bin_op::BinOpKind::Logical(
                    ergotree_ir::mir::bin_op::LogicalOp::Or
                        | ergotree_ir::mir::bin_op::LogicalOp::And
                )
            ) =>
        {
            let mut v: Vec<&Expr> = Vec::new();
            collect_logical_chain_arms(inner, s.expr.kind, &mut v);
            v
        }
        _ => vec![inner],
    };
    siblings
        .iter()
        .filter(|s| count_occurrences(s, target) > 0)
        .count()
}

fn collect_logical_chain_arms<'a>(
    expr: &'a Expr,
    kind: ergotree_ir::mir::bin_op::BinOpKind,
    out: &mut Vec<&'a Expr>,
) {
    if let Expr::BinOp(s) = expr {
        if s.expr.kind == kind {
            collect_logical_chain_arms(&s.expr.left, kind, out);
            collect_logical_chain_arms(&s.expr.right, kind, out);
            return;
        }
    }
    out.push(expr);
}

/// Estimate the depth of an expression (for sorting candidates innermost-first).
fn expr_depth(expr: &Expr) -> usize {
    match expr {
        Expr::Const(_) | Expr::GlobalVars(_) | Expr::ValUse(_) | Expr::Context | Expr::Global => 0,
        Expr::ExtractAmount(ea) => 1 + expr_depth(&ea.input),
        Expr::ExtractRegisterAs(s) => 1 + expr_depth(&s.expr.input),
        Expr::ExtractScriptBytes(esb) => 1 + expr_depth(&esb.input),
        Expr::ExtractBytes(eb) => 1 + expr_depth(&eb.input),
        Expr::ExtractId(ei) => 1 + expr_depth(&ei.input),
        Expr::ExtractCreationInfo(eci) => 1 + expr_depth(&eci.input),
        Expr::SizeOf(so) => 1 + expr_depth(&so.input),
        Expr::PropertyCall(s) => 1 + expr_depth(&s.expr.obj),
        Expr::MethodCall(s) => 1 + expr_depth(&s.expr.obj),
        Expr::SelectField(s) => 1 + expr_depth(&s.expr.input),
        Expr::OptionGet(s) => 1 + expr_depth(&s.expr.input),
        Expr::OptionIsDefined(s) => 1 + expr_depth(&s.expr.input),
        Expr::BinOp(s) => 1 + expr_depth(&s.expr.left).max(expr_depth(&s.expr.right)),
        Expr::ByIndex(s) => 1 + expr_depth(&s.expr.input),
        Expr::Upcast(uc) => 1 + expr_depth(&uc.input),
        Expr::LogicalNot(s) => 1 + expr_depth(&s.expr.input),
        Expr::Negation(s) => 1 + expr_depth(&s.expr.input),
        _ => 1,
    }
}

/// Find the maximum ValId in an expression tree.
fn find_max_val_id(expr: &Expr) -> u32 {
    match expr {
        Expr::ValDef(s) => {
            let rhs_max = find_max_val_id(&s.expr.rhs);
            s.expr.id.0.max(rhs_max)
        }
        Expr::ValUse(vu) => vu.val_id.0,
        Expr::BlockValue(s) => {
            let items_max = s.expr.items.iter().map(find_max_val_id).max().unwrap_or(0);
            let result_max = find_max_val_id(&s.expr.result);
            items_max.max(result_max)
        }
        Expr::BinOp(s) => find_max_val_id(&s.expr.left).max(find_max_val_id(&s.expr.right)),
        Expr::BoolToSigmaProp(bts) => find_max_val_id(&bts.input),
        Expr::If(if_op) => find_max_val_id(&if_op.condition)
            .max(find_max_val_id(&if_op.true_branch))
            .max(find_max_val_id(&if_op.false_branch)),
        Expr::FuncValue(fv) => {
            let args_max = fv.args().iter().map(|a| a.idx.0).max().unwrap_or(0);
            args_max.max(find_max_val_id(fv.body()))
        }
        Expr::Filter(s) => find_max_val_id(&s.expr.input).max(find_max_val_id(&s.expr.condition)),
        Expr::Exists(s) => find_max_val_id(&s.expr.input).max(find_max_val_id(&s.expr.condition)),
        Expr::ForAll(s) => find_max_val_id(&s.expr.input).max(find_max_val_id(&s.expr.condition)),
        Expr::Map(s) => find_max_val_id(&s.expr.input).max(find_max_val_id(&s.expr.mapper)),
        Expr::Fold(s) => find_max_val_id(&s.expr.input)
            .max(find_max_val_id(&s.expr.zero))
            .max(find_max_val_id(&s.expr.fold_op)),
        Expr::PropertyCall(s) => find_max_val_id(&s.expr.obj),
        Expr::MethodCall(s) => {
            let args_max = s.expr.args.iter().map(find_max_val_id).max().unwrap_or(0);
            find_max_val_id(&s.expr.obj).max(args_max)
        }
        Expr::ExtractAmount(ea) => find_max_val_id(&ea.input),
        Expr::ExtractRegisterAs(s) => find_max_val_id(&s.expr.input),
        Expr::ExtractScriptBytes(esb) => find_max_val_id(&esb.input),
        Expr::SizeOf(so) => find_max_val_id(&so.input),
        Expr::ByIndex(s) => {
            let idx_max = find_max_val_id(&s.expr.index);
            let def_max = s
                .expr
                .default
                .as_ref()
                .map(|d| find_max_val_id(d))
                .unwrap_or(0);
            find_max_val_id(&s.expr.input).max(idx_max).max(def_max)
        }
        Expr::SelectField(s) => find_max_val_id(&s.expr.input),
        Expr::OptionGet(s) => find_max_val_id(&s.expr.input),
        Expr::OptionIsDefined(s) => find_max_val_id(&s.expr.input),
        Expr::OptionGetOrElse(s) => {
            find_max_val_id(&s.expr.input).max(find_max_val_id(&s.expr.default))
        }
        Expr::Slice(s) => find_max_val_id(&s.expr.input)
            .max(find_max_val_id(&s.expr.from))
            .max(find_max_val_id(&s.expr.until)),
        Expr::LogicalNot(s) => find_max_val_id(&s.expr.input),
        Expr::Negation(s) => find_max_val_id(&s.expr.input),
        Expr::SigmaPropBytes(spb) => find_max_val_id(&spb.input),
        Expr::Upcast(uc) => find_max_val_id(&uc.input),
        Expr::Downcast(dc) => find_max_val_id(&dc.input),
        Expr::CalcBlake2b256(cb) => find_max_val_id(&cb.input),
        Expr::SigmaAnd(sa) => sa.items.iter().map(find_max_val_id).max().unwrap_or(0),
        Expr::SigmaOr(so) => so.items.iter().map(find_max_val_id).max().unwrap_or(0),
        Expr::Tuple(t) => t.items.iter().map(find_max_val_id).max().unwrap_or(0),
        Expr::TreeLookup(s) => find_max_val_id(&s.expr.tree)
            .max(find_max_val_id(&s.expr.key))
            .max(find_max_val_id(&s.expr.proof)),
        Expr::Apply(app) => {
            let args_max = app.args.iter().map(find_max_val_id).max().unwrap_or(0);
            find_max_val_id(&app.func).max(args_max)
        }
        Expr::And(a) => find_max_val_id(&a.expr.input),
        Expr::Or(o) => find_max_val_id(&o.expr.input),
        Expr::Atleast(s) => find_max_val_id(&s.bound).max(find_max_val_id(&s.input)),
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            items.iter().map(find_max_val_id).max().unwrap_or(0)
        }
        _ => 0,
    }
}

/// Count how many times `target` appears in `expr`.
///
/// COVERAGE: counting walker (WS-E.1). Same speculative-addition risk as
/// `direct_children` — see WS-E.1 matrix at
/// [`IR-PASS-COVERAGE-MATRIX.md`](../../tests/fixtures/significant_15/parity-handoffs/IR-PASS-COVERAGE-MATRIX.md).
/// Currently missing arms: Append, ByteArrayToLong, CreateProveDhTuple,
/// CreateProveDlog, DecodePoint, Exponentiate, LongToByteArray,
/// MultiplyGroup. Each addition requires per-fixture Metals + trace.
fn count_occurrences(expr: &Expr, target: &Expr) -> usize {
    let mut count = if expr == target { 1 } else { 0 };
    // Recurse into children
    match expr {
        Expr::BinOp(s) => {
            count += count_occurrences(&s.expr.left, target);
            count += count_occurrences(&s.expr.right, target);
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                count += count_occurrences(item, target);
            }
            count += count_occurrences(&s.expr.result, target);
        }
        Expr::ValDef(s) => {
            count += count_occurrences(&s.expr.rhs, target);
        }
        Expr::BoolToSigmaProp(bts) => {
            count += count_occurrences(&bts.input, target);
        }
        Expr::If(if_op) => {
            count += count_occurrences(&if_op.condition, target);
            count += count_occurrences(&if_op.true_branch, target);
            count += count_occurrences(&if_op.false_branch, target);
        }
        Expr::PropertyCall(s) => {
            count += count_occurrences(&s.expr.obj, target);
        }
        Expr::MethodCall(s) => {
            count += count_occurrences(&s.expr.obj, target);
            for arg in &s.expr.args {
                count += count_occurrences(arg, target);
            }
        }
        Expr::ExtractAmount(ea) => count += count_occurrences(&ea.input, target),
        Expr::ExtractRegisterAs(s) => count += count_occurrences(&s.expr.input, target),
        Expr::ExtractScriptBytes(esb) => count += count_occurrences(&esb.input, target),
        Expr::ExtractBytes(eb) => count += count_occurrences(&eb.input, target),
        Expr::ExtractId(ei) => count += count_occurrences(&ei.input, target),
        Expr::ExtractCreationInfo(eci) => count += count_occurrences(&eci.input, target),
        Expr::SizeOf(so) => count += count_occurrences(&so.input, target),
        Expr::ByIndex(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.index, target);
            if let Some(ref d) = s.expr.default {
                count += count_occurrences(d, target);
            }
        }
        Expr::SelectField(s) => count += count_occurrences(&s.expr.input, target),
        Expr::OptionGet(s) => count += count_occurrences(&s.expr.input, target),
        Expr::OptionIsDefined(s) => count += count_occurrences(&s.expr.input, target),
        Expr::OptionGetOrElse(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.default, target);
        }
        Expr::Filter(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.condition, target);
        }
        Expr::Exists(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.condition, target);
        }
        Expr::ForAll(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.condition, target);
        }
        Expr::Map(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.mapper, target);
        }
        Expr::Fold(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.zero, target);
            count += count_occurrences(&s.expr.fold_op, target);
        }
        Expr::Slice(s) => {
            count += count_occurrences(&s.expr.input, target);
            count += count_occurrences(&s.expr.from, target);
            count += count_occurrences(&s.expr.until, target);
        }
        Expr::LogicalNot(s) => count += count_occurrences(&s.expr.input, target),
        Expr::Negation(s) => count += count_occurrences(&s.expr.input, target),
        Expr::SigmaPropBytes(spb) => count += count_occurrences(&spb.input, target),
        Expr::Upcast(uc) => count += count_occurrences(&uc.input, target),
        Expr::Downcast(dc) => count += count_occurrences(&dc.input, target),
        Expr::CalcBlake2b256(cb) => count += count_occurrences(&cb.input, target),
        Expr::ByteArrayToBigInt(s) => count += count_occurrences(&s.expr.input, target),
        Expr::SigmaAnd(sa) => {
            for item in sa.items.iter() {
                count += count_occurrences(item, target);
            }
        }
        Expr::SigmaOr(so) => {
            for item in so.items.iter() {
                count += count_occurrences(item, target);
            }
        }
        Expr::Tuple(t) => {
            for item in t.items.iter() {
                count += count_occurrences(item, target);
            }
        }
        Expr::TreeLookup(s) => {
            count += count_occurrences(&s.expr.tree, target);
            count += count_occurrences(&s.expr.key, target);
            count += count_occurrences(&s.expr.proof, target);
        }
        Expr::Apply(app) => {
            count += count_occurrences(&app.func, target);
            for arg in &app.args {
                count += count_occurrences(arg, target);
            }
        }
        Expr::And(a) => count += count_occurrences(&a.expr.input, target),
        Expr::Or(o) => count += count_occurrences(&o.expr.input, target),
        Expr::Atleast(s) => {
            count += count_occurrences(&s.bound, target);
            count += count_occurrences(&s.input, target);
        }
        Expr::CreateProveDhTuple(cpd) => {
            count += count_occurrences(&cpd.g, target);
            count += count_occurrences(&cpd.h, target);
            count += count_occurrences(&cpd.u, target);
            count += count_occurrences(&cpd.v, target);
        }
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                count += count_occurrences(item, target);
            }
        }
        Expr::FuncValue(fv) => {
            // Recurse into lambda body so cross-lambda occurrences are seen.
            // Mirrors Scala's mainG.hasManyUsagesGlobal which counts parent
            // edges across ThunkDef boundaries on the global IR sym graph
            // (sig-15 rosen_event_trigger: OUTPUTS(0).propBytes used 2×
            // wrapped in Blake2b256 + 1× raw inside a forall lambda body).
            count += count_occurrences(fv.body(), target);
        }
        _ => {}
    }
    count
}

/// Lambda-rescue post-pass shared between `cse_expr`'s has_lambdas branch and
/// `process_ast_graph_impl`. For each sub-expression of an already-extracted
/// ValDef RHS that ALSO appears in a lambda body in `result`, extract it as a
/// new ValDef. Mirrors Scala's `mainG.hasManyUsagesGlobal` which counts
/// parent-edge multiplicity through ThunkDef (lambda) boundaries — Rust's
/// `count_dag_usages` stops at FuncValue, undercounting candidates whose
/// outer parent-edges are all wrapped in another (already-extracted)
/// candidate but which also have a literal occurrence in some lambda body.
///
/// sig-15 rosen_event_trigger pattern: `OUTPUTS(0).propBytes` wrapped 2× in
/// `Blake2b256(...)` at top-level + 1× raw inside a forall lambda body. Main
/// extraction sees only the Blake2b256 wrapper (parent count = 1 to propBytes
/// at outer); this post-pass recovers val for propBytes when total occurrences
/// (RHS + lambda body, via FV-recursive count_occurrences) reaches ≥ 2.
fn lambda_rescue_post_pass(
    mut new_val_defs: Vec<Expr>,
    mut result: Expr,
    mut next_id: u32,
) -> (Vec<Expr>, Expr, u32) {
    loop {
        let mut subs: Vec<Expr> = Vec::new();
        for vd in &new_val_defs {
            if let Expr::ValDef(s) = vd {
                collect_subexprs(&s.expr.rhs, &mut subs);
            }
        }
        let mut unique: Vec<Expr> = Vec::new();
        for s in &subs {
            if !unique.iter().any(|u| u == s) {
                unique.push(s.clone());
            }
        }
        let mut best: Option<(Expr, i32)> = None;
        for cand in &unique {
            if !is_collectible(cand) || contains_val_use(cand) {
                continue;
            }
            // count_occurrences is FV-recursive (sees through lambda bodies).
            let mut total = 0usize;
            for vd in &new_val_defs {
                if let Expr::ValDef(s) = vd {
                    total += count_occurrences(&s.expr.rhs, cand);
                }
            }
            total += count_occurrences(&result, cand);
            if total < 2 {
                continue;
            }
            // Require ≥ 1 occurrence inside a lambda body — otherwise the
            // candidate was already considered by main extraction's outer
            // counting and either extracted or rejected.
            let in_lambda = count_in_lambda_bodies(&result, cand);
            if in_lambda == 0 {
                continue;
            }
            let depth = expr_depth(cand) as i32;
            let savings = (total as i32 - 1) * depth;
            if best.as_ref().map_or(true, |(_, s)| savings > *s) {
                best = Some((cand.clone(), savings));
            }
        }
        let Some((cand, _)) = best else { break };
        let val_id = next_id;
        next_id += 1;
        let tpe = expr_type(&cand);
        let val_use = Expr::ValUse(ValUse {
            val_id: ValId(val_id),
            tpe,
        });
        for vd in new_val_defs.iter_mut() {
            if let Expr::ValDef(s) = vd {
                let new_rhs = replace_all(&s.expr.rhs, &cand, &val_use);
                *vd = Expr::ValDef(Spanned {
                    source_span: s.source_span,
                    expr: ValDef {
                        id: s.expr.id,
                        rhs: new_rhs.into(),
                    },
                });
            }
        }
        // replace_all is FV-recursive, so lambda bodies are rewritten too.
        result = replace_all(&result, &cand, &val_use);
        new_val_defs.push(Expr::ValDef(Spanned {
            source_span: SourceSpan::empty(),
            expr: ValDef {
                id: ValId(val_id),
                rhs: cand.into(),
            },
        }));
    }
    (new_val_defs, result, next_id)
}

/// Count occurrences of `target` strictly inside FuncValue bodies of `expr`.
/// Used by the has_lambdas post-pass to gate rescue extractions: only fire
/// for candidates with at least one lambda-body occurrence (otherwise the
/// candidate would already have been considered by the main loop's
/// outer-only counting and either extracted or rejected).
fn count_in_lambda_bodies(expr: &Expr, target: &Expr) -> usize {
    match expr {
        Expr::FuncValue(fv) => count_occurrences(fv.body(), target),
        _ => direct_children(expr)
            .iter()
            .map(|c| count_in_lambda_bodies(c, target))
            .sum(),
    }
}

/// Like `count_occurrences` but stops at `Expr::If` branches — only recurses
/// into the If condition (which is evaluated at the enclosing ThunkDef scope).
///
/// Used for the S40 global bump in `process_ast_graph_branch`: expressions
/// inside `&&`/`||` right arms should be counted (Scala's global graph sees
/// them as part of the same surrounding scope), but expressions inside nested
/// `If` true/false branches must NOT be counted because they belong to child
/// ThunkDef scopes that get their own `process_ast_graph_branch` pass.
///
/// Without this restriction, an inlined expression like
/// `ExtractAmount(If(isLastSale, OUTPUTS(4), OUTPUTS(5)))` inside the
/// isLastSale false branch would inflate the global count of `OUTPUTS(4)`,
/// causing spurious extraction that Scala never performs.
///
/// COVERAGE: counting walker (WS-E.1). Arms mirror `count_occurrences`
/// minus the inner-If recursion. Same gaps; same speculative-addition
/// rule. See [`IR-PASS-COVERAGE-MATRIX.md`](../../tests/fixtures/significant_15/parity-handoffs/IR-PASS-COVERAGE-MATRIX.md).
fn count_occurrences_no_inner_if(expr: &Expr, target: &Expr) -> usize {
    let mut count = if expr == target { 1 } else { 0 };
    match expr {
        Expr::BinOp(s) => {
            count += count_occurrences_no_inner_if(&s.expr.left, target);
            count += count_occurrences_no_inner_if(&s.expr.right, target);
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                count += count_occurrences_no_inner_if(item, target);
            }
            count += count_occurrences_no_inner_if(&s.expr.result, target);
        }
        Expr::ValDef(s) => {
            count += count_occurrences_no_inner_if(&s.expr.rhs, target);
        }
        Expr::BoolToSigmaProp(bts) => {
            count += count_occurrences_no_inner_if(&bts.input, target);
        }
        Expr::If(if_op) => {
            // Recurse only into condition — true/false branches are child ThunkDefs.
            count += count_occurrences_no_inner_if(&if_op.condition, target);
        }
        Expr::PropertyCall(s) => {
            count += count_occurrences_no_inner_if(&s.expr.obj, target);
        }
        Expr::MethodCall(s) => {
            count += count_occurrences_no_inner_if(&s.expr.obj, target);
            for arg in &s.expr.args {
                count += count_occurrences_no_inner_if(arg, target);
            }
        }
        Expr::ExtractAmount(ea) => count += count_occurrences_no_inner_if(&ea.input, target),
        Expr::ExtractRegisterAs(s) => count += count_occurrences_no_inner_if(&s.expr.input, target),
        Expr::ExtractScriptBytes(esb) => count += count_occurrences_no_inner_if(&esb.input, target),
        Expr::ExtractBytes(eb) => count += count_occurrences_no_inner_if(&eb.input, target),
        Expr::ExtractId(ei) => count += count_occurrences_no_inner_if(&ei.input, target),
        Expr::ExtractCreationInfo(eci) => {
            count += count_occurrences_no_inner_if(&eci.input, target)
        }
        Expr::SizeOf(so) => count += count_occurrences_no_inner_if(&so.input, target),
        Expr::ByIndex(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.index, target);
            if let Some(ref d) = s.expr.default {
                count += count_occurrences_no_inner_if(d, target);
            }
        }
        Expr::SelectField(s) => count += count_occurrences_no_inner_if(&s.expr.input, target),
        Expr::OptionGet(s) => count += count_occurrences_no_inner_if(&s.expr.input, target),
        Expr::OptionIsDefined(s) => count += count_occurrences_no_inner_if(&s.expr.input, target),
        Expr::OptionGetOrElse(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.default, target);
        }
        Expr::Filter(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.condition, target);
        }
        Expr::Exists(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.condition, target);
        }
        Expr::ForAll(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.condition, target);
        }
        Expr::Map(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.mapper, target);
        }
        Expr::Fold(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.zero, target);
            count += count_occurrences_no_inner_if(&s.expr.fold_op, target);
        }
        Expr::Slice(s) => {
            count += count_occurrences_no_inner_if(&s.expr.input, target);
            count += count_occurrences_no_inner_if(&s.expr.from, target);
            count += count_occurrences_no_inner_if(&s.expr.until, target);
        }
        Expr::LogicalNot(s) => count += count_occurrences_no_inner_if(&s.expr.input, target),
        Expr::Negation(s) => count += count_occurrences_no_inner_if(&s.expr.input, target),
        Expr::SigmaPropBytes(spb) => count += count_occurrences_no_inner_if(&spb.input, target),
        Expr::Upcast(uc) => count += count_occurrences_no_inner_if(&uc.input, target),
        Expr::Downcast(dc) => count += count_occurrences_no_inner_if(&dc.input, target),
        Expr::CalcBlake2b256(cb) => count += count_occurrences_no_inner_if(&cb.input, target),
        Expr::ByteArrayToBigInt(s) => count += count_occurrences_no_inner_if(&s.expr.input, target),
        Expr::SigmaAnd(sa) => {
            for item in sa.items.iter() {
                count += count_occurrences_no_inner_if(item, target);
            }
        }
        Expr::SigmaOr(so) => {
            for item in so.items.iter() {
                count += count_occurrences_no_inner_if(item, target);
            }
        }
        Expr::Tuple(t) => {
            for item in t.items.iter() {
                count += count_occurrences_no_inner_if(item, target);
            }
        }
        Expr::TreeLookup(s) => {
            count += count_occurrences_no_inner_if(&s.expr.tree, target);
            count += count_occurrences_no_inner_if(&s.expr.key, target);
            count += count_occurrences_no_inner_if(&s.expr.proof, target);
        }
        Expr::Apply(app) => {
            count += count_occurrences_no_inner_if(&app.func, target);
            for arg in &app.args {
                count += count_occurrences_no_inner_if(arg, target);
            }
        }
        Expr::And(a) => count += count_occurrences_no_inner_if(&a.expr.input, target),
        Expr::Or(o) => count += count_occurrences_no_inner_if(&o.expr.input, target),
        Expr::Atleast(s) => {
            count += count_occurrences_no_inner_if(&s.bound, target);
            count += count_occurrences_no_inner_if(&s.input, target);
        }
        Expr::CreateProveDhTuple(cpd) => {
            count += count_occurrences_no_inner_if(&cpd.g, target);
            count += count_occurrences_no_inner_if(&cpd.h, target);
            count += count_occurrences_no_inner_if(&cpd.u, target);
            count += count_occurrences_no_inner_if(&cpd.v, target);
        }
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                count += count_occurrences_no_inner_if(item, target);
            }
        }
        Expr::FuncValue(_) => {}
        _ => {}
    }
    count
}

/// Replace all occurrences of `target` with `replacement` in the expression tree.
///
/// COVERAGE: this is a **completeness walker** (WS-E.1). Adding a
/// missed arm is monotonic-direction (cannot regress, only fix the
/// silent-failure case where a substitution doesn't recurse and leaves
/// a dangling target reference). Canonical fix: chaincash S76 added
/// the `Append` arm — single trace, single arm, narrow fix. New
/// additions still need a concrete fixture failure trace per arm.
/// See [`IR-PASS-COVERAGE-MATRIX.md`](../../tests/fixtures/significant_15/parity-handoffs/IR-PASS-COVERAGE-MATRIX.md).
fn replace_all(expr: &Expr, target: &Expr, replacement: &Expr) -> Expr {
    if expr == target {
        return replacement.clone();
    }
    match expr {
        Expr::BinOp(s) => {
            let new_left = replace_all(&s.expr.left, target, replacement);
            let new_right = replace_all(&s.expr.right, target, replacement);
            Expr::BinOp(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::bin_op::BinOp {
                    kind: s.expr.kind,
                    left: new_left.into(),
                    right: new_right.into(),
                },
            })
        }
        Expr::BlockValue(s) => {
            let new_items: Vec<Expr> = s
                .expr
                .items
                .iter()
                .map(|i| replace_all(i, target, replacement))
                .collect();
            let new_result = replace_all(&s.expr.result, target, replacement);
            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: new_items,
                    result: new_result.into(),
                },
            })
        }
        Expr::ValDef(s) => {
            let new_rhs = replace_all(&s.expr.rhs, target, replacement);
            Expr::ValDef(Spanned {
                source_span: s.source_span,
                expr: ValDef {
                    id: s.expr.id,
                    rhs: new_rhs.into(),
                },
            })
        }
        Expr::BoolToSigmaProp(bts) => {
            let new_input = replace_all(&bts.input, target, replacement);
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: new_input.into(),
            })
        }
        Expr::If(if_op) => {
            let new_cond = replace_all(&if_op.condition, target, replacement);
            let new_then = replace_all(&if_op.true_branch, target, replacement);
            let new_else = replace_all(&if_op.false_branch, target, replacement);
            Expr::If(ergotree_ir::mir::if_op::If {
                condition: new_cond.into(),
                true_branch: new_then.into(),
                false_branch: new_else.into(),
            })
        }
        Expr::PropertyCall(s) => {
            let new_obj = replace_all(&s.expr.obj, target, replacement);
            Expr::PropertyCall(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::property_call::PropertyCall {
                    obj: new_obj.into(),
                    method: s.expr.method.clone(),
                },
            })
        }
        Expr::MethodCall(s) => {
            let new_obj = replace_all(&s.expr.obj, target, replacement);
            let new_args: Vec<Expr> = s
                .expr
                .args
                .iter()
                .map(|a| replace_all(a, target, replacement))
                .collect();
            ergotree_ir::mir::method_call::MethodCall::with_type_args(
                new_obj,
                s.expr.method.clone(),
                new_args,
                s.expr.explicit_type_args.clone(),
            )
            .map(|mc| {
                Expr::MethodCall(Spanned {
                    source_span: s.source_span,
                    expr: mc,
                })
            })
            .unwrap_or_else(|_| Expr::MethodCall(s.clone()))
        }
        Expr::ExtractAmount(ea) => {
            let new_input = replace_all(&ea.input, target, replacement);
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: new_input.into(),
            })
        }
        Expr::ExtractRegisterAs(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            Expr::ExtractRegisterAs(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::extract_reg_as::ExtractRegisterAs {
                    input: new_input.into(),
                    register_id: s.expr.register_id,
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        Expr::ExtractScriptBytes(esb) => {
            let new_input = replace_all(&esb.input, target, replacement);
            Expr::ExtractScriptBytes(ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes {
                input: new_input.into(),
            })
        }
        Expr::ExtractBytes(eb) => {
            let new_input = replace_all(&eb.input, target, replacement);
            Expr::ExtractBytes(ergotree_ir::mir::extract_bytes::ExtractBytes {
                input: new_input.into(),
            })
        }
        Expr::ExtractId(ei) => {
            let new_input = replace_all(&ei.input, target, replacement);
            Expr::ExtractId(ergotree_ir::mir::extract_id::ExtractId {
                input: new_input.into(),
            })
        }
        Expr::ExtractCreationInfo(eci) => {
            let new_input = replace_all(&eci.input, target, replacement);
            Expr::ExtractCreationInfo(
                ergotree_ir::mir::extract_creation_info::ExtractCreationInfo {
                    input: new_input.into(),
                },
            )
        }
        Expr::SizeOf(so) => {
            let new_input = replace_all(&so.input, target, replacement);
            Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
                input: new_input.into(),
            })
        }
        Expr::ByIndex(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_index = replace_all(&s.expr.index, target, replacement);
            let new_default = s
                .expr
                .default
                .as_ref()
                .map(|d| Box::new(replace_all(d, target, replacement)));
            ergotree_ir::mir::coll_by_index::ByIndex::new(new_input, new_index, new_default)
                .map(|bi| {
                    Expr::ByIndex(Spanned {
                        source_span: s.source_span,
                        expr: bi,
                    })
                })
                .unwrap_or_else(|_| Expr::ByIndex(s.clone()))
        }
        Expr::SelectField(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            ergotree_ir::mir::select_field::SelectField::new(new_input, s.expr.field_index)
                .map(|sf| {
                    Expr::SelectField(Spanned {
                        source_span: s.source_span,
                        expr: sf,
                    })
                })
                .unwrap_or_else(|_| Expr::SelectField(s.clone()))
        }
        Expr::OptionGet(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            ergotree_ir::mir::option_get::OptionGet::try_build(new_input)
                .map(|og| {
                    Expr::OptionGet(Spanned {
                        source_span: s.source_span,
                        expr: og,
                    })
                })
                .unwrap_or_else(|_| Expr::OptionGet(s.clone()))
        }
        Expr::OptionIsDefined(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            ergotree_ir::mir::option_is_defined::OptionIsDefined::try_build(new_input)
                .map(|oid| {
                    Expr::OptionIsDefined(Spanned {
                        source_span: s.source_span,
                        expr: oid,
                    })
                })
                .unwrap_or_else(|_| Expr::OptionIsDefined(s.clone()))
        }
        Expr::OptionGetOrElse(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_default = replace_all(&s.expr.default, target, replacement);
            ergotree_ir::mir::option_get_or_else::OptionGetOrElse::new(new_input, new_default)
                .map(|oge| {
                    Expr::OptionGetOrElse(Spanned {
                        source_span: s.source_span,
                        expr: oge,
                    })
                })
                .unwrap_or_else(|_| Expr::OptionGetOrElse(s.clone()))
        }
        Expr::Filter(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_cond = replace_all(&s.expr.condition, target, replacement);
            Expr::Filter(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_filter::Filter {
                    input: new_input.into(),
                    condition: new_cond.into(),
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        Expr::Exists(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_cond = replace_all(&s.expr.condition, target, replacement);
            Expr::Exists(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_exists::Exists {
                    input: new_input.into(),
                    condition: new_cond.into(),
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        Expr::ForAll(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_cond = replace_all(&s.expr.condition, target, replacement);
            Expr::ForAll(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_forall::ForAll {
                    input: new_input.into(),
                    condition: new_cond.into(),
                    elem_tpe: s.expr.elem_tpe.clone(),
                },
            })
        }
        Expr::Map(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_mapper = replace_all(&s.expr.mapper, target, replacement);
            ergotree_ir::mir::coll_map::Map::new(new_input, new_mapper)
                .map(|m| {
                    Expr::Map(Spanned {
                        source_span: s.source_span,
                        expr: m,
                    })
                })
                .unwrap_or_else(|_| Expr::Map(s.clone()))
        }
        Expr::Fold(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_zero = replace_all(&s.expr.zero, target, replacement);
            let new_fold_op = replace_all(&s.expr.fold_op, target, replacement);
            ergotree_ir::mir::coll_fold::Fold::new(new_input, new_zero, new_fold_op)
                .map(|f| {
                    Expr::Fold(Spanned {
                        source_span: s.source_span,
                        expr: f,
                    })
                })
                .unwrap_or_else(|_| Expr::Fold(s.clone()))
        }
        Expr::Slice(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_from = replace_all(&s.expr.from, target, replacement);
            let new_until = replace_all(&s.expr.until, target, replacement);
            Expr::Slice(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::coll_slice::Slice {
                    input: new_input.into(),
                    from: new_from.into(),
                    until: new_until.into(),
                },
            })
        }
        Expr::Append(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            let new_col_2 = replace_all(&s.expr.col_2, target, replacement);
            ergotree_ir::mir::coll_append::Append::new(new_input, new_col_2)
                .map(|a| {
                    Expr::Append(Spanned {
                        source_span: s.source_span,
                        expr: a,
                    })
                })
                .unwrap_or_else(|_| Expr::Append(s.clone()))
        }
        Expr::Exponentiate(e) => {
            let new_left = replace_all(&e.left, target, replacement);
            let new_right = replace_all(&e.right, target, replacement);
            ergotree_ir::mir::exponentiate::Exponentiate::new(new_left, new_right)
                .map(Expr::Exponentiate)
                .unwrap_or_else(|_| Expr::Exponentiate(e.clone()))
        }
        Expr::MultiplyGroup(m) => {
            let new_left = replace_all(&m.left, target, replacement);
            let new_right = replace_all(&m.right, target, replacement);
            ergotree_ir::mir::multiply_group::MultiplyGroup::new(new_left, new_right)
                .map(Expr::MultiplyGroup)
                .unwrap_or_else(|_| Expr::MultiplyGroup(m.clone()))
        }
        Expr::LogicalNot(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            Expr::LogicalNot(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::logical_not::LogicalNot {
                    input: new_input.into(),
                },
            })
        }
        Expr::Negation(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            Expr::Negation(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::negation::Negation {
                    input: new_input.into(),
                },
            })
        }
        Expr::SigmaPropBytes(spb) => {
            let new_input = replace_all(&spb.input, target, replacement);
            Expr::SigmaPropBytes(ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes {
                input: new_input.into(),
            })
        }
        Expr::Upcast(uc) => {
            let new_input = replace_all(&uc.input, target, replacement);
            Expr::Upcast(ergotree_ir::mir::upcast::Upcast {
                input: new_input.into(),
                tpe: uc.tpe.clone(),
            })
        }
        Expr::Downcast(dc) => {
            let new_input = replace_all(&dc.input, target, replacement);
            Expr::Downcast(ergotree_ir::mir::downcast::Downcast {
                input: new_input.into(),
                tpe: dc.tpe.clone(),
            })
        }
        Expr::CalcBlake2b256(cb) => {
            let new_input = replace_all(&cb.input, target, replacement);
            Expr::CalcBlake2b256(ergotree_ir::mir::calc_blake2b256::CalcBlake2b256 {
                input: new_input.into(),
            })
        }
        Expr::ByteArrayToBigInt(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            ergotree_ir::mir::byte_array_to_bigint::ByteArrayToBigInt::try_build(new_input)
                .map(|bb| {
                    Expr::ByteArrayToBigInt(Spanned {
                        source_span: s.source_span,
                        expr: bb,
                    })
                })
                .unwrap_or_else(|_| Expr::ByteArrayToBigInt(s.clone()))
        }
        Expr::SigmaAnd(sa) => {
            let new_items: Vec<Expr> = sa
                .items
                .iter()
                .map(|i| replace_all(i, target, replacement))
                .collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: new_items
                    .try_into()
                    .expect("SigmaAnd items should have >= 2 elements"),
            })
        }
        Expr::SigmaOr(so) => {
            let new_items: Vec<Expr> = so
                .items
                .iter()
                .map(|i| replace_all(i, target, replacement))
                .collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: new_items
                    .try_into()
                    .expect("SigmaOr items should have >= 2 elements"),
            })
        }
        Expr::Tuple(t) => {
            let new_items: Vec<Expr> = t
                .items
                .iter()
                .map(|i| replace_all(i, target, replacement))
                .collect();
            Expr::Tuple(
                ergotree_ir::mir::tuple::Tuple::new(new_items)
                    .expect("Tuple should have valid number of items"),
            )
        }
        Expr::TreeLookup(s) => {
            let new_tree = replace_all(&s.expr.tree, target, replacement);
            let new_key = replace_all(&s.expr.key, target, replacement);
            let new_proof = replace_all(&s.expr.proof, target, replacement);
            Expr::TreeLookup(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::tree_lookup::TreeLookup {
                    tree: new_tree.into(),
                    key: new_key.into(),
                    proof: new_proof.into(),
                },
            })
        }
        Expr::Apply(app) => {
            let new_func = replace_all(&app.func, target, replacement);
            let new_args: Vec<Expr> = app
                .args
                .iter()
                .map(|a| replace_all(a, target, replacement))
                .collect();
            ergotree_ir::mir::apply::Apply::new(new_func, new_args)
                .map(Expr::Apply)
                .unwrap_or_else(|_| Expr::Apply(app.clone()))
        }
        Expr::And(a) => {
            let new_input = replace_all(&a.expr.input, target, replacement);
            Expr::And(ergotree_ir::source_span::Spanned {
                source_span: a.source_span,
                expr: ergotree_ir::mir::and::And {
                    input: new_input.into(),
                },
            })
        }
        Expr::Or(o) => {
            let new_input = replace_all(&o.expr.input, target, replacement);
            Expr::Or(ergotree_ir::source_span::Spanned {
                source_span: o.source_span,
                expr: ergotree_ir::mir::or::Or {
                    input: new_input.into(),
                },
            })
        }
        Expr::Collection(c) => match c {
            ergotree_ir::mir::collection::Collection::Exprs { elem_tpe, items } => {
                let new_items: Vec<Expr> = items
                    .iter()
                    .map(|i| replace_all(i, target, replacement))
                    .collect();
                Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs {
                    elem_tpe: elem_tpe.clone(),
                    items: new_items,
                })
            }
            other => Expr::Collection(other.clone()),
        },
        Expr::Atleast(s) => {
            let new_bound = replace_all(&s.bound, target, replacement);
            let new_input = replace_all(&s.input, target, replacement);
            ergotree_ir::mir::atleast::Atleast::new(new_bound, new_input)
                .map(Expr::Atleast)
                .unwrap_or_else(|_| Expr::Atleast(s.clone()))
        }
        Expr::CreateProveDhTuple(cpd) => {
            let new_g = replace_all(&cpd.g, target, replacement);
            let new_h = replace_all(&cpd.h, target, replacement);
            let new_u = replace_all(&cpd.u, target, replacement);
            let new_v = replace_all(&cpd.v, target, replacement);
            ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple::new(
                new_g, new_h, new_u, new_v,
            )
            .map(Expr::CreateProveDhTuple)
            .unwrap_or_else(|_| Expr::CreateProveDhTuple(cpd.clone()))
        }
        // WS-F cluster 010 B4: without this arm, `proveDlog(gg)` appearing
        // inline at a use site falls through to `other => other.clone()`,
        // so the extracted ValDef for `groupGenerator` is never substituted
        // into the CreateProveDlog input. Empirical fixture: composition_064.
        Expr::CreateProveDlog(cpd) => {
            let new_input = replace_all(&cpd.input, target, replacement);
            ergotree_ir::mir::create_provedlog::CreateProveDlog::try_build(new_input)
                .map(Expr::CreateProveDlog)
                .unwrap_or_else(|_| Expr::CreateProveDlog(cpd.clone()))
        }
        // FuncValue: recurse into body so target → replacement substitution
        // reaches lambda-body occurrences. Mirrors Scala's mainG behaviour
        // where a sym's references in lambda bodies all become ValUse(outer_id)
        // when the sym is extracted at the surrounding scope.
        Expr::FuncValue(fv) => {
            let new_body = replace_all(fv.body(), target, replacement);
            Expr::FuncValue(FuncValue::new(fv.args().to_vec(), new_body))
        }
        // sig-15 sigmao S7 (2026-05-09): DecodePoint walker-completeness fix.
        // Previously fell into the leaf catch-all → CreateProveDlog(DecodePoint(VU(K)))
        // body's VU(K) was never substituted by inline_single_use_vals's
        // replace_all loop, leaving orphan VU(K) when ValDef(K) was inlined.
        // Empirical: Probe 1 with B0 widened gate showed sigmao orphan VU(21)
        // appearing at stage 05a. Adding this arm closes the substitution gap.
        Expr::DecodePoint(dp) => {
            let new_input = replace_all(&dp.input, target, replacement);
            Expr::DecodePoint(ergotree_ir::mir::decode_point::DecodePoint {
                input: new_input.into(),
            })
        }
        // Other single-input wrappers also missing from this match — add
        // proactively to avoid latent walker-completeness bugs of the same
        // shape. Each is byte-neutral on current sig-15 + F.2 (no test
        // currently triggers a ValUse inside one of these positions during
        // inline substitution), but the gap exists structurally.
        Expr::CalcSha256(s) => {
            let new_input = replace_all(&s.input, target, replacement);
            Expr::CalcSha256(ergotree_ir::mir::calc_sha256::CalcSha256 {
                input: new_input.into(),
            })
        }
        Expr::BitInversion(s) => {
            let new_input = replace_all(&s.input, target, replacement);
            Expr::BitInversion(ergotree_ir::mir::bit_inversion::BitInversion {
                input: new_input.into(),
            })
        }
        Expr::ByteArrayToLong(s) => {
            let new_input = replace_all(&s.expr.input, target, replacement);
            Expr::ByteArrayToLong(Spanned {
                source_span: s.source_span,
                expr: ergotree_ir::mir::byte_array_to_long::ByteArrayToLong {
                    input: new_input.into(),
                },
            })
        }
        Expr::ExtractBytesWithNoRef(s) => {
            let new_input = replace_all(&s.input, target, replacement);
            Expr::ExtractBytesWithNoRef(
                ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef {
                    input: new_input.into(),
                },
            )
        }
        Expr::LongToByteArray(s) => {
            let new_input = replace_all(&s.input, target, replacement);
            Expr::LongToByteArray(ergotree_ir::mir::long_to_byte_array::LongToByteArray {
                input: new_input.into(),
            })
        }
        Expr::SigmaPropIsProven(s) => {
            let new_input = replace_all(&s.input, target, replacement);
            Expr::SigmaPropIsProven(
                ergotree_ir::mir::sigma_prop_is_proven::SigmaPropIsProven {
                    input: new_input.into(),
                },
            )
        }
        Expr::XorOf(s) => {
            let new_input = replace_all(&s.input, target, replacement);
            Expr::XorOf(ergotree_ir::mir::xor_of::XorOf {
                input: new_input.into(),
            })
        }
        // Leaves
        other => other.clone(),
    }
}

// -----------------------------------------------------------------------
// ValId Renumbering
// -----------------------------------------------------------------------

/// Collect all ValDef IDs AND FuncArg IDs from the expression tree (including lambda scopes).
/// Used to build the renumbering map for top-level CSE.
#[allow(dead_code)]
fn collect_all_outer_def_ids(expr: &Expr, ids: &mut Vec<u32>) {
    match expr {
        Expr::ValDef(s) => {
            ids.push(s.expr.id.0);
            collect_all_outer_def_ids(&s.expr.rhs, ids);
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_all_outer_def_ids(item, ids);
            }
            collect_all_outer_def_ids(&s.expr.result, ids);
        }
        Expr::FuncValue(fv) => {
            for arg in fv.args() {
                ids.push(arg.idx.0);
            }
            // Recurse into body to find nested FuncArgs (but NOT to shift lambda-internal CSE vals)
            collect_all_outer_def_ids(fv.body(), ids);
        }
        Expr::BinOp(s) => {
            collect_all_outer_def_ids(&s.expr.left, ids);
            collect_all_outer_def_ids(&s.expr.right, ids);
        }
        Expr::BoolToSigmaProp(bts) => collect_all_outer_def_ids(&bts.input, ids),
        Expr::If(if_op) => {
            collect_all_outer_def_ids(&if_op.condition, ids);
            collect_all_outer_def_ids(&if_op.true_branch, ids);
            collect_all_outer_def_ids(&if_op.false_branch, ids);
        }
        Expr::Filter(s) => {
            collect_all_outer_def_ids(&s.expr.input, ids);
            collect_all_outer_def_ids(&s.expr.condition, ids);
        }
        Expr::Exists(s) => {
            collect_all_outer_def_ids(&s.expr.input, ids);
            collect_all_outer_def_ids(&s.expr.condition, ids);
        }
        Expr::ForAll(s) => {
            collect_all_outer_def_ids(&s.expr.input, ids);
            collect_all_outer_def_ids(&s.expr.condition, ids);
        }
        Expr::Map(s) => {
            collect_all_outer_def_ids(&s.expr.input, ids);
            collect_all_outer_def_ids(&s.expr.mapper, ids);
        }
        Expr::Fold(s) => {
            collect_all_outer_def_ids(&s.expr.input, ids);
            collect_all_outer_def_ids(&s.expr.zero, ids);
            collect_all_outer_def_ids(&s.expr.fold_op, ids);
        }
        Expr::SizeOf(so) => collect_all_outer_def_ids(&so.input, ids),
        Expr::PropertyCall(s) => collect_all_outer_def_ids(&s.expr.obj, ids),
        Expr::MethodCall(s) => {
            collect_all_outer_def_ids(&s.expr.obj, ids);
            for a in &s.expr.args {
                collect_all_outer_def_ids(a, ids);
            }
        }
        Expr::ExtractAmount(ea) => collect_all_outer_def_ids(&ea.input, ids),
        Expr::ExtractRegisterAs(s) => collect_all_outer_def_ids(&s.expr.input, ids),
        Expr::ExtractScriptBytes(esb) => collect_all_outer_def_ids(&esb.input, ids),
        Expr::SigmaAnd(sa) => {
            for i in sa.items.iter() {
                collect_all_outer_def_ids(i, ids);
            }
        }
        Expr::SigmaOr(so) => {
            for i in so.items.iter() {
                collect_all_outer_def_ids(i, ids);
            }
        }
        Expr::SelectField(s) => collect_all_outer_def_ids(&s.expr.input, ids),
        Expr::ByIndex(s) => {
            collect_all_outer_def_ids(&s.expr.input, ids);
            collect_all_outer_def_ids(&s.expr.index, ids);
        }
        Expr::OptionGet(s) => collect_all_outer_def_ids(&s.expr.input, ids),
        Expr::OptionIsDefined(s) => collect_all_outer_def_ids(&s.expr.input, ids),
        Expr::LogicalNot(s) => collect_all_outer_def_ids(&s.expr.input, ids),
        Expr::Upcast(uc) => collect_all_outer_def_ids(&uc.input, ids),
        _ => {}
    }
}

#[allow(dead_code)]
fn collect_top_level_val_ids(_expr: &Expr, _ids: &mut Vec<u32>) {}
#[allow(dead_code)]
fn collect_func_arg_ids(_expr: &Expr, _ids: &mut Vec<u32>) {}

/// Collect all ValDef IDs (not FuncArg) from the expression tree.
#[allow(dead_code)]
fn collect_existing_val_ids(expr: &Expr, ids: &mut Vec<u32>) {
    match expr {
        Expr::ValDef(s) => {
            ids.push(s.expr.id.0);
            collect_existing_val_ids(&s.expr.rhs, ids);
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_existing_val_ids(item, ids);
            }
            collect_existing_val_ids(&s.expr.result, ids);
        }
        Expr::BinOp(s) => {
            collect_existing_val_ids(&s.expr.left, ids);
            collect_existing_val_ids(&s.expr.right, ids);
        }
        Expr::BoolToSigmaProp(bts) => collect_existing_val_ids(&bts.input, ids),
        Expr::If(if_op) => {
            collect_existing_val_ids(&if_op.condition, ids);
            collect_existing_val_ids(&if_op.true_branch, ids);
            collect_existing_val_ids(&if_op.false_branch, ids);
        }
        Expr::Filter(s) => {
            collect_existing_val_ids(&s.expr.input, ids);
        }
        Expr::Exists(s) => {
            collect_existing_val_ids(&s.expr.input, ids);
        }
        Expr::ForAll(s) => {
            collect_existing_val_ids(&s.expr.input, ids);
        }
        Expr::Map(s) => {
            collect_existing_val_ids(&s.expr.input, ids);
        }
        Expr::Fold(s) => {
            collect_existing_val_ids(&s.expr.input, ids);
            collect_existing_val_ids(&s.expr.zero, ids);
        }
        Expr::SizeOf(so) => collect_existing_val_ids(&so.input, ids),
        Expr::PropertyCall(s) => collect_existing_val_ids(&s.expr.obj, ids),
        Expr::MethodCall(s) => {
            collect_existing_val_ids(&s.expr.obj, ids);
            for a in &s.expr.args {
                collect_existing_val_ids(a, ids);
            }
        }
        Expr::ExtractAmount(ea) => collect_existing_val_ids(&ea.input, ids),
        Expr::SigmaAnd(sa) => {
            for i in sa.items.iter() {
                collect_existing_val_ids(i, ids);
            }
        }
        Expr::SigmaOr(so) => {
            for i in so.items.iter() {
                collect_existing_val_ids(i, ids);
            }
        }
        // Don't recurse into FuncValue bodies — lambda IDs are independent
        _ => {}
    }
}

use std::collections::{HashMap, HashSet};

/// Renumber all ValIds sequentially from 1, in definition order.
/// This matches the Scala compiler's post-optimization ID assignment.
#[allow(dead_code)]
fn renumber_val_ids(expr: Expr) -> Expr {
    let mut id_map: HashMap<u32, u32> = HashMap::new();
    let mut next_id: u32 = 1;
    // First pass: collect all definition sites in order and assign new IDs
    collect_def_ids(&expr, &mut id_map, &mut next_id);
    // Second pass: rewrite all IDs using the map
    rewrite_ids(expr, &id_map)
}

/// Collect all ValDef and FuncArg IDs in definition order, assigning sequential new IDs.
#[allow(dead_code)]
fn collect_def_ids(expr: &Expr, id_map: &mut HashMap<u32, u32>, next_id: &mut u32) {
    match expr {
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                collect_def_ids(item, id_map, next_id);
            }
            collect_def_ids(&s.expr.result, id_map, next_id);
        }
        Expr::ValDef(s) => {
            let old_id = s.expr.id.0;
            #[allow(clippy::map_entry)]
            if !id_map.contains_key(&old_id) {
                id_map.insert(old_id, *next_id);
                *next_id += 1;
            }
            collect_def_ids(&s.expr.rhs, id_map, next_id);
        }
        Expr::FuncValue(fv) => {
            for arg in fv.args() {
                let old_id = arg.idx.0;
                #[allow(clippy::map_entry)]
                if !id_map.contains_key(&old_id) {
                    id_map.insert(old_id, *next_id);
                    *next_id += 1;
                }
            }
            collect_def_ids(fv.body(), id_map, next_id);
        }
        Expr::BinOp(s) => {
            collect_def_ids(&s.expr.left, id_map, next_id);
            collect_def_ids(&s.expr.right, id_map, next_id);
        }
        Expr::BoolToSigmaProp(bts) => collect_def_ids(&bts.input, id_map, next_id),
        Expr::If(if_op) => {
            collect_def_ids(&if_op.condition, id_map, next_id);
            collect_def_ids(&if_op.true_branch, id_map, next_id);
            collect_def_ids(&if_op.false_branch, id_map, next_id);
        }
        Expr::Filter(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.condition, id_map, next_id);
        }
        Expr::Exists(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.condition, id_map, next_id);
        }
        Expr::ForAll(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.condition, id_map, next_id);
        }
        Expr::Map(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.mapper, id_map, next_id);
        }
        Expr::Fold(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.zero, id_map, next_id);
            collect_def_ids(&s.expr.fold_op, id_map, next_id);
        }
        Expr::PropertyCall(s) => collect_def_ids(&s.expr.obj, id_map, next_id),
        Expr::MethodCall(s) => {
            collect_def_ids(&s.expr.obj, id_map, next_id);
            for arg in &s.expr.args {
                collect_def_ids(arg, id_map, next_id);
            }
        }
        Expr::ExtractAmount(ea) => collect_def_ids(&ea.input, id_map, next_id),
        Expr::ExtractRegisterAs(s) => collect_def_ids(&s.expr.input, id_map, next_id),
        Expr::ExtractScriptBytes(esb) => collect_def_ids(&esb.input, id_map, next_id),
        Expr::ExtractBytes(eb) => collect_def_ids(&eb.input, id_map, next_id),
        Expr::ExtractId(ei) => collect_def_ids(&ei.input, id_map, next_id),
        Expr::ExtractCreationInfo(eci) => collect_def_ids(&eci.input, id_map, next_id),
        Expr::SizeOf(so) => collect_def_ids(&so.input, id_map, next_id),
        Expr::ByIndex(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.index, id_map, next_id);
            if let Some(ref d) = s.expr.default {
                collect_def_ids(d, id_map, next_id);
            }
        }
        Expr::SelectField(s) => collect_def_ids(&s.expr.input, id_map, next_id),
        Expr::OptionGet(s) => collect_def_ids(&s.expr.input, id_map, next_id),
        Expr::OptionIsDefined(s) => collect_def_ids(&s.expr.input, id_map, next_id),
        Expr::OptionGetOrElse(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.default, id_map, next_id);
        }
        Expr::Slice(s) => {
            collect_def_ids(&s.expr.input, id_map, next_id);
            collect_def_ids(&s.expr.from, id_map, next_id);
            collect_def_ids(&s.expr.until, id_map, next_id);
        }
        Expr::LogicalNot(s) => collect_def_ids(&s.expr.input, id_map, next_id),
        Expr::Negation(s) => collect_def_ids(&s.expr.input, id_map, next_id),
        Expr::SigmaPropBytes(spb) => collect_def_ids(&spb.input, id_map, next_id),
        Expr::Upcast(uc) => collect_def_ids(&uc.input, id_map, next_id),
        Expr::Downcast(dc) => collect_def_ids(&dc.input, id_map, next_id),
        Expr::CalcBlake2b256(cb) => collect_def_ids(&cb.input, id_map, next_id),
        Expr::SigmaAnd(sa) => {
            for item in sa.items.iter() {
                collect_def_ids(item, id_map, next_id);
            }
        }
        Expr::SigmaOr(so) => {
            for item in so.items.iter() {
                collect_def_ids(item, id_map, next_id);
            }
        }
        Expr::Tuple(t) => {
            for item in t.items.iter() {
                collect_def_ids(item, id_map, next_id);
            }
        }
        Expr::TreeLookup(s) => {
            collect_def_ids(&s.expr.tree, id_map, next_id);
            collect_def_ids(&s.expr.key, id_map, next_id);
            collect_def_ids(&s.expr.proof, id_map, next_id);
        }
        Expr::Apply(app) => {
            collect_def_ids(&app.func, id_map, next_id);
            for arg in &app.args {
                collect_def_ids(arg, id_map, next_id);
            }
        }
        _ => {}
    }
}

/// Rewrite all ValDef IDs, ValUse IDs, and FuncArg IDs using the renumbering map.
#[allow(dead_code)]
fn rewrite_ids(expr: Expr, id_map: &HashMap<u32, u32>) -> Expr {
    match expr {
        Expr::ValDef(s) => {
            let new_id = id_map.get(&s.expr.id.0).copied().unwrap_or(s.expr.id.0);
            let new_rhs = rewrite_ids(*s.expr.rhs, id_map);
            Expr::ValDef(Spanned {
                source_span: s.source_span,
                expr: ValDef {
                    id: ValId(new_id),
                    rhs: new_rhs.into(),
                },
            })
        }
        Expr::ValUse(vu) => {
            let new_id = id_map.get(&vu.val_id.0).copied().unwrap_or(vu.val_id.0);
            Expr::ValUse(ValUse {
                val_id: ValId(new_id),
                tpe: vu.tpe,
            })
        }
        Expr::FuncValue(fv) => {
            let new_args: Vec<ergotree_ir::mir::func_value::FuncArg> = fv
                .args()
                .iter()
                .map(|a| {
                    let new_id = id_map.get(&a.idx.0).copied().unwrap_or(a.idx.0);
                    ergotree_ir::mir::func_value::FuncArg {
                        idx: ValId(new_id),
                        tpe: a.tpe.clone(),
                    }
                })
                .collect();
            let new_body = rewrite_ids(fv.body().clone(), id_map);
            Expr::FuncValue(FuncValue::new(new_args, new_body))
        }
        Expr::BlockValue(s) => Expr::BlockValue(Spanned {
            source_span: s.source_span,
            expr: BlockValue {
                items: s
                    .expr
                    .items
                    .into_iter()
                    .map(|i| rewrite_ids(i, id_map))
                    .collect(),
                result: rewrite_ids(*s.expr.result, id_map).into(),
            },
        }),
        Expr::BinOp(s) => Expr::BinOp(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::bin_op::BinOp {
                kind: s.expr.kind,
                left: rewrite_ids(*s.expr.left, id_map).into(),
                right: rewrite_ids(*s.expr.right, id_map).into(),
            },
        }),
        Expr::BoolToSigmaProp(bts) => {
            Expr::BoolToSigmaProp(ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                input: rewrite_ids(*bts.input, id_map).into(),
            })
        }
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: rewrite_ids(*if_op.condition, id_map).into(),
            true_branch: rewrite_ids(*if_op.true_branch, id_map).into(),
            false_branch: rewrite_ids(*if_op.false_branch, id_map).into(),
        }),
        Expr::Filter(s) => Expr::Filter(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_filter::Filter {
                input: rewrite_ids(*s.expr.input, id_map).into(),
                condition: rewrite_ids(*s.expr.condition, id_map).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::Exists(s) => Expr::Exists(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_exists::Exists {
                input: rewrite_ids(*s.expr.input, id_map).into(),
                condition: rewrite_ids(*s.expr.condition, id_map).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::ForAll(s) => Expr::ForAll(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_forall::ForAll {
                input: rewrite_ids(*s.expr.input, id_map).into(),
                condition: rewrite_ids(*s.expr.condition, id_map).into(),
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::Map(s) => {
            let input = rewrite_ids(*s.expr.input, id_map);
            let mapper = rewrite_ids(*s.expr.mapper, id_map);
            ergotree_ir::mir::coll_map::Map::new(input, mapper)
                .map(|m| {
                    Expr::Map(Spanned {
                        source_span: SourceSpan::empty(),
                        expr: m,
                    })
                })
                .expect("Map::new in rewrite_ids")
        }
        Expr::Fold(s) => {
            let input = rewrite_ids(*s.expr.input, id_map);
            let zero = rewrite_ids(*s.expr.zero, id_map);
            let fold_op = rewrite_ids(*s.expr.fold_op, id_map);
            ergotree_ir::mir::coll_fold::Fold::new(input, zero, fold_op)
                .map(|f| {
                    Expr::Fold(Spanned {
                        source_span: SourceSpan::empty(),
                        expr: f,
                    })
                })
                .expect("Fold::new in rewrite_ids")
        }
        Expr::PropertyCall(s) => Expr::PropertyCall(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::property_call::PropertyCall {
                obj: rewrite_ids(*s.expr.obj, id_map).into(),
                method: s.expr.method,
            },
        }),
        Expr::MethodCall(s) => {
            let obj = rewrite_ids(*s.expr.obj, id_map);
            let args: Vec<Expr> = s
                .expr
                .args
                .into_iter()
                .map(|a| rewrite_ids(a, id_map))
                .collect();
            ergotree_ir::mir::method_call::MethodCall::with_type_args(
                obj,
                s.expr.method,
                args,
                s.expr.explicit_type_args,
            )
            .map(|mc| {
                Expr::MethodCall(Spanned {
                    source_span: s.source_span,
                    expr: mc,
                })
            })
            .expect("MethodCall::new in rewrite_ids")
        }
        Expr::ExtractAmount(ea) => {
            Expr::ExtractAmount(ergotree_ir::mir::extract_amount::ExtractAmount {
                input: rewrite_ids(*ea.input, id_map).into(),
            })
        }
        Expr::ExtractRegisterAs(s) => Expr::ExtractRegisterAs(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::extract_reg_as::ExtractRegisterAs {
                input: rewrite_ids(*s.expr.input, id_map).into(),
                register_id: s.expr.register_id,
                elem_tpe: s.expr.elem_tpe,
            },
        }),
        Expr::ExtractScriptBytes(esb) => {
            Expr::ExtractScriptBytes(ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes {
                input: rewrite_ids(*esb.input, id_map).into(),
            })
        }
        Expr::ExtractBytes(eb) => {
            Expr::ExtractBytes(ergotree_ir::mir::extract_bytes::ExtractBytes {
                input: rewrite_ids(*eb.input, id_map).into(),
            })
        }
        Expr::ExtractId(ei) => Expr::ExtractId(ergotree_ir::mir::extract_id::ExtractId {
            input: rewrite_ids(*ei.input, id_map).into(),
        }),
        Expr::ExtractCreationInfo(eci) => Expr::ExtractCreationInfo(
            ergotree_ir::mir::extract_creation_info::ExtractCreationInfo {
                input: rewrite_ids(*eci.input, id_map).into(),
            },
        ),
        Expr::SizeOf(so) => Expr::SizeOf(ergotree_ir::mir::coll_size::SizeOf {
            input: rewrite_ids(*so.input, id_map).into(),
        }),
        Expr::ByIndex(s) => {
            let input = rewrite_ids(*s.expr.input, id_map);
            let index = rewrite_ids(*s.expr.index, id_map);
            let default = s.expr.default.map(|d| Box::new(rewrite_ids(*d, id_map)));
            ergotree_ir::mir::coll_by_index::ByIndex::new(input, index, default)
                .map(|bi| {
                    Expr::ByIndex(Spanned {
                        source_span: s.source_span,
                        expr: bi,
                    })
                })
                .expect("ByIndex::new in rewrite_ids")
        }
        Expr::SelectField(s) => {
            let input = rewrite_ids(*s.expr.input, id_map);
            ergotree_ir::mir::select_field::SelectField::new(input, s.expr.field_index)
                .map(|sf| {
                    Expr::SelectField(Spanned {
                        source_span: s.source_span,
                        expr: sf,
                    })
                })
                .expect("SelectField::new in rewrite_ids")
        }
        Expr::OptionGet(s) => {
            let input = rewrite_ids(*s.expr.input, id_map);
            ergotree_ir::mir::option_get::OptionGet::try_build(input)
                .map(|og| {
                    Expr::OptionGet(Spanned {
                        source_span: s.source_span,
                        expr: og,
                    })
                })
                .expect("OptionGet in rewrite_ids")
        }
        Expr::OptionIsDefined(s) => {
            let input = rewrite_ids(*s.expr.input, id_map);
            ergotree_ir::mir::option_is_defined::OptionIsDefined::try_build(input)
                .map(|oid| {
                    Expr::OptionIsDefined(Spanned {
                        source_span: s.source_span,
                        expr: oid,
                    })
                })
                .expect("OptionIsDefined in rewrite_ids")
        }
        Expr::OptionGetOrElse(s) => {
            let input = rewrite_ids(*s.expr.input, id_map);
            let default = rewrite_ids(*s.expr.default, id_map);
            ergotree_ir::mir::option_get_or_else::OptionGetOrElse::new(input, default)
                .map(|oge| {
                    Expr::OptionGetOrElse(Spanned {
                        source_span: s.source_span,
                        expr: oge,
                    })
                })
                .expect("OptionGetOrElse in rewrite_ids")
        }
        Expr::Slice(s) => Expr::Slice(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::coll_slice::Slice {
                input: rewrite_ids(*s.expr.input, id_map).into(),
                from: rewrite_ids(*s.expr.from, id_map).into(),
                until: rewrite_ids(*s.expr.until, id_map).into(),
            },
        }),
        Expr::LogicalNot(s) => Expr::LogicalNot(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::logical_not::LogicalNot {
                input: rewrite_ids(*s.expr.input, id_map).into(),
            },
        }),
        Expr::Negation(s) => Expr::Negation(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::negation::Negation {
                input: rewrite_ids(*s.expr.input, id_map).into(),
            },
        }),
        Expr::SigmaPropBytes(spb) => {
            Expr::SigmaPropBytes(ergotree_ir::mir::sigma_prop_bytes::SigmaPropBytes {
                input: rewrite_ids(*spb.input, id_map).into(),
            })
        }
        Expr::SigmaPropIsProven(sip) => {
            Expr::SigmaPropIsProven(ergotree_ir::mir::sigma_prop_is_proven::SigmaPropIsProven {
                input: rewrite_ids(*sip.input, id_map).into(),
            })
        }
        Expr::ZkProofBlock(zk) => Expr::ZkProofBlock(ergotree_ir::mir::zk_proof::ZkProofBlock {
            input: rewrite_ids(*zk.input, id_map).into(),
        }),
        Expr::Upcast(uc) => Expr::Upcast(ergotree_ir::mir::upcast::Upcast {
            input: rewrite_ids(*uc.input, id_map).into(),
            tpe: uc.tpe,
        }),
        Expr::CalcBlake2b256(cb) => {
            Expr::CalcBlake2b256(ergotree_ir::mir::calc_blake2b256::CalcBlake2b256 {
                input: rewrite_ids(*cb.input, id_map).into(),
            })
        }
        Expr::CreateProveDlog(cpd) => {
            let input = rewrite_ids(*cpd.input, id_map);
            ergotree_ir::mir::create_provedlog::CreateProveDlog::try_build(input)
                .map(Expr::CreateProveDlog)
                .expect("CreateProveDlog::try_build in rewrite_ids")
        }
        Expr::SigmaAnd(sa) => {
            let items: Vec<Expr> = sa
                .items
                .into_iter()
                .map(|i| rewrite_ids(i, id_map))
                .collect();
            Expr::SigmaAnd(ergotree_ir::mir::sigma_and::SigmaAnd {
                items: items.try_into().expect("SigmaAnd >= 2"),
            })
        }
        Expr::SigmaOr(so) => {
            let items: Vec<Expr> = so
                .items
                .into_iter()
                .map(|i| rewrite_ids(i, id_map))
                .collect();
            Expr::SigmaOr(ergotree_ir::mir::sigma_or::SigmaOr {
                items: items.try_into().expect("SigmaOr >= 2"),
            })
        }
        Expr::Tuple(t) => {
            let items: Vec<Expr> = t
                .items
                .into_iter()
                .map(|i| rewrite_ids(i, id_map))
                .collect();
            Expr::Tuple(ergotree_ir::mir::tuple::Tuple::new(items).expect("valid tuple"))
        }
        Expr::TreeLookup(s) => Expr::TreeLookup(Spanned {
            source_span: s.source_span,
            expr: ergotree_ir::mir::tree_lookup::TreeLookup {
                tree: rewrite_ids(*s.expr.tree, id_map).into(),
                key: rewrite_ids(*s.expr.key, id_map).into(),
                proof: rewrite_ids(*s.expr.proof, id_map).into(),
            },
        }),
        Expr::Apply(app) => {
            let func = rewrite_ids(*app.func, id_map);
            let args: Vec<Expr> = app
                .args
                .into_iter()
                .map(|a| rewrite_ids(a, id_map))
                .collect();
            ergotree_ir::mir::apply::Apply::new(func, args)
                .map(Expr::Apply)
                .expect("Apply::new in rewrite_ids")
        }
        Expr::Atleast(s) => ergotree_ir::mir::atleast::Atleast::new(
            rewrite_ids(*s.bound, id_map),
            rewrite_ids(*s.input, id_map),
        )
        .map(Expr::Atleast)
        .expect("Atleast::new in rewrite_ids"),
        Expr::And(a) => Expr::And(ergotree_ir::source_span::Spanned {
            source_span: a.source_span,
            expr: ergotree_ir::mir::and::And {
                input: rewrite_ids(*a.expr.input, id_map).into(),
            },
        }),
        Expr::Or(o) => Expr::Or(ergotree_ir::source_span::Spanned {
            source_span: o.source_span,
            expr: ergotree_ir::mir::or::Or {
                input: rewrite_ids(*o.expr.input, id_map).into(),
            },
        }),
        Expr::Collection(c) => match c {
            ergotree_ir::mir::collection::Collection::Exprs { elem_tpe, items } => {
                let new_items: Vec<Expr> =
                    items.into_iter().map(|i| rewrite_ids(i, id_map)).collect();
                Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs {
                    elem_tpe,
                    items: new_items,
                })
            }
            other => Expr::Collection(other),
        },
        // Single-input wrappers (OneArgOpTryBuild, non-Spanned)
        Expr::LongToByteArray(s) => {
            ergotree_ir::mir::long_to_byte_array::LongToByteArray::try_build(rewrite_ids(
                *s.input, id_map,
            ))
            .map(Expr::LongToByteArray)
            .expect("LongToByteArray in rewrite_ids")
        }
        Expr::DecodePoint(s) => {
            ergotree_ir::mir::decode_point::DecodePoint::try_build(rewrite_ids(*s.input, id_map))
                .map(Expr::DecodePoint)
                .expect("DecodePoint in rewrite_ids")
        }
        Expr::ExtractBytesWithNoRef(s) => {
            ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef::try_build(
                rewrite_ids(*s.input, id_map),
            )
            .map(Expr::ExtractBytesWithNoRef)
            .expect("ExtractBytesWithNoRef in rewrite_ids")
        }
        Expr::CalcSha256(s) => {
            ergotree_ir::mir::calc_sha256::CalcSha256::try_build(rewrite_ids(*s.input, id_map))
                .map(Expr::CalcSha256)
                .expect("CalcSha256 in rewrite_ids")
        }
        Expr::BitInversion(s) => {
            ergotree_ir::mir::bit_inversion::BitInversion::try_build(rewrite_ids(*s.input, id_map))
                .map(Expr::BitInversion)
                .expect("BitInversion in rewrite_ids")
        }
        // XorOf — direct struct literal (no constructor)
        Expr::XorOf(s) => Expr::XorOf(ergotree_ir::mir::xor_of::XorOf {
            input: rewrite_ids(*s.input, id_map).into(),
        }),
        // Single-input wrappers (OneArgOpTryBuild, Spanned)
        Expr::ByteArrayToLong(s) => {
            ergotree_ir::mir::byte_array_to_long::ByteArrayToLong::try_build(rewrite_ids(
                *s.expr.input,
                id_map,
            ))
            .map(|v| {
                Expr::ByteArrayToLong(Spanned {
                    source_span: s.source_span,
                    expr: v,
                })
            })
            .expect("ByteArrayToLong in rewrite_ids")
        }
        Expr::ByteArrayToBigInt(s) => {
            ergotree_ir::mir::byte_array_to_bigint::ByteArrayToBigInt::try_build(rewrite_ids(
                *s.expr.input,
                id_map,
            ))
            .map(|v| {
                Expr::ByteArrayToBigInt(Spanned {
                    source_span: s.source_span,
                    expr: v,
                })
            })
            .expect("ByteArrayToBigInt in rewrite_ids")
        }
        // Multi-child nodes (Spanned, with constructor)
        Expr::Append(s) => ergotree_ir::mir::coll_append::Append::new(
            rewrite_ids(*s.expr.input, id_map),
            rewrite_ids(*s.expr.col_2, id_map),
        )
        .map(|v| {
            Expr::Append(Spanned {
                source_span: s.source_span,
                expr: v,
            })
        })
        .expect("Append in rewrite_ids"),
        Expr::SubstConstants(s) => ergotree_ir::mir::subst_const::SubstConstants::new(
            rewrite_ids(*s.expr.script_bytes, id_map),
            rewrite_ids(*s.expr.positions, id_map),
            rewrite_ids(*s.expr.new_values, id_map),
        )
        .map(|v| {
            Expr::SubstConstants(Spanned {
                source_span: s.source_span,
                expr: v,
            })
        })
        .expect("SubstConstants in rewrite_ids"),
        // Multi-child nodes (non-Spanned, with constructor)
        Expr::Xor(s) => ergotree_ir::mir::xor::Xor::new(
            rewrite_ids(*s.left, id_map),
            rewrite_ids(*s.right, id_map),
        )
        .map(Expr::Xor)
        .expect("Xor in rewrite_ids"),
        Expr::CreateProveDhTuple(s) => {
            ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple::new(
                rewrite_ids(*s.g, id_map),
                rewrite_ids(*s.h, id_map),
                rewrite_ids(*s.u, id_map),
                rewrite_ids(*s.v, id_map),
            )
            .map(Expr::CreateProveDhTuple)
            .expect("CreateProveDhTuple in rewrite_ids")
        }
        Expr::CreateAvlTree(s) => ergotree_ir::mir::create_avl_tree::CreateAvlTree::new(
            rewrite_ids(*s.flags, id_map),
            rewrite_ids(*s.digest, id_map),
            rewrite_ids(*s.key_length, id_map),
            s.value_length.map(|vl| Box::new(rewrite_ids(*vl, id_map))),
        )
        .map(Expr::CreateAvlTree)
        .expect("CreateAvlTree in rewrite_ids"),
        Expr::MultiplyGroup(s) => ergotree_ir::mir::multiply_group::MultiplyGroup::new(
            rewrite_ids(*s.left, id_map),
            rewrite_ids(*s.right, id_map),
        )
        .map(Expr::MultiplyGroup)
        .expect("MultiplyGroup in rewrite_ids"),
        Expr::Exponentiate(s) => ergotree_ir::mir::exponentiate::Exponentiate::new(
            rewrite_ids(*s.left, id_map),
            rewrite_ids(*s.right, id_map),
        )
        .map(Expr::Exponentiate)
        .expect("Exponentiate in rewrite_ids"),
        // Downcast — same pattern as Upcast
        Expr::Downcast(dc) => Expr::Downcast(ergotree_ir::mir::downcast::Downcast {
            input: rewrite_ids(*dc.input, id_map).into(),
            tpe: dc.tpe,
        }),
        // Optional child only (direct struct literal)
        Expr::DeserializeRegister(s) => {
            let new_default = s.default.map(|d| Box::new(rewrite_ids(*d, id_map)));
            Expr::DeserializeRegister(
                ergotree_ir::mir::deserialize_register::DeserializeRegister {
                    reg: s.reg,
                    tpe: s.tpe,
                    default: new_default,
                },
            )
        }
        // True leaves — no child Expr fields, pass through unchanged
        Expr::Const(_)
        | Expr::ConstPlaceholder(_)
        | Expr::GlobalVars(_)
        | Expr::Context
        | Expr::Global
        | Expr::GetVar(_)
        | Expr::DeserializeContext(_) => expr,
    }
}

#[cfg(test)]
mod renumber_scope_safety {
    //! Repro tests for the S33 latent bug: `sequential_renumber` can produce
    //! a `BlockValue` whose `items` list contains two `ValDef`s with the
    //! same `ValId`. Triggered when an old_id appears in a sibling If
    //! branch *and* in this scope's items: the second occurrence is
    //! treated as "already mapped" and does not advance `next_id`, so the
    //! NEXT sibling ValDef steals the same new_id.
    //!
    //! These tests exercise `sequential_renumber` (and its prerequisite
    //! `flatten_nested_blocks`) directly on hand-built MIR, no parser
    //! involvement. They are intended to FAIL on master until the
    //! scope-merge bug is fixed.
    use super::*;
    use ergotree_ir::mir::block::BlockValue;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::if_op::If;
    use ergotree_ir::mir::val_def::{ValDef, ValId};
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::source_span::{SourceSpan, Spanned};
    use ergotree_ir::types::stype::SType;
    use std::collections::HashSet;

    fn c(v: i64) -> Expr {
        Expr::Const(Constant::from(v))
    }
    fn cb(v: bool) -> Expr {
        Expr::Const(Constant::from(v))
    }
    fn vdef(id: u32, rhs: Expr) -> Expr {
        Expr::ValDef(Spanned {
            source_span: SourceSpan::empty(),
            expr: ValDef {
                id: ValId(id),
                rhs: Box::new(rhs),
            },
        })
    }
    fn vuse(id: u32) -> Expr {
        Expr::ValUse(ValUse {
            val_id: ValId(id),
            tpe: SType::SLong,
        })
    }
    fn block(items: Vec<Expr>, result: Expr) -> Expr {
        Expr::BlockValue(Spanned {
            source_span: SourceSpan::empty(),
            expr: BlockValue {
                items,
                result: Box::new(result),
            },
        })
    }
    fn if_e(cond: Expr, t: Expr, f: Expr) -> Expr {
        Expr::If(If {
            condition: Box::new(cond),
            true_branch: Box::new(t),
            false_branch: Box::new(f),
        })
    }

    /// Walk the tree; for every BlockValue, return any duplicate ValIds
    /// among its direct `items` (each scope checked independently).
    fn find_same_scope_dup_ids(expr: &Expr) -> Vec<(String, Vec<u32>)> {
        let mut out = Vec::new();
        walk(expr, "root", &mut out);
        return out;

        fn walk(e: &Expr, path: &str, out: &mut Vec<(String, Vec<u32>)>) {
            if let Expr::BlockValue(s) = e {
                let mut seen: HashSet<u32> = HashSet::new();
                let mut dups: Vec<u32> = Vec::new();
                for item in &s.expr.items {
                    if let Expr::ValDef(vd) = item {
                        let id = vd.expr.id.0;
                        if !seen.insert(id) && !dups.contains(&id) {
                            dups.push(id);
                        }
                    }
                }
                if !dups.is_empty() {
                    out.push((path.to_string(), dups));
                }
            }
            // Recurse into children
            for (i, child) in expr_children(e).into_iter().enumerate() {
                let p = format!("{}/{}", path, i);
                walk(&child, &p, out);
            }
        }
    }

    fn expr_children(e: &Expr) -> Vec<Expr> {
        match e {
            Expr::BlockValue(s) => {
                let mut v: Vec<Expr> = s.expr.items.clone();
                v.push((*s.expr.result).clone());
                v
            }
            Expr::ValDef(s) => vec![(*s.expr.rhs).clone()],
            Expr::If(i) => vec![
                (*i.condition).clone(),
                (*i.true_branch).clone(),
                (*i.false_branch).clone(),
            ],
            _ => Vec::new(),
        }
    }

    /// Minimal repro of the S33 bug.
    ///
    /// Pre-renumber tree:
    /// ```
    /// BlockValue {
    ///   items: [ValDef(id=1, Const(0))]
    ///   result: If(true,
    ///     then: BlockValue { items: [ValDef(id=2, Const(10))], result: ValUse(2) },
    ///     else: BlockValue {
    ///       items: [
    ///         ValDef(id=2, Const(20)),   // same old_id as then-branch's V_x
    ///         ValDef(id=3, Const(30)),
    ///       ],
    ///       result: ValUse(3)
    ///     }
    ///   )
    /// }
    /// ```
    ///
    /// Trace through `sequential_renumber`:
    /// - Outer ValDef(1): my_def_id=0, new_id=1; id_map={1→1}, next_id=1
    /// - If: branch_start=1
    ///   - then: ValDef(2): my_def_id=1, new_id=2; id_map={1→1,2→2}, next_id=2
    ///   - reset next_id=1
    ///   - else:
    ///     - ValDef(2): id_map ALREADY has 2 → skip assignment, just recurse RHS.
    ///       next_id stays at 1. (BUG: this ValDef's id is now also 2, but
    ///       next_id was not advanced.)
    ///     - ValDef(3): my_def_id=1, new_id=2; id_map={1→1,2→2,3→2}, next_id=2
    ///       *** Both else-branch items now bear new_id=2 — SAME SCOPE DUP. ***
    ///
    /// Documents the latent bug: `sequential_renumber` *alone* still
    /// produces same-scope duplicate ValIds on this input. This is the
    /// historical S33 failure mode. The fix lives in
    /// `disambiguate_val_ids`, which is run by `apply_cse` before
    /// `sequential_renumber`. Asserts the bug still reproduces if
    /// disambig is skipped — guards against silent re-introduction.
    #[test]
    fn raw_sequential_renumber_still_buggy_without_disambig() {
        let pre = block(
            vec![vdef(1, c(0))],
            if_e(
                cb(true),
                block(vec![vdef(2, c(10))], vuse(2)),
                block(vec![vdef(2, c(20)), vdef(3, c(30))], vuse(3)),
            ),
        );

        let post = sequential_renumber(pre);
        let dups = find_same_scope_dup_ids(&post);
        assert!(
            !dups.is_empty(),
            "expected raw sequential_renumber to still exhibit the bug \
             (else disambiguate_val_ids no longer needed): {:#?}",
            post
        );
    }

    /// The actual fix: `disambiguate_val_ids` followed by
    /// `sequential_renumber` produces no same-scope duplicates on the
    /// shape that previously broke ProxyBorrow.
    #[test]
    fn disambig_then_renumber_resolves_collision() {
        let pre = block(
            vec![vdef(1, c(0))],
            if_e(
                cb(true),
                block(vec![vdef(2, c(10))], vuse(2)),
                block(vec![vdef(2, c(20)), vdef(3, c(30))], vuse(3)),
            ),
        );

        let disambiguated = disambiguate_val_ids(pre);
        // After disambig: every ValDef has a globally-unique id.
        let mut seen: HashSet<u32> = HashSet::new();
        let mut all_unique = true;
        collect_all_valdef_ids(&disambiguated, &mut |id| {
            if !seen.insert(id) {
                all_unique = false;
            }
        });
        assert!(
            all_unique,
            "disambig_val_ids did not produce globally-unique ValDef ids:\n{:#?}",
            disambiguated
        );

        let post = sequential_renumber(disambiguated);
        let dups = find_same_scope_dup_ids(&post);
        assert!(
            dups.is_empty(),
            "fix regressed: same-scope dup after disambig+renumber: {:?}\n{:#?}",
            dups,
            post
        );
    }

    fn collect_all_valdef_ids(expr: &Expr, f: &mut dyn FnMut(u32)) {
        if let Expr::ValDef(s) = expr {
            f(s.expr.id.0);
        }
        for child in expr_children(expr) {
            collect_all_valdef_ids(&child, f);
        }
    }

    /// Sanity check: when the same old_id appears in BOTH If branches but
    /// no other items are added, no scope sees a dup (each branch has at
    /// most one ValDef of that id). This should pass on master and serves
    /// as a regression baseline.
    #[test]
    fn sequential_renumber_same_id_in_both_branches_no_dup() {
        let pre = if_e(
            cb(true),
            block(vec![vdef(2, c(10))], vuse(2)),
            block(vec![vdef(2, c(20))], vuse(2)),
        );
        let post = sequential_renumber(pre);
        let dups = find_same_scope_dup_ids(&post);
        assert!(dups.is_empty(), "unexpected dup: {:?}", dups);
    }

    /// Variant: collision when one branch's old_id matches an OUTER
    /// scope's item-ValDef old_id. After renumber, the inner branch
    /// item's new_id collides with the outer item's; if any later
    /// inner-scope item picks the same `next_id` snapshot, we get a dup
    /// inside the inner scope.
    #[test]
    fn sequential_renumber_collides_with_outer_scope_id() {
        // Outer items use ids 7, 8. The inner else-branch reuses old_id=7
        // (which would be plausible if dfs_reassign_val_ids didn't
        // descend into branches), then has another fresh id.
        let pre = block(
            vec![vdef(7, c(0)), vdef(8, c(1))],
            if_e(
                cb(true),
                vuse(7),
                block(vec![vdef(7, c(20)), vdef(9, c(30))], vuse(9)),
            ),
        );
        let post = sequential_renumber(pre);
        let dups = find_same_scope_dup_ids(&post);
        assert!(
            dups.is_empty(),
            "same-scope dup detected: {:?}\ntree:\n{:#?}",
            dups,
            post
        );
    }

    /// Run `flatten_nested_blocks` then `sequential_renumber` on a tree
    /// where flatten merges a nested BlockValue into the outer scope.
    /// Confirm that flatten itself doesn't introduce dups, and renumber
    /// after flatten doesn't either.
    #[test]
    fn flatten_then_renumber_no_dup() {
        // BlockValue { items: [ValDef(1, c(0))],
        //              result: BlockValue { items: [ValDef(2, c(1))], result: ValUse(2) } }
        // After flatten: BlockValue { items: [ValDef(1, c(0)), ValDef(2, c(1))], result: ValUse(2) }
        let pre = block(vec![vdef(1, c(0))], block(vec![vdef(2, c(1))], vuse(2)));
        let flat = flatten_nested_blocks(pre);
        let dups = find_same_scope_dup_ids(&flat);
        assert!(dups.is_empty(), "flatten dup: {:?}", dups);
        let post = sequential_renumber(flat);
        let dups = find_same_scope_dup_ids(&post);
        assert!(dups.is_empty(), "renumber dup: {:?}", dups);
    }
}

// =============================================================================
// WS-G.2.2 — hash-cons primitive (standalone)
// =============================================================================
//
// Standalone hash-cons data primitive mirroring Scala's
// `sigma.compiler.ir.primitives.Thunks.ThunkScope.findDef` chain
// (current-scope bodyDefs → parent.findDef → root._globalDefs). See
// `parity-handoffs/G2.2-HASH-CONS-PRIMITIVE-HANDOFF.md` for the design probe
// and the Scala source it replicates.
//
// G.2.2 is layer 0: data primitive + unit tests only. No CSE integration —
// `process_ast_graph_impl`, `apply_cse_within_branches`, and the renumbering
// pipeline are not touched. That's G.2.3.
#[allow(dead_code)] // G.2.3 will wire these in.
mod sym_table {
    use super::direct_children;
    use ergotree_ir::mir::bin_op::BinOpKind;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::serialization::SigmaSerializable;
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    /// Scope is identified by its index in `SymTable.scope_parents`.
    /// Scope 0 is the root (no parent).
    pub(super) type ScopeId = usize;

    /// ValDef ID assigned by the hash-cons pass.
    /// The renumbering pipeline (`sequential_renumber`) will reassign final
    /// IDs later; these are stable within one hash-cons pass only.
    pub(super) type SymId = u32;

    /// Newtype wrapping `Expr` so it can be used as a `HashMap` key with
    /// structural equality. Equality is `Expr: PartialEq` (consumers MUST
    /// strip `SourceSpan`s on the wrapped Expr before insertion — see
    /// `strip_source_spans` in the parent module — because `Spanned<T>`'s
    /// derived `PartialEq` includes the span).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub(super) struct ExprKey(pub Expr);

    impl Hash for ExprKey {
        fn hash<H: Hasher>(&self, state: &mut H) {
            expr_hash(&self.0, state);
        }
    }

    /// Structural, order-sensitive hash for `Expr` consistent with
    /// `Expr: PartialEq` on span-stripped inputs. Recurses via
    /// `direct_children` and hashes per-arm scalar identity for leaves and
    /// nodes whose discriminant alone does not distinguish them (BinOp kind,
    /// ValDef id, MethodCall method_id, etc.).
    fn expr_hash<H: Hasher>(expr: &Expr, state: &mut H) {
        std::mem::discriminant(expr).hash(state);
        match expr {
            Expr::Const(c) => {
                // `Constant` has no `Hash` impl; sigma_serialize produces
                // canonical bytes for structurally-equal Constants.
                if let Ok(bytes) = c.sigma_serialize_bytes() {
                    bytes.hash(state);
                }
            }
            Expr::ConstPlaceholder(cp) => {
                cp.id.hash(state);
            }
            Expr::ValUse(vu) => {
                vu.val_id.0.hash(state);
                std::mem::discriminant(&vu.tpe).hash(state);
            }
            Expr::GlobalVars(gv) => {
                std::mem::discriminant(gv).hash(state);
            }
            Expr::ValDef(vd) => {
                vd.expr.id.0.hash(state);
            }
            Expr::BinOp(b) => {
                // BinOpKind is a 2-level enum (outer category × inner op);
                // hash both discriminants for distinctness.
                std::mem::discriminant(&b.expr.kind).hash(state);
                match &b.expr.kind {
                    BinOpKind::Arith(a) => std::mem::discriminant(a).hash(state),
                    BinOpKind::Relation(r) => std::mem::discriminant(r).hash(state),
                    BinOpKind::Logical(l) => std::mem::discriminant(l).hash(state),
                    BinOpKind::Bit(b) => std::mem::discriminant(b).hash(state),
                }
            }
            Expr::FuncValue(fv) => {
                for arg in fv.args() {
                    arg.idx.0.hash(state);
                    std::mem::discriminant(&arg.tpe).hash(state);
                }
            }
            Expr::PropertyCall(pc) => {
                pc.expr.method.method_id().0.hash(state);
            }
            Expr::MethodCall(mc) => {
                mc.expr.method.method_id().0.hash(state);
            }
            Expr::SelectField(sf) => {
                sf.expr.field_index.zero_based_index().hash(state);
            }
            _ => {
                // Other variants: discriminant + child hashes suffice for
                // structural distinctness. PartialEq is the final arbiter on
                // collision.
            }
        }
        for child in direct_children(expr) {
            expr_hash(child, state);
        }
    }

    /// Hash-cons table mirroring Scala's `_globalDefs` + `ThunkScope.bodyDefs`
    /// chain. Each scope has its own structural-equality map; lookup walks
    /// from the current scope up through its parent chain to the root.
    pub(super) struct SymTable {
        /// `scope_parents[i]` = parent scope of scope i, or None for root
        /// (scope 0).
        scope_parents: Vec<Option<ScopeId>>,
        /// Per-scope structural-equality maps:
        /// `scope_defs[scope_id][ExprKey] = SymId`.
        scope_defs: Vec<HashMap<ExprKey, SymId>>,
        /// QB Session 21 — Probe 2 instrumentation for the handoff §3.2
        /// per-thunk-distinct-sym implementation sketch. `sym_scope[id]`
        /// records the scope where SymId `id` was first interned. Used by
        /// `find_or_intern` to detect (and count) sibling-thunk lookup
        /// attempts. The post-`find` `is_ancestor` gate that the handoff
        /// prescribed is empirically a no-op because `find` already walks
        /// current→parent→root only.
        sym_scope: Vec<ScopeId>,
        /// QB Session 25 — β.1 canonical ExprKey count (mirror of Scala's
        /// `_globalDefs` hash-cons table + `mainG.hasManyUsagesGlobal` use
        /// count). `canonical[ExprKey]` = the FIRST SymId interned for that
        /// structural form across ALL scopes. Sibling scopes that intern the
        /// same shape get DISTINCT SymIds (per Thunks.scala semantics; see
        /// G.2.1 sibling-independence test) but all map to the same canonical
        /// SymId here. The emission gate at Root dispatch uses
        /// `canonical_count >= 2` (not per-sym count) and `LCA(canonical_scopes)
        /// == root` to mirror Scala's `hasManyUsagesGlobal` + first-use-scope
        /// placement.
        canonical: HashMap<ExprKey, SymId>,
        /// β.1 — count per canonical SymId (incremented on every
        /// `find_or_intern` regardless of which sibling sym was returned).
        canonical_counts: HashMap<SymId, u32>,
        /// β.1 — list of scopes where each canonical SymId was encountered.
        /// LCA of this list determines emission placement (root for cross-
        /// sibling-shared; sub-scope for ancestor/descendant-only sharing).
        canonical_scopes: HashMap<SymId, Vec<ScopeId>>,
        /// Counter for fresh sym IDs.
        next_sym: SymId,
    }

    impl SymTable {
        pub fn new() -> Self {
            // Scope 0 = root
            Self {
                scope_parents: vec![None],
                scope_defs: vec![HashMap::new()],
                sym_scope: Vec::new(),
                canonical: HashMap::new(),
                canonical_counts: HashMap::new(),
                canonical_scopes: HashMap::new(),
                next_sym: 0,
            }
        }

        /// QB Session 21 — accessor for the scope where SymId `sym` was
        /// first interned. See `sym_scope` field documentation.
        #[allow(dead_code)]
        pub fn scope_of(&self, sym: SymId) -> ScopeId {
            self.sym_scope[sym as usize]
        }

        /// QB Session 25 — β.1 — canonical SymId for an Expr (the first sym
        /// ever interned for this structural form across all scopes). Returns
        /// None if `expr` was never interned. Expects span-stripped input
        /// (mirrors `find` / `find_or_intern` conventions).
        #[allow(dead_code)]
        pub fn canonical_for(&self, expr: &Expr) -> Option<SymId> {
            self.canonical.get(&ExprKey(expr.clone())).copied()
        }

        /// β.1 — aggregate count for a canonical SymId. Returns 0 if `sym`
        /// is not a canonical representative.
        #[allow(dead_code)]
        pub fn canonical_count(&self, sym: SymId) -> u32 {
            self.canonical_counts.get(&sym).copied().unwrap_or(0)
        }

        /// β.1 — scope list for a canonical SymId. Returns empty slice if
        /// not a canonical rep.
        #[allow(dead_code)]
        pub fn canonical_scopes_for(&self, sym: SymId) -> &[ScopeId] {
            self.canonical_scopes
                .get(&sym)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        }

        /// QB Session 26 — read-only enumerator of canonical SymIds. Returns
        /// sorted Vec for deterministic dump ordering. Consumer is the
        /// `CSE_TRACE_CANONICAL=1` diagnostic at the v2 emission gate; the
        /// dump cross-references aggregate canonical state against the per-
        /// dispatch `sym_counts` + S24 `sym_home==root` emission decision to
        /// surface the dexy-extracted-by-Scala vs chaincash-rejected-by-Scala
        /// shape distinction the S25 β.1 falsification exposed.
        #[allow(dead_code)]
        pub fn canonical_syms(&self) -> Vec<SymId> {
            let mut v: Vec<SymId> = self.canonical_counts.keys().copied().collect();
            v.sort();
            v
        }

        /// β.1 — Lowest common ancestor of a non-empty set of scopes. Returns
        /// 0 (root) for empty input or when no closer common ancestor exists.
        /// Algorithm: collect ancestors of `scopes[0]` into a set, then for
        /// each subsequent scope walk parents until hitting that set.
        #[allow(dead_code)]
        pub fn lca_of_scopes(&self, scopes: &[ScopeId]) -> ScopeId {
            if scopes.is_empty() {
                return 0;
            }
            let mut acc = scopes[0];
            for &s in &scopes[1..] {
                acc = self.lca_pair(acc, s);
                if acc == 0 {
                    return 0;
                }
            }
            acc
        }

        fn lca_pair(&self, a: ScopeId, b: ScopeId) -> ScopeId {
            // Collect a's ancestors (incl. self) into a set.
            let mut a_chain: std::collections::HashSet<ScopeId> =
                std::collections::HashSet::new();
            let mut cur = a;
            loop {
                a_chain.insert(cur);
                match self.scope_parents[cur] {
                    Some(p) => cur = p,
                    None => break,
                }
            }
            // Walk b's ancestor chain; first hit is the LCA.
            let mut cur = b;
            loop {
                if a_chain.contains(&cur) {
                    return cur;
                }
                match self.scope_parents[cur] {
                    Some(p) => cur = p,
                    None => return cur,
                }
            }
        }

        /// Create a new child scope under `parent`. Returns the new scope id.
        pub fn new_scope(&mut self, parent: ScopeId) -> ScopeId {
            let id = self.scope_parents.len();
            self.scope_parents.push(Some(parent));
            self.scope_defs.push(HashMap::new());
            id
        }

        /// Search the scope chain for `expr` starting at `scope`, walking
        /// current → parent → ... → root. Mirrors `ThunkScope.findDef`.
        ///
        /// Env-gated trace `CSE_TRACE_SCOPE_CHAIN=1` emits one
        /// `[HC/chain] visit scope=X hit=true|false` line per scope
        /// visited, plus a terminal `[HC/chain] result=Some(N)|None`
        /// line. Used by the G.2.4c scope-chain probe to compare
        /// hash-cons-path decisions against `process_ast_graph_impl`
        /// decisions on the same source positions.
        pub fn find(&self, expr: &Expr, scope: ScopeId) -> Option<SymId> {
            let trace = std::env::var("CSE_TRACE_SCOPE_CHAIN").is_ok();
            let key = ExprKey(expr.clone());
            let mut current = scope;
            loop {
                let hit = self.scope_defs[current].get(&key).copied();
                if trace {
                    eprintln!(
                        "[HC/chain]   visit scope={} hit={}",
                        current,
                        hit.is_some()
                    );
                }
                if let Some(sym_id) = hit {
                    if trace {
                        eprintln!("[HC/chain] result=Some({})", sym_id);
                    }
                    return Some(sym_id);
                }
                match self.scope_parents[current] {
                    Some(parent) => current = parent,
                    None => {
                        if trace {
                            eprintln!("[HC/chain] result=None");
                        }
                        return None;
                    }
                }
            }
        }

        /// Insert `expr` into `scope` with a fresh `SymId`. Returns the
        /// assigned id. Caller must only call this after `find` returns None
        /// (otherwise the new id shadows the existing entry in this scope
        /// but the ancestor entry remains visible from sibling scopes).
        pub fn intern(&mut self, expr: &Expr, scope: ScopeId) -> SymId {
            let id = self.next_sym;
            self.next_sym += 1;
            self.scope_defs[scope].insert(ExprKey(expr.clone()), id);
            debug_assert_eq!(self.sym_scope.len(), id as usize);
            self.sym_scope.push(scope);
            id
        }

        /// Combined `find` + `intern`. Returns `(sym_id, is_new)` where
        /// `is_new = true` on first encounter.
        ///
        /// QB Session 21 — Probe 2 literal implementation of handoff §3.2
        /// per-thunk-distinct-sym semantics. The handoff prescribed
        /// `is_ancestor(scope_of(sym), scope)` check after `find` returns
        /// `Some`. EMPIRICALLY this gate is a no-op: `find` already walks
        /// current→parent→root only, never visiting sibling scope_defs, so
        /// any sym it returns has `scope_of(sym)` in the current ancestor
        /// chain by construction. The diagnostic `CSE_TRACE_SIBLING_SCAN=1`
        /// emits `visible=true|false` for each call; in 33rd falsification
        /// fingerprint run on sig-15, `visible=false` count is 0.
        pub fn find_or_intern(&mut self, expr: &Expr, scope: ScopeId) -> (SymId, bool) {
            let trace = std::env::var("CSE_TRACE_SIBLING_SCAN").is_ok();
            let result = match self.find(expr, scope) {
                Some(sym_id) => {
                    let sym_home = self.sym_scope[sym_id as usize];
                    let visible = self.is_ancestor(sym_home, scope);
                    if trace {
                        eprintln!(
                            "[HC/sibling-scan] sym={} sym_scope={} lookup_scope={} visible={}",
                            sym_id, sym_home, scope, visible
                        );
                    }
                    if visible {
                        (sym_id, false)
                    } else {
                        // Handoff §3.2 fall-through: fresh intern in current
                        // scope. Empirically unreachable — see method doc.
                        let new_id = self.intern(expr, scope);
                        (new_id, true)
                    }
                }
                None => {
                    let sym_id = self.intern(expr, scope);
                    (sym_id, true)
                }
            };
            // QB Session 25 — β.1: establish/update canonical-form tracking.
            // The canonical SymId is the FIRST sym ever interned for this
            // ExprKey across ALL scopes (including sibling scopes that the
            // scope-chain `find` cannot see). On every lookup-or-intern we
            // bump the canonical count and record the scope, so the emission
            // pass can apply Scala's `hasManyUsagesGlobal` semantics (count
            // across the whole graph, not just the visible ancestor chain).
            let (sym_id, _is_new) = result;
            let key = ExprKey(expr.clone());
            let canonical_sym = *self.canonical.entry(key).or_insert(sym_id);
            *self.canonical_counts.entry(canonical_sym).or_insert(0) += 1;
            self.canonical_scopes
                .entry(canonical_sym)
                .or_default()
                .push(scope);
            result
        }

        /// Is `ancestor` an ancestor of (or equal to) `scope`? Used to
        /// determine whether a sym's owning scope is visible from the
        /// current scope.
        pub fn is_ancestor(&self, ancestor: ScopeId, scope: ScopeId) -> bool {
            let mut current = scope;
            loop {
                if current == ancestor {
                    return true;
                }
                match self.scope_parents[current] {
                    Some(parent) => current = parent,
                    None => return false,
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ergotree_ir::mir::bin_op::{ArithOp, BinOp, BinOpKind};
        use ergotree_ir::mir::constant::Constant;
        use ergotree_ir::mir::val_def::{ValDef, ValId};
        use ergotree_ir::source_span::{SourceSpan, Spanned};

        fn c_i64(v: i64) -> Expr {
            Expr::Const(Constant::from(v))
        }

        fn binop_plus_spanned(left: Expr, right: Expr, span_offset: usize) -> Expr {
            Expr::BinOp(Spanned {
                source_span: SourceSpan {
                    offset: span_offset,
                    length: 1,
                },
                expr: BinOp {
                    kind: BinOpKind::Arith(ArithOp::Plus),
                    left: Box::new(left),
                    right: Box::new(right),
                },
            })
        }

        fn vdef(id: u32, rhs: Expr) -> Expr {
            Expr::ValDef(Spanned {
                source_span: SourceSpan::empty(),
                expr: ValDef {
                    id: ValId(id),
                    rhs: Box::new(rhs),
                },
            })
        }

        fn hash_of(expr: &Expr) -> u64 {
            use std::collections::hash_map::DefaultHasher;
            let mut h = DefaultHasher::new();
            ExprKey(expr.clone()).hash(&mut h);
            h.finish()
        }

        /// Test #1: intern `Const(5i64)` twice in root scope. Second call
        /// returns same SymId; table still has 1 entry.
        #[test]
        fn sym_table_same_expr_same_scope() {
            let mut t = SymTable::new();
            let e = c_i64(5);
            let (id1, new1) = t.find_or_intern(&e, 0);
            let (id2, new2) = t.find_or_intern(&e, 0);
            assert!(new1);
            assert!(!new2);
            assert_eq!(id1, id2);
            assert_eq!(t.scope_defs[0].len(), 1);
        }

        /// Test #2: intern E in root; create child scope; `find_or_intern(E,
        /// child)` returns the same SymId, `is_new=false`. Confirms the
        /// scope-chain walk sees the root-level entry.
        #[test]
        fn sym_table_same_expr_child_scope() {
            let mut t = SymTable::new();
            let e = c_i64(7);
            let (root_id, _) = t.find_or_intern(&e, 0);
            let child = t.new_scope(0);
            let (child_id, is_new) = t.find_or_intern(&e, child);
            assert!(!is_new, "child lookup should find root entry");
            assert_eq!(root_id, child_id);
        }

        /// Test #3: intern E in scope A; create sibling scope B (both
        /// children of root). `find_or_intern(E, B)` walks B → root, misses
        /// (E lives in A.bodyDefs, not root), returns a NEW sym. Two
        /// separate SymIds. This is the SIBLING INDEPENDENCE behavior
        /// G.2.1 confirmed from Thunks.scala.
        #[test]
        fn sym_table_sibling_scopes_create_separate_syms() {
            let mut t = SymTable::new();
            let scope_a = t.new_scope(0);
            let scope_b = t.new_scope(0);
            let e = c_i64(42);
            let (id_a, new_a) = t.find_or_intern(&e, scope_a);
            let (id_b, new_b) = t.find_or_intern(&e, scope_b);
            assert!(new_a);
            assert!(new_b, "sibling scope should not see scope_a's entry");
            assert_ne!(
                id_a, id_b,
                "siblings must produce distinct SymIds (Thunks.scala parity)"
            );
        }

        /// Test #4: root(0) → child(1) → grandchild(2). Verify `is_ancestor`
        /// returns true for ancestors (incl. self), false otherwise.
        #[test]
        fn sym_table_is_ancestor() {
            let mut t = SymTable::new();
            let child = t.new_scope(0);
            let grandchild = t.new_scope(child);
            assert!(t.is_ancestor(0, grandchild));
            assert!(t.is_ancestor(child, grandchild));
            assert!(!t.is_ancestor(grandchild, 0));
            assert!(t.is_ancestor(child, child), "self-ancestor");
        }

        /// Test #5: two separately-constructed `BinOp(Plus, Const(1),
        /// Const(2))` with different `SourceSpan` are `ExprKey`-equal AND
        /// hash to the same value AFTER source-span strip. (Consumers
        /// canonicalize via `strip_source_spans` before insertion.)
        #[test]
        fn expr_key_structural_equality() {
            let e1 = super::super::strip_source_spans(binop_plus_spanned(c_i64(1), c_i64(2), 10));
            let e2 = super::super::strip_source_spans(binop_plus_spanned(c_i64(1), c_i64(2), 20));
            assert_eq!(ExprKey(e1.clone()), ExprKey(e2.clone()));
            assert_eq!(hash_of(&e1), hash_of(&e2));
        }

        /// Test #6: `BinOp(Plus, Const(1), Const(2))` ≠ `BinOp(Plus,
        /// Const(2), Const(1))`. Operand order matters for both equality
        /// and hash.
        #[test]
        fn expr_key_structural_inequality() {
            let e1 = super::super::strip_source_spans(binop_plus_spanned(c_i64(1), c_i64(2), 0));
            let e2 = super::super::strip_source_spans(binop_plus_spanned(c_i64(2), c_i64(1), 0));
            assert_ne!(ExprKey(e1.clone()), ExprKey(e2.clone()));
            assert_ne!(
                hash_of(&e1),
                hash_of(&e2),
                "operand order must affect hash"
            );
        }

        /// Test #7: `ValDef(id=1, rhs=Const(5))` and `ValDef(id=2,
        /// rhs=Const(5))` are NOT equal (different ids) and must hash
        /// differently. ValDefs are rarely extraction candidates but the
        /// hash must be consistent with `Expr: PartialEq`.
        #[test]
        fn expr_key_vd_id_independence() {
            let e1 = vdef(1, c_i64(5));
            let e2 = vdef(2, c_i64(5));
            assert_ne!(ExprKey(e1.clone()), ExprKey(e2.clone()));
            assert_ne!(hash_of(&e1), hash_of(&e2), "ValDef id must affect hash");
        }

        /// Test #8: intern E in root (scope 0); create child(1); intern E'
        /// (different expr) in child(1). `find(E, child)` finds the root
        /// entry via the chain walk; `find(E', root)` misses (child entry
        /// is not visible from an ancestor).
        #[test]
        fn sym_table_find_after_reparent() {
            let mut t = SymTable::new();
            let e_root = c_i64(100);
            let e_child = c_i64(200);
            let (root_sym, _) = t.find_or_intern(&e_root, 0);
            let child = t.new_scope(0);
            let (child_sym, _) = t.find_or_intern(&e_child, child);
            // From child: e_root visible via parent chain.
            assert_eq!(t.find(&e_root, child), Some(root_sym));
            // From root: e_child not visible (it lives in a descendant scope).
            assert_eq!(t.find(&e_child, 0), None);
            // Sanity: child sees its own entry too.
            assert_eq!(t.find(&e_child, child), Some(child_sym));
        }

        /// G.2.3 Audit 1.B — perf baseline for `expr_hash`.
        ///
        /// Runs 1_000_000 hashes over a representative AST mix and prints
        /// µs/call. Gated `#[ignore]`; invoke with:
        ///   `cargo test -p ergoscript-compiler --lib --release \
        ///       hash_cons::tests::audit_1b_expr_hash_perf -- --ignored --nocapture`
        ///
        /// Projection: per G.2.3 handoff, F.2 corpus is 575 programs × ~50
        /// Consts/program × multiple `find_or_intern` calls/pipeline pass.
        /// Pass condition: projected wall-time adds <10% to F.2 run. The bench
        /// prints both per-call time and a projected total for the reader to
        /// judge against the F.2 baseline (~seconds, varies by host).
        #[test]
        #[ignore]
        fn audit_1b_expr_hash_perf() {
            use std::collections::hash_map::DefaultHasher;
            use std::time::Instant;

            // Mixed AST: a small expression containing every arm `expr_hash`
            // dispatches on by-arm (Const, ValUse, BinOp) plus structural
            // recursion via `direct_children`. Keep representative, not
            // huge — Scala's `_globalDefs` keys are typically small sub-exprs.
            let mix = binop_plus_spanned(
                binop_plus_spanned(c_i64(1), c_i64(2), 0),
                binop_plus_spanned(c_i64(3), c_i64(4), 0),
                0,
            );
            let mix = super::super::strip_source_spans(mix);

            const N: u64 = 1_000_000;
            let t0 = Instant::now();
            let mut acc: u64 = 0;
            for _ in 0..N {
                let mut h = DefaultHasher::new();
                ExprKey(mix.clone()).hash(&mut h);
                acc = acc.wrapping_add(h.finish());
            }
            let dt = t0.elapsed();
            let ns_per_call = dt.as_nanos() as f64 / N as f64;

            // Projected F.2 work: 575 programs × 50 Consts × 4 pipeline passes
            // = 115_000 `find_or_intern` hashes. Each hash recurses into all
            // sub-exprs, so per-program work is much larger; this is a floor.
            let proj_hashes_low: u64 = 575 * 50 * 4;
            let proj_us = (ns_per_call * proj_hashes_low as f64) / 1_000.0;

            eprintln!(
                "[AUDIT 1.B] N={N} hashes elapsed={dt:?} per-call={ns_per_call:.1}ns \
                 floor-projection={proj_us:.1}µs ({proj_hashes_low} hashes × per-call) \
                 acc={acc}"
            );
        }

        /// G.2.3 Audit 1.B (Const arm) — perf isolation of the
        /// `sigma_serialize_bytes`-per-Const path. The Const arm allocates a
        /// `Vec<u8>` per call, so it's the suspected hot-spot. Bench it
        /// separately to compare against the structural-mix bench above.
        #[test]
        #[ignore]
        fn audit_1b_expr_hash_perf_const_arm() {
            use std::collections::hash_map::DefaultHasher;
            use std::time::Instant;

            let c = super::super::strip_source_spans(c_i64(0x1234_5678_9abc_def0));

            const N: u64 = 1_000_000;
            let t0 = Instant::now();
            let mut acc: u64 = 0;
            for _ in 0..N {
                let mut h = DefaultHasher::new();
                ExprKey(c.clone()).hash(&mut h);
                acc = acc.wrapping_add(h.finish());
            }
            let dt = t0.elapsed();
            let ns_per_call = dt.as_nanos() as f64 / N as f64;

            eprintln!(
                "[AUDIT 1.B / Const] N={N} hashes elapsed={dt:?} per-call={ns_per_call:.1}ns \
                 acc={acc}"
            );
        }
    }
}

/// G.2.3a — walker-completeness probe for `direct_children`.
///
/// Audit 1.A failed: 10 Expr variants have real Expr children but fall through
/// the catch-all `_ => vec![]` arm, so all 50+ helpers that walk via
/// `direct_children` (count_dag_usages / contains_val_use / collect_* / expr_hash)
/// silently under-count uses for ASTs containing them.
///
/// This probe constructs a minimal AST for each affected variant and asserts
/// the expected child count. Today (HEAD `51dda6a1`) the 10 high/med-risk
/// arms FAIL; after the fix they pass. Test is the falsification artifact —
/// partial arm coverage shows up as partial pass.
#[cfg(test)]
mod walker_completeness_probe {
    use super::*;
    use ergotree_ir::chain::ergo_box::RegisterId;
    use ergotree_ir::mir::bit_inversion::BitInversion;
    use ergotree_ir::mir::calc_sha256::CalcSha256;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::create_avl_tree::CreateAvlTree;
    use ergotree_ir::mir::deserialize_register::DeserializeRegister;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef;
    use ergotree_ir::mir::sigma_prop_is_proven::SigmaPropIsProven;
    use ergotree_ir::mir::subst_const::SubstConstants;
    use ergotree_ir::mir::xor::Xor;
    use ergotree_ir::mir::xor_of::XorOf;
    use ergotree_ir::mir::zk_proof::ZkProofBlock;
    use ergotree_ir::source_span::{SourceSpan, Spanned};
    use ergotree_ir::types::stype::SType;

    fn ci(v: i32) -> Expr {
        Expr::Const(Constant::from(v))
    }
    fn cb_byte(v: i8) -> Expr {
        Expr::Const(Constant::from(v))
    }
    fn cb_bytes() -> Expr {
        Expr::Const(Constant::from(vec![1i8, 2, 3]))
    }
    fn cb_bool() -> Expr {
        Expr::Const(Constant::from(true))
    }

    #[test]
    fn calc_sha256_has_one_child() {
        let e = Expr::CalcSha256(CalcSha256 { input: Box::new(cb_bytes()) });
        assert_eq!(direct_children(&e).len(), 1);
    }

    #[test]
    fn bit_inversion_has_one_child() {
        let e = Expr::BitInversion(BitInversion { input: Box::new(ci(7)) });
        assert_eq!(direct_children(&e).len(), 1);
    }

    #[test]
    fn extract_bytes_with_no_ref_has_one_child() {
        // SBox-typed input; substituting a placeholder of the right shape is
        // sufficient for the walker-arm test — we don't evaluate.
        let e = Expr::ExtractBytesWithNoRef(ExtractBytesWithNoRef {
            input: Box::new(Expr::GlobalVars(
                ergotree_ir::mir::global_vars::GlobalVars::SelfBox,
            )),
        });
        assert_eq!(direct_children(&e).len(), 1);
    }

    #[test]
    fn sigma_prop_is_proven_has_one_child() {
        use ergotree_ir::mir::create_provedlog::CreateProveDlog;
        let body = Expr::CreateProveDlog(CreateProveDlog {
            input: Box::new(Expr::GlobalVars(
                ergotree_ir::mir::global_vars::GlobalVars::Height,
            )),
        });
        let e = Expr::SigmaPropIsProven(SigmaPropIsProven { input: Box::new(body) });
        assert_eq!(direct_children(&e).len(), 1);
    }

    #[test]
    fn xor_of_has_one_child() {
        let e = Expr::XorOf(XorOf {
            input: Box::new(cb_bool()),
        });
        assert_eq!(direct_children(&e).len(), 1);
    }

    #[test]
    fn xor_has_two_children() {
        let e = Expr::Xor(Xor {
            left: Box::new(cb_bytes()),
            right: Box::new(cb_bytes()),
        });
        assert_eq!(direct_children(&e).len(), 2);
    }

    #[test]
    fn subst_constants_has_three_children() {
        let e = Expr::SubstConstants(Spanned {
            source_span: SourceSpan::empty(),
            expr: SubstConstants {
                script_bytes: Box::new(cb_bytes()),
                positions: Box::new(Expr::Const(Constant::from(vec![0i32]))),
                new_values: Box::new(Expr::Const(Constant::from(vec![1i32]))),
            },
        });
        assert_eq!(direct_children(&e).len(), 3);
    }

    #[test]
    fn create_avl_tree_has_three_or_four_children() {
        let e3 = Expr::CreateAvlTree(CreateAvlTree {
            flags: Box::new(cb_byte(1)),
            digest: Box::new(cb_bytes()),
            key_length: Box::new(ci(32)),
            value_length: None,
        });
        assert_eq!(direct_children(&e3).len(), 3);
        let e4 = Expr::CreateAvlTree(CreateAvlTree {
            flags: Box::new(cb_byte(1)),
            digest: Box::new(cb_bytes()),
            key_length: Box::new(ci(32)),
            value_length: Some(Box::new(ci(8))),
        });
        assert_eq!(direct_children(&e4).len(), 4);
    }

    #[test]
    fn deserialize_register_default_visited_when_present() {
        let none_default = Expr::DeserializeRegister(DeserializeRegister {
            reg: RegisterId::R0,
            tpe: SType::SLong,
            default: None,
        });
        assert_eq!(direct_children(&none_default).len(), 0);
        let some_default = Expr::DeserializeRegister(DeserializeRegister {
            reg: RegisterId::R0,
            tpe: SType::SLong,
            default: Some(Box::new(ci(0))),
        });
        assert_eq!(direct_children(&some_default).len(), 1);
    }

    #[test]
    fn zk_proof_block_has_one_child() {
        // ZkProofBlock requires SSigmaProp body — use a CreateProveDlog
        // placeholder so the assertion focuses purely on child count.
        use ergotree_ir::mir::create_provedlog::CreateProveDlog;
        let body = Expr::CreateProveDlog(CreateProveDlog {
            input: Box::new(Expr::GlobalVars(
                ergotree_ir::mir::global_vars::GlobalVars::Height,
            )),
        });
        let e = Expr::ZkProofBlock(ZkProofBlock { input: Box::new(body) });
        assert_eq!(direct_children(&e).len(), 1);
    }
}
