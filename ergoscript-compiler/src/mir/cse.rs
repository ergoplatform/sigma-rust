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
            let expr = strip_source_spans(expr);
            let global_max_id = find_max_val_id(&expr);
            let cse_result = cse_expr(expr, global_max_id, false);
            let branch_cse_max = find_max_val_id(&cse_result);
            let branch_cse = apply_cse_within_branches(cse_result, branch_cse_max);
            let pre_extract_max = find_max_val_id(&branch_cse);
            let mut next_id = pre_extract_max + 1;
            let pre_extracted = pre_extract_from_valdefs(branch_cse, &mut next_id);
            let inlined = inline_single_use_vals(pre_extracted);
            let deduped = deduplicate_inner_consts(inlined);
            // Re-run outer inlining: Phase 3 (relational BinOp dedup, sig-15
            // sigmausd_bank close) extracts a new branch-scope ValDef whose
            // RHS may consume two prior uses of an OUTER user-bound val
            // (e.g. `oraclePoolNFT`), reducing it to a single use. The earlier
            // `inline_single_use_vals` pass ran before that drop, so the now-
            // single-use outer ValDef would otherwise survive as overhead.
            let deduped = inline_single_use_vals(deduped);
            let flattened = flatten_nested_blocks(deduped);

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
            let reordered = reorder_valdefs(reassigned);
            sequential_renumber(reordered)
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
    match expr {
        Expr::ValUse(_) => true,
        Expr::PropertyCall(s) => contains_val_use(&s.expr.obj),
        Expr::MethodCall(s) => {
            contains_val_use(&s.expr.obj) || s.expr.args.iter().any(contains_val_use)
        }
        Expr::ExtractAmount(ea) => contains_val_use(&ea.input),
        Expr::ExtractRegisterAs(s) => contains_val_use(&s.expr.input),
        Expr::ExtractScriptBytes(esb) => contains_val_use(&esb.input),
        Expr::ExtractBytes(eb) => contains_val_use(&eb.input),
        Expr::ExtractId(ei) => contains_val_use(&ei.input),
        Expr::ExtractCreationInfo(eci) => contains_val_use(&eci.input),
        Expr::SizeOf(so) => contains_val_use(&so.input),
        Expr::ByIndex(s) => {
            contains_val_use(&s.expr.input)
                || contains_val_use(&s.expr.index)
                || s.expr.default.as_ref().is_some_and(|d| contains_val_use(d))
        }
        Expr::SelectField(s) => contains_val_use(&s.expr.input),
        Expr::OptionGet(s) => contains_val_use(&s.expr.input),
        Expr::OptionIsDefined(s) => contains_val_use(&s.expr.input),
        Expr::BinOp(s) => contains_val_use(&s.expr.left) || contains_val_use(&s.expr.right),
        Expr::LogicalNot(s) => contains_val_use(&s.expr.input),
        Expr::Negation(s) => contains_val_use(&s.expr.input),
        Expr::Upcast(uc) => contains_val_use(&uc.input),
        Expr::ByteArrayToBigInt(s) => contains_val_use(&s.expr.input),
        Expr::CalcBlake2b256(cb) => contains_val_use(&cb.input),
        Expr::SigmaPropBytes(spb) => contains_val_use(&spb.input),
        _ => false,
    }
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
        // Leaf nodes — no children
        Expr::Const(_)
        | Expr::ConstPlaceholder(_)
        | Expr::GlobalVars(_)
        | Expr::ValUse(_)
        | Expr::Context
        | Expr::Global => vec![],
        // Catch-all for less common nodes
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

    // Step 3: Return (expr, usage_count) pairs
    unique.into_iter().zip(parent_counts).collect()
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
    process_ast_graph_impl(expr, global_max_id, dag_usages, schedule, ScopeMode::Root)
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

    process_ast_graph_impl(expr, global_max_id, dag_usages, schedule, ScopeMode::Branch)
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
                // sig-15 sigmao_option: `SelectField(ValUse(outer_val), n)` on a tuple
                // bound at root (typical pattern: `val tup = box.tokens.getOrElse(i, _)`
                // followed by many `tup._1` / `tup._2` references) is hash-consed in
                // Scala's graph IR — the SelectField sym depends only on the outer
                // ValUse, which is itself root-bound. Scala's `mainG.hasManyUsagesGlobal`
                // counts uses across &&/|| ThunkDefs AND If-branch ThunkDefs and emits
                // the ValDef at root regardless of where the uses physically sit. The
                // strict `appears_in_main_scope` AND permissive `appears_outside_if_branches`
                // both reject when all uses are inside If arms — but this is the wrong
                // model for SelectField(ValUse): the ValUse itself is the only stable
                // dependency, and it IS at root, so the SelectField can be hoisted to
                // root unconditionally. (`references_locally_defined` upstream already
                // rejects ValUses targeting branch-local IDs, so reaching this point
                // implies the ValUse is root-bound.)
                let is_select_field_on_val_use = matches!(node,
                    Expr::SelectField(s) if matches!(&*s.expr.input, Expr::ValUse(_))
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
                let needs_check = (use_if_branch_check
                    || touches_context(node)
                    || is_pure_const_upcast
                    || is_bare_const)
                    && !is_select_field_on_val_use;
                if needs_check {
                    let in_scope = if use_if_branch_check {
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
