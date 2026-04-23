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
            // Normalize all source spans to empty so CSE hash-consing
            // treats structurally identical nodes as equal regardless
            // of source position. Without this, two identical expressions
            // from different source locations (e.g. SELF.R4[T].get used
            // in two places) would be treated as different nodes.
            let expr = strip_source_spans(expr);
            let global_max_id = find_max_val_id(&expr);
            let cse_result = cse_expr(expr, global_max_id, false);
            // Inline single-use vals (e.g., after CSE extracts Upcast(ValUse(x), BigInt),
            // the original val x may become single-use and should be folded in)
            let inlined = inline_single_use_vals(cse_result);
            // Extract duplicate constants in inner blocks as ValDefs (matches Scala's
            // graph IR where multi-use constants get shared symbols within ThunkDef scopes)
            let deduped = deduplicate_inner_consts(inlined);
            // Reorder ValDefs to match Scala's DFS dependency order, then renumber
            let reordered = reorder_valdefs(deduped);
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
            );

            // Any ValDefs not referenced transitively from result go at the end
            for item in &s.expr.items {
                if let Expr::ValDef(vd) = item {
                    if !emitted_ids.contains(&vd.expr.id.0) {
                        emitted_ids.insert(vd.expr.id.0);
                        emitted.push(item.clone());
                    }
                }
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
            let mut inline_map: std::collections::HashMap<u32, Expr> =
                std::collections::HashMap::new();
            for item in &items {
                if let Expr::ValDef(vd) = item {
                    let id = vd.expr.id.0;
                    let count = use_counts.get(&id).copied().unwrap_or(0);
                    if count == 1 {
                        inline_map.insert(id, (*vd.expr.rhs).clone());
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

            // Remove inlined ValDefs from items
            let remaining_items: Vec<Expr> = items
                .into_iter()
                .filter(|item| {
                    if let Expr::ValDef(vd) = item {
                        !inline_map.contains_key(&vd.expr.id.0)
                    } else {
                        true
                    }
                })
                .collect();

            // Substitute all inlined vals using replace_all
            let mut block = Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: remaining_items,
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
            block
        }
        Expr::If(if_op) => Expr::If(ergotree_ir::mir::if_op::If {
            condition: inline_single_use_vals(*if_op.condition).into(),
            true_branch: inline_single_use_vals(*if_op.true_branch).into(),
            false_branch: inline_single_use_vals(*if_op.false_branch).into(),
        }),
        other => other,
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

/// Dedup constants within a block, then recurse for deeper If nodes.
fn dedup_consts_in_block(expr: Expr) -> Expr {
    // First recurse to handle nested If expressions
    let expr = deduplicate_inner_consts(expr);

    match expr {
        Expr::BlockValue(s) => {
            // Collect all Const nodes in the block
            let mut all_consts: Vec<Expr> = Vec::new();
            for item in &s.expr.items {
                collect_consts(item, &mut all_consts);
            }
            collect_consts(&s.expr.result, &mut all_consts);

            // Find constants appearing 2+ times (dedup by equality)
            let mut duplicates: Vec<Expr> = Vec::new();
            for c in &all_consts {
                let count = all_consts.iter().filter(|x| *x == c).count();
                if count >= 2 && !duplicates.contains(c) {
                    duplicates.push(c.clone());
                }
            }

            if duplicates.is_empty() {
                return Expr::BlockValue(s);
            }

            let mut next_id = find_max_val_id(&Expr::BlockValue(s.clone())) + 1;
            let mut result_items = s.expr.items;
            let mut result_expr = *s.expr.result;
            let mut new_defs: Vec<Expr> = Vec::new();

            for const_expr in duplicates {
                let val_use = Expr::ValUse(ValUse {
                    val_id: ValId(next_id),
                    tpe: const_expr.tpe(),
                });
                result_items = result_items
                    .into_iter()
                    .map(|item| replace_all(&item, &const_expr, &val_use))
                    .collect();
                result_expr = replace_all(&result_expr, &const_expr, &val_use);

                new_defs.push(Expr::ValDef(Spanned {
                    source_span: SourceSpan::empty(),
                    expr: ValDef {
                        id: ValId(next_id),
                        rhs: const_expr.into(),
                    },
                }));
                next_id += 1;
            }

            // Prepend new const defs before existing items
            new_defs.extend(result_items);
            Expr::BlockValue(Spanned {
                source_span: s.source_span,
                expr: BlockValue {
                    items: new_defs,
                    result: result_expr.into(),
                },
            })
        }
        other => other,
    }
}

/// Collect all Const nodes in an expression tree.
fn collect_consts(expr: &Expr, out: &mut Vec<Expr>) {
    if let Expr::Const(_) = expr {
        out.push(expr.clone());
    }
    for child in direct_children(expr) {
        collect_consts(child, out);
    }
}

/// DFS walk an expression; when a ValUse is found whose ValDef is in val_map
/// and hasn't been emitted yet, recursively emit its RHS dependencies first,
/// then emit the ValDef.
///
/// `in_thunk`: true when we're inside the right arm of a logical &&/|| chain.
/// Collect all ValUse IDs referenced in an expression tree (non-recursive into inner blocks).
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
fn emit_deps(
    expr: &Expr,
    val_map: &HashMap<u32, Expr>,
    emitted: &mut Vec<Expr>,
    emitted_ids: &mut HashSet<u32>,
    in_thunk: bool,
) {
    match expr {
        Expr::ValUse(vu) => {
            let id = vu.val_id.0;
            if !emitted_ids.contains(&id) {
                if let Some(vd_expr) = val_map.get(&id) {
                    if let Expr::ValDef(vd) = vd_expr {
                        // Emit dependencies of this ValDef's RHS first
                        emit_deps(&vd.expr.rhs, val_map, emitted, emitted_ids, in_thunk);
                    }
                    emitted_ids.insert(id);
                    emitted.push(vd_expr.clone());
                }
            }
        }
        Expr::BlockValue(s) => {
            for item in &s.expr.items {
                emit_deps(item, val_map, emitted, emitted_ids, in_thunk);
            }
            emit_deps(&s.expr.result, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::ValDef(s) => emit_deps(&s.expr.rhs, val_map, emitted, emitted_ids, in_thunk),
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
                let reverse_inner = in_thunk && is_or && !left_is_or;
                if reverse_inner {
                    // Innermost || in thunk context: right arm first (matches Scala's
                    // ThunkDef body flatSchedule where ThunkDef children come before parents)
                    emit_deps(&s.expr.right, val_map, emitted, emitted_ids, true);
                    emit_deps(&s.expr.left, val_map, emitted, emitted_ids, true);
                } else {
                    // Left arm is in main scope (not thunk)
                    emit_deps(&s.expr.left, val_map, emitted, emitted_ids, in_thunk);
                    // Right arm enters thunk context
                    emit_deps(&s.expr.right, val_map, emitted, emitted_ids, true);
                }
            } else {
                emit_deps(&s.expr.left, val_map, emitted, emitted_ids, in_thunk);
                emit_deps(&s.expr.right, val_map, emitted, emitted_ids, in_thunk);
            }
        }
        Expr::BoolToSigmaProp(bts) => {
            emit_deps(&bts.input, val_map, emitted, emitted_ids, in_thunk)
        }
        Expr::If(if_op) => {
            // Condition is in the main scope — process normally
            emit_deps(&if_op.condition, val_map, emitted, emitted_ids, in_thunk);
            // If branches are ThunkDef scopes in Scala's graph IR.
            // ThunkDef.deps (free variables) are ordered by symbol ID,
            // so we collect all val refs from both branches, sort by ID,
            // and emit in that order (matching Scala's schedule).
            let mut branch_val_ids: Vec<u32> = Vec::new();
            collect_all_val_uses(&if_op.true_branch, &mut branch_val_ids);
            collect_all_val_uses(&if_op.false_branch, &mut branch_val_ids);
            branch_val_ids.sort();
            branch_val_ids.dedup();
            for id in branch_val_ids {
                if !emitted_ids.contains(&id) {
                    if let Some(vd_expr) = val_map.get(&id) {
                        if let Expr::ValDef(vd) = vd_expr {
                            emit_deps(&vd.expr.rhs, val_map, emitted, emitted_ids, in_thunk);
                        }
                        emitted_ids.insert(id);
                        emitted.push(vd_expr.clone());
                    }
                }
            }
        }
        Expr::Filter(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.condition, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::Exists(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.condition, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::ForAll(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.condition, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::Map(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.mapper, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::Fold(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.zero, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.fold_op, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::FuncValue(fv) => emit_deps(fv.body(), val_map, emitted, emitted_ids, in_thunk),
        Expr::PropertyCall(s) => emit_deps(&s.expr.obj, val_map, emitted, emitted_ids, in_thunk),
        Expr::MethodCall(s) => {
            emit_deps(&s.expr.obj, val_map, emitted, emitted_ids, in_thunk);
            for a in &s.expr.args {
                emit_deps(a, val_map, emitted, emitted_ids, in_thunk);
            }
        }
        Expr::ExtractAmount(ea) => emit_deps(&ea.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::ExtractRegisterAs(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk)
        }
        Expr::ExtractScriptBytes(esb) => {
            emit_deps(&esb.input, val_map, emitted, emitted_ids, in_thunk)
        }
        Expr::ExtractBytes(eb) => emit_deps(&eb.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::ExtractId(ei) => emit_deps(&ei.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::ExtractCreationInfo(eci) => {
            emit_deps(&eci.input, val_map, emitted, emitted_ids, in_thunk)
        }
        Expr::SizeOf(so) => emit_deps(&so.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::ByIndex(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.index, val_map, emitted, emitted_ids, in_thunk);
            if let Some(ref d) = s.expr.default {
                emit_deps(d, val_map, emitted, emitted_ids, in_thunk);
            }
        }
        Expr::SelectField(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::OptionGet(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::OptionIsDefined(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk)
        }
        Expr::OptionGetOrElse(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.default, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::Slice(s) => {
            emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.from, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.until, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::LogicalNot(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::Negation(s) => emit_deps(&s.expr.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::SigmaPropBytes(spb) => emit_deps(&spb.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::Upcast(uc) => emit_deps(&uc.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::Downcast(dc) => emit_deps(&dc.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::CalcBlake2b256(cb) => emit_deps(&cb.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::CreateProveDlog(cpd) => {
            emit_deps(&cpd.input, val_map, emitted, emitted_ids, in_thunk)
        }
        Expr::SigmaAnd(sa) => {
            for i in sa.items.iter() {
                emit_deps(i, val_map, emitted, emitted_ids, in_thunk);
            }
        }
        Expr::SigmaOr(so) => {
            for i in so.items.iter() {
                emit_deps(i, val_map, emitted, emitted_ids, in_thunk);
            }
        }
        Expr::Tuple(t) => {
            for i in t.items.iter() {
                emit_deps(i, val_map, emitted, emitted_ids, in_thunk);
            }
        }
        Expr::TreeLookup(s) => {
            emit_deps(&s.expr.tree, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.key, val_map, emitted, emitted_ids, in_thunk);
            emit_deps(&s.expr.proof, val_map, emitted, emitted_ids, in_thunk);
        }
        Expr::Apply(app) => {
            emit_deps(&app.func, val_map, emitted, emitted_ids, in_thunk);
            for a in &app.args {
                emit_deps(a, val_map, emitted, emitted_ids, in_thunk);
            }
        }
        Expr::And(a) => emit_deps(&a.expr.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::Or(o) => emit_deps(&o.expr.input, val_map, emitted, emitted_ids, in_thunk),
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                emit_deps(item, val_map, emitted, emitted_ids, in_thunk);
            }
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
            collect_and_assign_ids(&if_op.true_branch, id_map, next_id, def_id);
            collect_and_assign_ids(&if_op.false_branch, id_map, next_id, def_id);
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
        // Lambda scope: use the old tree-based approach with savings sort.
        // Lambda bodies use +2 gap from max ID (Scala compiler convention).
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
        // Lambda scope: use +2 gap from max ID (Scala compiler convention)
        let mut next_id = local_max.max(global_max_id) + 2;
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
        match result {
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
        }
    } else if has_lambdas {
        // Top-level with lambdas: use the old savings-based approach.
        // The Scala compiler's flatSchedule (which includes lambda body nodes)
        // changes effective usage counts, so we block ValUse-containing
        // expressions to match. The graph IR approach over-extracts here
        // because it doesn't model the flatSchedule interaction.
        let mut all_subexprs: Vec<Expr> = Vec::new();
        collect_subexprs(&expr, &mut all_subexprs);
        let dag_usages = count_dag_usages(&expr);
        let mut candidates: Vec<(Expr, usize)> = Vec::new();
        for (sub, dag_count) in &dag_usages {
            if *dag_count < 2 {
                continue;
            }
            if !is_collectible(sub) || contains_val_use(sub) {
                continue;
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
        match result {
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
        // Other types: pass through (lambda bodies can't be nested deeper in these)
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
            ergotree_ir::mir::method_call::MethodCall::new(obj, s.expr.method, args)
                .map(|mc| {
                    Expr::MethodCall(Spanned {
                        source_span: s.source_span,
                        expr: mc,
                    })
                })
                .expect("MethodCall::new in map_children")
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
        // FuncValue handled above in process_lambdas; leaves pass through
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
        // Eq/NEq: non-singleton Equals[A: Elem]
        Expr::BinOp(s) => !matches!(
            s.expr.kind,
            ergotree_ir::mir::bin_op::BinOpKind::Relation(
                ergotree_ir::mir::bin_op::RelationOp::Eq
                    | ergotree_ir::mir::bin_op::RelationOp::NEq
            )
        ),
        // Box accessor methods: shared only when the input is a global
        // (SELF.value is one graph node; output.value creates separate nodes)
        Expr::ExtractAmount(ea) => is_input_global(&ea.input),
        Expr::ExtractRegisterAs(s) => is_input_global(&s.expr.input),
        Expr::ExtractScriptBytes(esb) => is_input_global(&esb.input),
        Expr::ExtractBytes(eb) => is_input_global(&eb.input),
        Expr::ExtractId(ei) => is_input_global(&ei.input),
        Expr::ExtractCreationInfo(eci) => is_input_global(&eci.input),
        // ByIndex (Coll.apply in Scala): shared when input is stable
        // (global, PropertyCall chain on global, or referencing a val-bound collection).
        // In Scala's graph, MethodCall(coll, apply, [idx]) is extractable when usages >= 2.
        Expr::ByIndex(s) => is_input_stable(&s.expr.input),
        // OptionGet/IsDefined: separate per call site in Scala graph
        Expr::OptionGet(_) | Expr::OptionIsDefined(_) => false,
        // MethodCall: not shared (rewriteDef produces different results per call)
        Expr::MethodCall(_) => false,
        // Everything else (PropertyCall/tokens, SizeOf, ByIndex, SelectField,
        // arithmetic BinOp, constants): shared
        _ => true,
    }
}

/// Check if an expression resolves to a shared graph node.
/// GlobalVars are singletons. PropertyCall on a global is shared.
/// ByIndex on a shared collection is shared. Used to determine if
/// box accessor methods on it are graph-shared.
fn is_input_global(expr: &Expr) -> bool {
    match expr {
        Expr::GlobalVars(_) | Expr::Context => true,
        Expr::PropertyCall(s) => is_input_global(&s.expr.obj),
        _ => false,
    }
}

/// Like `is_input_global` but also accepts `PropertyCall` on a `ValUse`.
/// In Scala's graph, `MethodCall(coll, apply, idx)` on a val-bound or
/// CSE-extracted collection is shared. `ValUse` indicates a stable binding.
fn is_input_stable(expr: &Expr) -> bool {
    match expr {
        Expr::GlobalVars(_) | Expr::Context => true,
        Expr::PropertyCall(s) => is_input_stable(&s.expr.obj),
        Expr::ValUse(_) => true,
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
        Expr::CalcBlake2b256(cb) => vec![&cb.input],
        Expr::SigmaPropBytes(spb) => vec![&spb.input],
        Expr::CreateProveDlog(cpd) => vec![&cpd.input],
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

    // Step 2: For each unique expression, find its direct children and
    // record which parent index references which child index.
    // parent_sets[child_idx] = set of parent indices that reference this child
    let mut parent_sets: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); unique.len()];

    // Also process the root expression itself as a parent
    let root_children = direct_children(expr);
    for child in &root_children {
        if let Some(child_idx) = unique.iter().position(|u| u == *child) {
            // Use a special "root" index (unique.len()) as parent
            parent_sets[child_idx].insert(unique.len());
        }
    }

    for (parent_idx, parent) in unique.iter().enumerate() {
        let children = direct_children(parent);
        // In Scala's graph, OptionGet/OptionIsDefined create separate symbols
        // per occurrence (not hash-consed). Count tree occurrences as separate
        // parents so their children see the correct usage count.
        for child in children {
            if let Some(child_idx) = unique.iter().position(|u| u == child) {
                parent_sets[child_idx].insert(parent_idx);
            }
        }
    }

    // Step 3: Return (expr, usage_count) pairs
    unique
        .into_iter()
        .zip(parent_sets)
        .map(|(expr, parents)| (expr, parents.len()))
        .collect()
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

/// Can this expression be extracted as a ValDef?
/// Mirrors the Scala compiler's filters in processAstGraph:
///   !IsContextProperty && !IsInternalDef && !IsConstantDef
fn is_extractable(expr: &Expr) -> bool {
    !matches!(
        expr,
        // IsConstantDef: constants are segregated, not extracted
        Expr::Const(_) | Expr::ConstPlaceholder(_)
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

/// Port of the Scala compiler's `processAstGraph`.
/// Builds a DAG via hash-consing, iterates nodes in DFS schedule order,
/// and extracts multi-use nodes as ValDefs.
fn process_ast_graph(expr: Expr, global_max_id: u32) -> Expr {
    // Step 1-2: DAG usage counts (hash-consing + parent edge counting)
    let dag_usages = count_dag_usages(&expr);

    // Step 3: DFS schedule (children before parents)
    let schedule = dfs_schedule(&expr);

    // Step 4: Iterate schedule, select multi-use nodes for extraction.
    // Unlike the old approach, we do NOT modify the tree during selection.
    // This matches the Scala processAstGraph which marks nodes for ValDef
    // and then builds the result tree from scratch via buildValue.
    let mut env: Vec<(Expr, u32)> = Vec::new();
    let mut next_id = find_max_val_id(&expr).max(global_max_id) + 1;

    for node in &schedule {
        if !is_extractable(node) {
            continue;
        }
        if !is_graph_shared(node) {
            continue;
        }
        let dag_count = dag_usages
            .iter()
            .find(|(e, _)| e == node)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        if dag_count >= 2 {
            // In Scala, && and || wrap the right operand in a ThunkDef,
            // creating a separate hash-consing scope. Expressions that
            // only appear inside &&/|| right arms get separate graph
            // symbols per ThunkDef scope. Only extract if it also appears
            // in a left-arm (main scope) position.
            //
            // This applies to ValUse-dependent expressions (PropertyCall
            // or ByIndex on a ValUse/PropertyCall) and ByIndex on globals
            // (e.g., ByIndex(Outputs, 0) used only in && right arms).
            let needs_scope_check = matches!(node, Expr::PropertyCall(s) if matches!(&*s.expr.obj, Expr::ValUse(_)))
                || matches!(node, Expr::ByIndex(s) if matches!(&*s.expr.input, Expr::ValUse(_) | Expr::PropertyCall(_) | Expr::GlobalVars(_)))
                || matches!(node, Expr::Upcast(uc) if matches!(&*uc.input, Expr::ValUse(_) | Expr::Const(_)));
            if needs_scope_check && !appears_in_main_scope(&expr, node) {
                continue;
            }
            env.push((node.clone(), next_id));
            next_id += 1;
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

    // Wrap in BlockValue
    match result {
        Expr::BlockValue(spanned) => {
            let mut items = val_defs;
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
                items: val_defs,
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
fn appears_in_main_scope(tree: &Expr, target: &Expr) -> bool {
    appears_in_main_scope_inner(tree, target, false)
}

fn appears_in_main_scope_inner(expr: &Expr, target: &Expr, directly_in_thunk: bool) -> bool {
    if expr == target {
        // Found the target. It's in a "left arm position" if we're NOT
        // directly inside a right arm that hasn't been "reset" by a left arm.
        return !directly_in_thunk;
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
            // Left arm: entering a scope's "main" position — reset the thunk flag.
            // In Scala, the left operand is eagerly evaluated in the enclosing scope.
            // Even if we're inside a ThunkDef, the ThunkDef has its own main scope.
            appears_in_main_scope_inner(&s.expr.left, target, false)
                || appears_in_main_scope_inner(&s.expr.right, target, true)
        }
        // In Scala's graph IR, If branches are ThunkDefs (lazy evaluation).
        // The condition is eagerly evaluated (main scope), but true/false
        // branches are separate ThunkDef scopes.
        Expr::If(if_op) => {
            appears_in_main_scope_inner(&if_op.condition, target, directly_in_thunk)
                || appears_in_main_scope_inner(&if_op.true_branch, target, true)
                || appears_in_main_scope_inner(&if_op.false_branch, target, true)
        }
        _ => direct_children(expr)
            .into_iter()
            .any(|child| appears_in_main_scope_inner(child, target, directly_in_thunk)),
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
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            items.iter().map(find_max_val_id).max().unwrap_or(0)
        }
        _ => 0,
    }
}

/// Count how many times `target` appears in `expr`.
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
        Expr::Collection(ergotree_ir::mir::collection::Collection::Exprs { items, .. }) => {
            for item in items {
                count += count_occurrences(item, target);
            }
        }
        Expr::FuncValue(_) => {
            // Don't count inside lambda bodies — they're handled separately
        }
        _ => {}
    }
    count
}

/// Replace all occurrences of `target` with `replacement` in the expression tree.
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
            ergotree_ir::mir::method_call::MethodCall::new(new_obj, s.expr.method.clone(), new_args)
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
        // Leaves and lambda bodies (not traversed for replacement at this level)
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
            ergotree_ir::mir::method_call::MethodCall::new(obj, s.expr.method, args)
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
