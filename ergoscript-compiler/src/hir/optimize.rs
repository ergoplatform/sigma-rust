//! HIR optimization passes: constant folding, val inlining, negation elimination

use std::collections::HashMap;

use super::{
    Apply, Binary, BinaryOp, Expr, ExprKind, FieldAccessExpr, IfExprHir, LambdaExpr, Literal,
    Spanned, ValDef,
};
use ergotree_ir::types::stype::SType;

/// Run all optimization passes in order.
pub fn optimize(expr: Expr) -> Expr {
    let expr = widen_numeric_literals(expr);
    let expr = constant_fold(expr, &mut HashMap::new());
    let expr = inline_single_use_vals(expr);
    eliminate_negation(expr)
}

// ---------------------------------------------------------------------------
// Pass 0: Numeric Literal Widening
// ---------------------------------------------------------------------------

/// Widen inline numeric literals in BinOps to match a wider operand type.
/// Must run BEFORE constant_fold so val-bound constants (ValUse) are still
/// distinguishable from inline literals (Literal). This matches Scala's
/// behavior: inline `1` in `longExpr - 1` becomes Long(1), but val-bound
/// `MaxHistorySize` in `MaxHistorySize * longVal` stays Int with Upcast.
fn widen_numeric_literals(expr: Expr) -> Expr {
    match expr.kind {
        ExprKind::Binary(bin) => {
            let new_lhs = widen_numeric_literals(*bin.lhs);
            let new_rhs = widen_numeric_literals(*bin.rhs);
            let is_numeric_op = matches!(
                bin.op.node,
                BinaryOp::Plus
                    | BinaryOp::Minus
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo
                    | BinaryOp::Gt
                    | BinaryOp::Lt
                    | BinaryOp::Ge
                    | BinaryOp::Le
                    | BinaryOp::Eq
                    | BinaryOp::Neq
            );
            if is_numeric_op {
                let (wl, wr) = widen_binop_literal_pair(new_lhs, new_rhs);
                // Update result type for arithmetic ops (left operand type)
                let tpe = match bin.op.node {
                    BinaryOp::Plus
                    | BinaryOp::Minus
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo => wl.tpe.clone(),
                    _ => expr.tpe,
                };
                Expr {
                    kind: ExprKind::Binary(Binary {
                        op: bin.op,
                        lhs: Box::new(wl),
                        rhs: Box::new(wr),
                    }),
                    tpe,
                    span: expr.span,
                }
            } else {
                Expr {
                    kind: ExprKind::Binary(Binary {
                        op: bin.op,
                        lhs: Box::new(new_lhs),
                        rhs: Box::new(new_rhs),
                    }),
                    ..expr
                }
            }
        }
        ExprKind::Block(items) => Expr {
            kind: ExprKind::Block(items.into_iter().map(widen_numeric_literals).collect()),
            ..expr
        },
        ExprKind::ValDef(vd) => Expr {
            kind: ExprKind::ValDef(ValDef {
                name: vd.name,
                id: vd.id,
                tpe: vd.tpe,
                rhs: Box::new(widen_numeric_literals(*vd.rhs)),
            }),
            ..expr
        },
        ExprKind::Apply(app) => Expr {
            kind: ExprKind::Apply(Apply {
                func: Box::new(widen_numeric_literals(*app.func)),
                args: app.args.into_iter().map(widen_numeric_literals).collect(),
                type_arg: app.type_arg,
            }),
            ..expr
        },
        ExprKind::FieldAccess(fa) => Expr {
            kind: ExprKind::FieldAccess(FieldAccessExpr {
                object: Box::new(widen_numeric_literals(*fa.object)),
                field: fa.field,
                type_args: fa.type_args,
            }),
            ..expr
        },
        ExprKind::If(if_expr) => Expr {
            kind: ExprKind::If(IfExprHir {
                condition: Box::new(widen_numeric_literals(*if_expr.condition)),
                then_branch: Box::new(widen_numeric_literals(*if_expr.then_branch)),
                else_branch: Box::new(widen_numeric_literals(*if_expr.else_branch)),
            }),
            ..expr
        },
        ExprKind::Lambda(lam) => Expr {
            kind: ExprKind::Lambda(LambdaExpr {
                params: lam.params,
                param_ids: lam.param_ids,
                body: Box::new(widen_numeric_literals(*lam.body)),
            }),
            ..expr
        },
        ExprKind::LogicalNot(inner) => Expr {
            kind: ExprKind::LogicalNot(Box::new(widen_numeric_literals(*inner))),
            ..expr
        },
        ExprKind::Negation(inner) => Expr {
            kind: ExprKind::Negation(Box::new(widen_numeric_literals(*inner))),
            ..expr
        },
        ExprKind::BitInversion(inner) => Expr {
            kind: ExprKind::BitInversion(Box::new(widen_numeric_literals(*inner))),
            ..expr
        },
        ExprKind::Tuple(items) => Expr {
            kind: ExprKind::Tuple(items.into_iter().map(widen_numeric_literals).collect()),
            ..expr
        },
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => expr,
    }
}

fn hir_numeric_rank(tpe: &SType) -> Option<u8> {
    match tpe {
        SType::SByte => Some(1),
        SType::SShort => Some(2),
        SType::SInt => Some(3),
        SType::SLong => Some(4),
        SType::SBigInt => Some(5),
        _ => None,
    }
}

/// If exactly one operand is a Literal with a narrower numeric type than the
/// other operand's known type, widen the literal in place.
fn widen_binop_literal_pair(lhs: Expr, rhs: Expr) -> (Expr, Expr) {
    // RHS is a literal, LHS has a wider known type
    if let ExprKind::Literal(ref lit) = rhs.kind {
        if let (Some(l), Some(r)) = (lhs.tpe.as_ref(), rhs.tpe.as_ref()) {
            if let (Some(lr), Some(rr)) = (hir_numeric_rank(l), hir_numeric_rank(r)) {
                if lr > rr {
                    if let Some(widened) = widen_literal(lit, l) {
                        let target = l.clone();
                        return (
                            lhs,
                            Expr {
                                kind: ExprKind::Literal(widened),
                                tpe: Some(target),
                                span: rhs.span,
                            },
                        );
                    }
                }
            }
        }
    }
    // LHS is a literal, RHS has a wider known type
    if let ExprKind::Literal(ref lit) = lhs.kind {
        if let (Some(l), Some(r)) = (lhs.tpe.as_ref(), rhs.tpe.as_ref()) {
            if let (Some(lr), Some(rr)) = (hir_numeric_rank(l), hir_numeric_rank(r)) {
                if rr > lr {
                    if let Some(widened) = widen_literal(lit, r) {
                        let target = r.clone();
                        return (
                            Expr {
                                kind: ExprKind::Literal(widened),
                                tpe: Some(target),
                                span: lhs.span,
                            },
                            rhs,
                        );
                    }
                }
            }
        }
    }
    (lhs, rhs)
}

/// Widen a numeric literal to a wider type.
fn widen_literal(lit: &Literal, target: &SType) -> Option<Literal> {
    match (lit, target) {
        (Literal::Int(v), SType::SLong) => Some(Literal::Long(*v as i64)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Pass 1: Constant Folding
// ---------------------------------------------------------------------------

/// Bottom-up constant folding with scope tracking for val bindings.
fn constant_fold(expr: Expr, scope: &mut HashMap<u32, Literal>) -> Expr {
    match expr.kind {
        ExprKind::Block(items) => {
            let new_items: Vec<Expr> = items
                .into_iter()
                .map(|item| constant_fold(item, scope))
                .collect();
            Expr {
                kind: ExprKind::Block(new_items),
                ..expr
            }
        }
        ExprKind::ValDef(vd) => {
            let new_rhs = constant_fold(*vd.rhs, scope);
            // If RHS folded to a constant, record it for later ValUse substitution
            if let ExprKind::Literal(ref lit) = new_rhs.kind {
                if let Some(id) = vd.id {
                    scope.insert(id, lit.clone());
                }
            }
            Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: vd.name,
                    id: vd.id,
                    tpe: vd.tpe,
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::ValUse(ref vu) => {
            if let Some(lit) = scope.get(&vu.id) {
                Expr {
                    kind: ExprKind::Literal(lit.clone()),
                    tpe: Some(vu.tpe.clone()),
                    ..expr
                }
            } else {
                expr
            }
        }
        ExprKind::Binary(bin) => {
            let new_lhs = constant_fold(*bin.lhs, scope);
            let new_rhs = constant_fold(*bin.rhs, scope);
            // Try to evaluate if both sides are constants
            if let (ExprKind::Literal(ref l), ExprKind::Literal(ref r)) =
                (&new_lhs.kind, &new_rhs.kind)
            {
                if let Some(result) = eval_binary(&bin.op.node, l, r) {
                    let tpe = literal_type(&result);
                    return Expr {
                        kind: ExprKind::Literal(result),
                        tpe: Some(tpe),
                        ..expr
                    };
                }
            }
            Expr {
                kind: ExprKind::Binary(Binary {
                    op: bin.op,
                    lhs: Box::new(new_lhs),
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::LogicalNot(inner) => {
            let new_inner = constant_fold(*inner, scope);
            if let ExprKind::Literal(Literal::Bool(b)) = &new_inner.kind {
                return Expr {
                    kind: ExprKind::Literal(Literal::Bool(!b)),
                    tpe: Some(SType::SBoolean),
                    ..expr
                };
            }
            Expr {
                kind: ExprKind::LogicalNot(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Negation(inner) => {
            let new_inner = constant_fold(*inner, scope);
            match &new_inner.kind {
                ExprKind::Literal(Literal::Int(n)) => Expr {
                    kind: ExprKind::Literal(Literal::Int(-n)),
                    tpe: Some(SType::SInt),
                    ..expr
                },
                ExprKind::Literal(Literal::Long(n)) => Expr {
                    kind: ExprKind::Literal(Literal::Long(-n)),
                    tpe: Some(SType::SLong),
                    ..expr
                },
                _ => Expr {
                    kind: ExprKind::Negation(Box::new(new_inner)),
                    ..expr
                },
            }
        }
        ExprKind::BitInversion(inner) => {
            let new_inner = constant_fold(*inner, scope);
            Expr {
                kind: ExprKind::BitInversion(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Apply(app) => {
            let new_func = constant_fold(*app.func, scope);
            let new_args: Vec<Expr> = app
                .args
                .into_iter()
                .map(|a| constant_fold(a, scope))
                .collect();
            Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(new_func),
                    args: new_args,
                    type_arg: app.type_arg,
                }),
                ..expr
            }
        }
        ExprKind::FieldAccess(fa) => {
            let new_obj = constant_fold(*fa.object, scope);
            Expr {
                kind: ExprKind::FieldAccess(FieldAccessExpr {
                    object: Box::new(new_obj),
                    field: fa.field,
                    type_args: fa.type_args,
                }),
                ..expr
            }
        }
        ExprKind::If(if_expr) => {
            let new_cond = constant_fold(*if_expr.condition, scope);
            let new_then = constant_fold(*if_expr.then_branch, scope);
            let new_else = constant_fold(*if_expr.else_branch, scope);
            Expr {
                kind: ExprKind::If(IfExprHir {
                    condition: Box::new(new_cond),
                    then_branch: Box::new(new_then),
                    else_branch: Box::new(new_else),
                }),
                ..expr
            }
        }
        ExprKind::Lambda(lam) => {
            // Don't propagate outer constant scope into lambda body
            let new_body = constant_fold(*lam.body, &mut HashMap::new());
            Expr {
                kind: ExprKind::Lambda(LambdaExpr {
                    params: lam.params,
                    param_ids: lam.param_ids,
                    body: Box::new(new_body),
                }),
                ..expr
            }
        }
        ExprKind::Tuple(items) => {
            let new_items: Vec<Expr> = items.into_iter().map(|i| constant_fold(i, scope)).collect();
            Expr {
                kind: ExprKind::Tuple(new_items),
                ..expr
            }
        }
        // Leaves: no children to fold
        ExprKind::Literal(_) | ExprKind::Ident(_) | ExprKind::GlobalVars(_) | ExprKind::Context => {
            expr
        }
    }
}

/// Evaluate a binary operation on two constant literals.
/// Returns None if the operation can't be folded (e.g., overflow, type mismatch).
fn eval_binary(op: &BinaryOp, lhs: &Literal, rhs: &Literal) -> Option<Literal> {
    match (op, lhs, rhs) {
        // Long arithmetic
        (BinaryOp::Plus, Literal::Long(a), Literal::Long(b)) => {
            a.checked_add(*b).map(Literal::Long)
        }
        (BinaryOp::Minus, Literal::Long(a), Literal::Long(b)) => {
            a.checked_sub(*b).map(Literal::Long)
        }
        (BinaryOp::Multiply, Literal::Long(a), Literal::Long(b)) => {
            a.checked_mul(*b).map(Literal::Long)
        }
        (BinaryOp::Divide, Literal::Long(a), Literal::Long(b)) => {
            if *b != 0 {
                a.checked_div(*b).map(Literal::Long)
            } else {
                None
            }
        }
        (BinaryOp::Modulo, Literal::Long(a), Literal::Long(b)) => {
            if *b != 0 {
                a.checked_rem(*b).map(Literal::Long)
            } else {
                None
            }
        }
        // Int arithmetic
        (BinaryOp::Plus, Literal::Int(a), Literal::Int(b)) => a.checked_add(*b).map(Literal::Int),
        (BinaryOp::Minus, Literal::Int(a), Literal::Int(b)) => a.checked_sub(*b).map(Literal::Int),
        (BinaryOp::Multiply, Literal::Int(a), Literal::Int(b)) => {
            a.checked_mul(*b).map(Literal::Int)
        }
        (BinaryOp::Divide, Literal::Int(a), Literal::Int(b)) => {
            if *b != 0 {
                a.checked_div(*b).map(Literal::Int)
            } else {
                None
            }
        }
        (BinaryOp::Modulo, Literal::Int(a), Literal::Int(b)) => {
            if *b != 0 {
                a.checked_rem(*b).map(Literal::Int)
            } else {
                None
            }
        }
        // Long comparisons
        (BinaryOp::Gt, Literal::Long(a), Literal::Long(b)) => Some(Literal::Bool(a > b)),
        (BinaryOp::Lt, Literal::Long(a), Literal::Long(b)) => Some(Literal::Bool(a < b)),
        (BinaryOp::Ge, Literal::Long(a), Literal::Long(b)) => Some(Literal::Bool(a >= b)),
        (BinaryOp::Le, Literal::Long(a), Literal::Long(b)) => Some(Literal::Bool(a <= b)),
        (BinaryOp::Eq, Literal::Long(a), Literal::Long(b)) => Some(Literal::Bool(a == b)),
        (BinaryOp::Neq, Literal::Long(a), Literal::Long(b)) => Some(Literal::Bool(a != b)),
        // Int comparisons
        (BinaryOp::Gt, Literal::Int(a), Literal::Int(b)) => Some(Literal::Bool(a > b)),
        (BinaryOp::Lt, Literal::Int(a), Literal::Int(b)) => Some(Literal::Bool(a < b)),
        (BinaryOp::Ge, Literal::Int(a), Literal::Int(b)) => Some(Literal::Bool(a >= b)),
        (BinaryOp::Le, Literal::Int(a), Literal::Int(b)) => Some(Literal::Bool(a <= b)),
        (BinaryOp::Eq, Literal::Int(a), Literal::Int(b)) => Some(Literal::Bool(a == b)),
        (BinaryOp::Neq, Literal::Int(a), Literal::Int(b)) => Some(Literal::Bool(a != b)),
        // Bool comparisons
        (BinaryOp::Eq, Literal::Bool(a), Literal::Bool(b)) => Some(Literal::Bool(a == b)),
        (BinaryOp::Neq, Literal::Bool(a), Literal::Bool(b)) => Some(Literal::Bool(a != b)),
        // Boolean logic
        (BinaryOp::And, Literal::Bool(a), Literal::Bool(b)) => Some(Literal::Bool(*a && *b)),
        (BinaryOp::Or, Literal::Bool(a), Literal::Bool(b)) => Some(Literal::Bool(*a || *b)),
        _ => None,
    }
}

/// Get the SType for a Literal value.
fn literal_type(lit: &Literal) -> SType {
    match lit {
        Literal::Int(_) => SType::SInt,
        Literal::Long(_) => SType::SLong,
        Literal::Bool(_) => SType::SBoolean,
        Literal::String(_) => SType::SColl(SType::SByte.into()),
    }
}

// ---------------------------------------------------------------------------
// Pass 2: Val Inlining (single-use elimination)
// ---------------------------------------------------------------------------

/// Count how many times each ValId is referenced in the expression tree.
fn count_val_uses(expr: &Expr, counts: &mut HashMap<u32, usize>) {
    match &expr.kind {
        ExprKind::ValUse(vu) => {
            *counts.entry(vu.id).or_insert(0) += 1;
        }
        ExprKind::Binary(bin) => {
            count_val_uses(&bin.lhs, counts);
            count_val_uses(&bin.rhs, counts);
        }
        ExprKind::Block(items) => {
            for item in items {
                count_val_uses(item, counts);
            }
        }
        ExprKind::ValDef(vd) => {
            count_val_uses(&vd.rhs, counts);
        }
        ExprKind::Apply(app) => {
            count_val_uses(&app.func, counts);
            for arg in &app.args {
                count_val_uses(arg, counts);
            }
        }
        ExprKind::FieldAccess(fa) => {
            count_val_uses(&fa.object, counts);
        }
        ExprKind::If(if_expr) => {
            count_val_uses(&if_expr.condition, counts);
            count_val_uses(&if_expr.then_branch, counts);
            count_val_uses(&if_expr.else_branch, counts);
        }
        ExprKind::Lambda(lam) => {
            count_val_uses(&lam.body, counts);
        }
        ExprKind::LogicalNot(inner) | ExprKind::Negation(inner) | ExprKind::BitInversion(inner) => {
            count_val_uses(inner, counts);
        }
        ExprKind::Tuple(items) => {
            for item in items {
                count_val_uses(item, counts);
            }
        }
        ExprKind::Literal(_) | ExprKind::Ident(_) | ExprKind::GlobalVars(_) | ExprKind::Context => {
        }
    }
}

/// Replace ValUse nodes with their definitions from the substitution map.
/// Replace `x.size` with `coll.size` when `x = coll.map { ... }`.
/// Map preserves collection length, so SizeOf(Map(coll, f)) == SizeOf(coll).
/// This matches the Scala compiler's graph-level rewrite and reduces the
/// map val's use count, enabling subsequent inlining.
fn rewrite_map_size(expr: Expr, map_colls: &HashMap<u32, Expr>) -> Expr {
    match expr.kind {
        ExprKind::FieldAccess(fa) => {
            if fa.field == "size" {
                if let ExprKind::ValUse(ref vu) = fa.object.kind {
                    if let Some(coll) = map_colls.get(&vu.id) {
                        return Expr {
                            kind: ExprKind::FieldAccess(FieldAccessExpr {
                                object: Box::new(coll.clone()),
                                field: fa.field,
                                type_args: fa.type_args,
                            }),
                            ..expr
                        };
                    }
                }
            }
            let new_obj = rewrite_map_size(*fa.object, map_colls);
            Expr {
                kind: ExprKind::FieldAccess(FieldAccessExpr {
                    object: Box::new(new_obj),
                    field: fa.field,
                    type_args: fa.type_args,
                }),
                ..expr
            }
        }
        ExprKind::Binary(bin) => {
            let new_lhs = rewrite_map_size(*bin.lhs, map_colls);
            let new_rhs = rewrite_map_size(*bin.rhs, map_colls);
            Expr {
                kind: ExprKind::Binary(Binary {
                    op: bin.op,
                    lhs: Box::new(new_lhs),
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::Block(items) => {
            let new_items: Vec<Expr> = items
                .into_iter()
                .map(|i| rewrite_map_size(i, map_colls))
                .collect();
            Expr {
                kind: ExprKind::Block(new_items),
                ..expr
            }
        }
        ExprKind::ValDef(vd) => {
            let new_rhs = rewrite_map_size(*vd.rhs, map_colls);
            Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: vd.name,
                    id: vd.id,
                    tpe: vd.tpe,
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::Apply(app) => {
            let new_func = rewrite_map_size(*app.func, map_colls);
            let new_args: Vec<Expr> = app
                .args
                .into_iter()
                .map(|a| rewrite_map_size(a, map_colls))
                .collect();
            Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(new_func),
                    args: new_args,
                    type_arg: app.type_arg,
                }),
                ..expr
            }
        }
        ExprKind::If(if_expr) => {
            let new_cond = rewrite_map_size(*if_expr.condition, map_colls);
            let new_then = rewrite_map_size(*if_expr.then_branch, map_colls);
            let new_else = rewrite_map_size(*if_expr.else_branch, map_colls);
            Expr {
                kind: ExprKind::If(IfExprHir {
                    condition: Box::new(new_cond),
                    then_branch: Box::new(new_then),
                    else_branch: Box::new(new_else),
                }),
                ..expr
            }
        }
        ExprKind::Lambda(lam) => {
            let new_body = rewrite_map_size(*lam.body, map_colls);
            Expr {
                kind: ExprKind::Lambda(LambdaExpr {
                    params: lam.params,
                    param_ids: lam.param_ids,
                    body: Box::new(new_body),
                }),
                ..expr
            }
        }
        ExprKind::LogicalNot(inner) => {
            let new_inner = rewrite_map_size(*inner, map_colls);
            Expr {
                kind: ExprKind::LogicalNot(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Negation(inner) => {
            let new_inner = rewrite_map_size(*inner, map_colls);
            Expr {
                kind: ExprKind::Negation(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::BitInversion(inner) => {
            let new_inner = rewrite_map_size(*inner, map_colls);
            Expr {
                kind: ExprKind::BitInversion(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Tuple(items) => {
            let new_items: Vec<Expr> = items
                .into_iter()
                .map(|i| rewrite_map_size(i, map_colls))
                .collect();
            Expr {
                kind: ExprKind::Tuple(new_items),
                ..expr
            }
        }
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => expr,
    }
}

/// Replace occurrences of a ValDef's RHS expression with ValUse(id).
/// Skips the ValDef's own RHS to avoid self-reference.
fn substitute_duplicate_rhs(expr: Expr, val_rhs: &[(u32, Expr)]) -> Expr {
    // First check if this entire expression matches any val's RHS
    for (id, rhs) in val_rhs {
        if hir_expr_eq(&expr, rhs) {
            // Don't substitute inside the ValDef's own item
            // (handled by skipping at the Block level in the caller)
            return Expr {
                kind: ExprKind::ValUse(super::ValUse {
                    id: *id,
                    tpe: expr.tpe.clone().unwrap_or(SType::SAny),
                }),
                tpe: expr.tpe.clone(),
                span: expr.span,
            };
        }
    }
    // Recurse into children
    match expr.kind {
        ExprKind::ValDef(vd) => {
            // Don't substitute inside the RHS of the val that owns this expression.
            // But DO substitute other vals' RHS patterns inside this val's RHS.
            let own_id = vd.id;
            let filtered: Vec<(u32, Expr)> = val_rhs
                .iter()
                .filter(|(id, _)| Some(*id) != own_id)
                .cloned()
                .collect();
            let new_rhs = substitute_duplicate_rhs(*vd.rhs, &filtered);
            Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: vd.name,
                    id: vd.id,
                    tpe: vd.tpe,
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::Binary(bin) => Expr {
            kind: ExprKind::Binary(Binary {
                op: bin.op,
                lhs: Box::new(substitute_duplicate_rhs(*bin.lhs, val_rhs)),
                rhs: Box::new(substitute_duplicate_rhs(*bin.rhs, val_rhs)),
            }),
            ..expr
        },
        ExprKind::Apply(app) => Expr {
            kind: ExprKind::Apply(Apply {
                func: Box::new(substitute_duplicate_rhs(*app.func, val_rhs)),
                args: app
                    .args
                    .into_iter()
                    .map(|a| substitute_duplicate_rhs(a, val_rhs))
                    .collect(),
                type_arg: app.type_arg,
            }),
            ..expr
        },
        ExprKind::Block(items) => Expr {
            kind: ExprKind::Block(
                items
                    .into_iter()
                    .map(|i| substitute_duplicate_rhs(i, val_rhs))
                    .collect(),
            ),
            ..expr
        },
        ExprKind::FieldAccess(fa) => Expr {
            kind: ExprKind::FieldAccess(FieldAccessExpr {
                object: Box::new(substitute_duplicate_rhs(*fa.object, val_rhs)),
                field: fa.field,
                type_args: fa.type_args,
            }),
            ..expr
        },
        ExprKind::If(if_expr) => Expr {
            kind: ExprKind::If(IfExprHir {
                condition: Box::new(substitute_duplicate_rhs(*if_expr.condition, val_rhs)),
                then_branch: Box::new(substitute_duplicate_rhs(*if_expr.then_branch, val_rhs)),
                else_branch: Box::new(substitute_duplicate_rhs(*if_expr.else_branch, val_rhs)),
            }),
            ..expr
        },
        ExprKind::Lambda(lam) => Expr {
            kind: ExprKind::Lambda(LambdaExpr {
                params: lam.params,
                param_ids: lam.param_ids,
                body: Box::new(substitute_duplicate_rhs(*lam.body, val_rhs)),
            }),
            ..expr
        },
        ExprKind::LogicalNot(inner) => Expr {
            kind: ExprKind::LogicalNot(Box::new(substitute_duplicate_rhs(*inner, val_rhs))),
            ..expr
        },
        ExprKind::Negation(inner) => Expr {
            kind: ExprKind::Negation(Box::new(substitute_duplicate_rhs(*inner, val_rhs))),
            ..expr
        },
        ExprKind::BitInversion(inner) => Expr {
            kind: ExprKind::BitInversion(Box::new(substitute_duplicate_rhs(*inner, val_rhs))),
            ..expr
        },
        ExprKind::Tuple(items) => Expr {
            kind: ExprKind::Tuple(
                items
                    .into_iter()
                    .map(|i| substitute_duplicate_rhs(i, val_rhs))
                    .collect(),
            ),
            ..expr
        },
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => expr,
    }
}

fn substitute_val_uses(expr: Expr, subs: &HashMap<u32, Expr>) -> Expr {
    match expr.kind {
        ExprKind::ValUse(ref vu) => {
            if let Some(replacement) = subs.get(&vu.id) {
                replacement.clone()
            } else {
                expr
            }
        }
        ExprKind::Binary(bin) => {
            let new_lhs = substitute_val_uses(*bin.lhs, subs);
            let new_rhs = substitute_val_uses(*bin.rhs, subs);
            Expr {
                kind: ExprKind::Binary(Binary {
                    op: bin.op,
                    lhs: Box::new(new_lhs),
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::Block(items) => {
            let new_items: Vec<Expr> = items
                .into_iter()
                .map(|i| substitute_val_uses(i, subs))
                .collect();
            Expr {
                kind: ExprKind::Block(new_items),
                ..expr
            }
        }
        ExprKind::ValDef(vd) => {
            let new_rhs = substitute_val_uses(*vd.rhs, subs);
            Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: vd.name,
                    id: vd.id,
                    tpe: vd.tpe,
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::Apply(app) => {
            let new_func = substitute_val_uses(*app.func, subs);
            let new_args: Vec<Expr> = app
                .args
                .into_iter()
                .map(|a| substitute_val_uses(a, subs))
                .collect();
            Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(new_func),
                    args: new_args,
                    type_arg: app.type_arg,
                }),
                ..expr
            }
        }
        ExprKind::FieldAccess(fa) => {
            let new_obj = substitute_val_uses(*fa.object, subs);
            Expr {
                kind: ExprKind::FieldAccess(FieldAccessExpr {
                    object: Box::new(new_obj),
                    field: fa.field,
                    type_args: fa.type_args,
                }),
                ..expr
            }
        }
        ExprKind::If(if_expr) => {
            let new_cond = substitute_val_uses(*if_expr.condition, subs);
            let new_then = substitute_val_uses(*if_expr.then_branch, subs);
            let new_else = substitute_val_uses(*if_expr.else_branch, subs);
            Expr {
                kind: ExprKind::If(IfExprHir {
                    condition: Box::new(new_cond),
                    then_branch: Box::new(new_then),
                    else_branch: Box::new(new_else),
                }),
                ..expr
            }
        }
        ExprKind::Lambda(lam) => {
            let new_body = substitute_val_uses(*lam.body, subs);
            Expr {
                kind: ExprKind::Lambda(LambdaExpr {
                    params: lam.params,
                    param_ids: lam.param_ids,
                    body: Box::new(new_body),
                }),
                ..expr
            }
        }
        ExprKind::LogicalNot(inner) => {
            let new_inner = substitute_val_uses(*inner, subs);
            Expr {
                kind: ExprKind::LogicalNot(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Negation(inner) => {
            let new_inner = substitute_val_uses(*inner, subs);
            Expr {
                kind: ExprKind::Negation(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::BitInversion(inner) => {
            let new_inner = substitute_val_uses(*inner, subs);
            Expr {
                kind: ExprKind::BitInversion(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Tuple(items) => {
            let new_items: Vec<Expr> = items
                .into_iter()
                .map(|i| substitute_val_uses(i, subs))
                .collect();
            Expr {
                kind: ExprKind::Tuple(new_items),
                ..expr
            }
        }
        ExprKind::Literal(_) | ExprKind::Ident(_) | ExprKind::GlobalVars(_) | ExprKind::Context => {
            expr
        }
    }
}

/// Compare two HIR expressions structurally (ignoring spans).
fn hir_expr_eq(a: &Expr, b: &Expr) -> bool {
    a.tpe == b.tpe && hir_kind_eq(&a.kind, &b.kind)
}

fn hir_kind_eq(a: &ExprKind, b: &ExprKind) -> bool {
    match (a, b) {
        (ExprKind::Literal(la), ExprKind::Literal(lb)) => la == lb,
        (ExprKind::Ident(a), ExprKind::Ident(b)) => a == b,
        (ExprKind::GlobalVars(a), ExprKind::GlobalVars(b)) => a == b,
        (ExprKind::ValUse(a), ExprKind::ValUse(b)) => a == b,
        (ExprKind::Context, ExprKind::Context) => true,
        (ExprKind::Binary(a), ExprKind::Binary(b)) => {
            a.op == b.op && hir_expr_eq(&a.lhs, &b.lhs) && hir_expr_eq(&a.rhs, &b.rhs)
        }
        (ExprKind::FieldAccess(a), ExprKind::FieldAccess(b)) => {
            a.field == b.field && a.type_args == b.type_args && hir_expr_eq(&a.object, &b.object)
        }
        (ExprKind::Apply(a), ExprKind::Apply(b)) => {
            a.type_arg == b.type_arg
                && a.args.len() == b.args.len()
                && hir_expr_eq(&a.func, &b.func)
                && a.args
                    .iter()
                    .zip(b.args.iter())
                    .all(|(x, y)| hir_expr_eq(x, y))
        }
        (ExprKind::If(a), ExprKind::If(b)) => {
            hir_expr_eq(&a.condition, &b.condition)
                && hir_expr_eq(&a.then_branch, &b.then_branch)
                && hir_expr_eq(&a.else_branch, &b.else_branch)
        }
        (ExprKind::Lambda(a), ExprKind::Lambda(b)) => {
            a.params == b.params && hir_expr_eq(&a.body, &b.body)
        }
        (ExprKind::Block(a), ExprKind::Block(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| hir_expr_eq(x, y))
        }
        (ExprKind::ValDef(a), ExprKind::ValDef(b)) => a.id == b.id && hir_expr_eq(&a.rhs, &b.rhs),
        (ExprKind::Tuple(a), ExprKind::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| hir_expr_eq(x, y))
        }
        (ExprKind::LogicalNot(a), ExprKind::LogicalNot(b)) => hir_expr_eq(a, b),
        (ExprKind::Negation(a), ExprKind::Negation(b)) => hir_expr_eq(a, b),
        (ExprKind::BitInversion(a), ExprKind::BitInversion(b)) => hir_expr_eq(a, b),
        _ => false,
    }
}

/// Check if `needle` structurally appears anywhere inside `haystack`
/// (ignoring source spans).
fn hir_expr_contains(haystack: &Expr, needle: &Expr) -> bool {
    if hir_expr_eq(haystack, needle) {
        return true;
    }
    match &haystack.kind {
        ExprKind::Binary(bin) => {
            hir_expr_contains(&bin.lhs, needle) || hir_expr_contains(&bin.rhs, needle)
        }
        ExprKind::Apply(app) => {
            hir_expr_contains(&app.func, needle)
                || app.args.iter().any(|a| hir_expr_contains(a, needle))
        }
        ExprKind::Block(items) => items.iter().any(|i| hir_expr_contains(i, needle)),
        ExprKind::ValDef(vd) => hir_expr_contains(&vd.rhs, needle),
        ExprKind::FieldAccess(fa) => hir_expr_contains(&fa.object, needle),
        ExprKind::If(if_expr) => {
            hir_expr_contains(&if_expr.condition, needle)
                || hir_expr_contains(&if_expr.then_branch, needle)
                || hir_expr_contains(&if_expr.else_branch, needle)
        }
        ExprKind::Lambda(lam) => hir_expr_contains(&lam.body, needle),
        ExprKind::LogicalNot(inner) | ExprKind::Negation(inner) | ExprKind::BitInversion(inner) => {
            hir_expr_contains(inner, needle)
        }
        ExprKind::Tuple(items) => items.iter().any(|i| hir_expr_contains(i, needle)),
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => false,
    }
}

/// Check if `target` appears in `expr` in "clean scope" — i.e., NOT exclusively
/// inside the right arm of a logical &&/|| chain. Mirrors the MIR
/// `appears_in_main_scope` check but operates on HIR expressions.
///
/// In Scala's graph IR, &&/|| right operands are wrapped in ThunkDef (lazy
/// evaluation). Expressions first created inside a ThunkDef get scope-local
/// graph symbols. This function detects whether `target` would be in main
/// scope (not ThunkDef-local) in Scala's graph construction.
fn hir_in_clean_scope(expr: &Expr, target: &Expr) -> bool {
    hir_in_clean_scope_inner(expr, target, false)
}

fn hir_in_clean_scope_inner(expr: &Expr, target: &Expr, in_thunk: bool) -> bool {
    if hir_expr_eq(expr, target) {
        return !in_thunk;
    }
    match &expr.kind {
        ExprKind::Binary(bin) if matches!(bin.op.node, BinaryOp::And | BinaryOp::Or) => {
            // Left arm: reset to main scope (even if we were in a thunk,
            // the left operand is eagerly evaluated in the enclosing scope).
            // Right arm: new ThunkDef scope.
            hir_in_clean_scope_inner(&bin.lhs, target, false)
                || hir_in_clean_scope_inner(&bin.rhs, target, true)
        }
        ExprKind::If(if_expr) => {
            // Condition: inherits scope. Branches: ThunkDef scopes.
            hir_in_clean_scope_inner(&if_expr.condition, target, in_thunk)
                || hir_in_clean_scope_inner(&if_expr.then_branch, target, true)
                || hir_in_clean_scope_inner(&if_expr.else_branch, target, true)
        }
        // All other nodes: recurse with inherited scope flag
        ExprKind::Binary(bin) => {
            hir_in_clean_scope_inner(&bin.lhs, target, in_thunk)
                || hir_in_clean_scope_inner(&bin.rhs, target, in_thunk)
        }
        ExprKind::FieldAccess(fa) => hir_in_clean_scope_inner(&fa.object, target, in_thunk),
        ExprKind::Apply(app) => {
            hir_in_clean_scope_inner(&app.func, target, in_thunk)
                || app
                    .args
                    .iter()
                    .any(|a| hir_in_clean_scope_inner(a, target, in_thunk))
        }
        ExprKind::Block(items) => items
            .iter()
            .any(|i| hir_in_clean_scope_inner(i, target, in_thunk)),
        ExprKind::ValDef(vd) => hir_in_clean_scope_inner(&vd.rhs, target, in_thunk),
        ExprKind::Lambda(lam) => hir_in_clean_scope_inner(&lam.body, target, in_thunk),
        ExprKind::LogicalNot(inner) | ExprKind::Negation(inner) | ExprKind::BitInversion(inner) => {
            hir_in_clean_scope_inner(inner, target, in_thunk)
        }
        ExprKind::Tuple(items) => items
            .iter()
            .any(|i| hir_in_clean_scope_inner(i, target, in_thunk)),
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => false,
    }
}

/// Collect FieldAccess sub-expressions from an HIR expression where the
/// object is a ValUse. These map to PropertyCall(ValUse, ...) in MIR —
/// exactly the pattern that needs scope anchoring in CSE's
/// `needs_scope_check`. GlobalVars-based accesses (SELF.tokens etc.)
/// don't need anchoring since CSE handles them without a scope check.
fn collect_field_accesses<'a>(expr: &'a Expr, result: &mut Vec<&'a Expr>) {
    if let ExprKind::FieldAccess(fa) = &expr.kind {
        if matches!(&fa.object.kind, ExprKind::ValUse(_)) {
            result.push(expr);
        }
    }
    match &expr.kind {
        ExprKind::FieldAccess(fa) => collect_field_accesses(&fa.object, result),
        ExprKind::Binary(bin) => {
            collect_field_accesses(&bin.lhs, result);
            collect_field_accesses(&bin.rhs, result);
        }
        ExprKind::Apply(app) => {
            collect_field_accesses(&app.func, result);
            for a in &app.args {
                collect_field_accesses(a, result);
            }
        }
        ExprKind::Block(items) => {
            for i in items {
                collect_field_accesses(i, result);
            }
        }
        ExprKind::ValDef(vd) => collect_field_accesses(&vd.rhs, result),
        ExprKind::If(if_expr) => {
            collect_field_accesses(&if_expr.condition, result);
            collect_field_accesses(&if_expr.then_branch, result);
            collect_field_accesses(&if_expr.else_branch, result);
        }
        ExprKind::Lambda(lam) => collect_field_accesses(&lam.body, result),
        ExprKind::LogicalNot(inner) | ExprKind::Negation(inner) | ExprKind::BitInversion(inner) => {
            collect_field_accesses(inner, result);
        }
        ExprKind::Tuple(items) => {
            for i in items {
                collect_field_accesses(i, result);
            }
        }
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => {}
    }
}

/// Check if a single-use val should be preserved because its RHS shares a
/// FieldAccess sub-expression with another val's RHS, where both occurrences
/// are in "clean scope" (not inside &&/|| right arms).
///
/// This matches Scala's behavior where user vals serve as scope anchors in
/// the graph IR — their sub-expressions get main-scope graph symbols even
/// though the vals themselves may be inlined into ThunkDef positions later.
fn has_shared_field_in_clean_scope(rhs: &Expr, items: &[Expr], current_idx: usize) -> bool {
    let mut field_accesses = Vec::new();
    collect_field_accesses(rhs, &mut field_accesses);

    // S73 anchor narrowing: precompute ValUse counts across items[] vs counts
    // inside this val's RHS. Used to detect when the FA's object ValDef has
    // uses outside this val (so its ValDef survives at MIR level as an
    // anchor for the chain).
    let mut counts_all: HashMap<u32, usize> = HashMap::new();
    for item in items {
        count_val_uses(item, &mut counts_all);
    }
    let mut counts_in_mine: HashMap<u32, usize> = HashMap::new();
    count_val_uses(rhs, &mut counts_in_mine);

    for fa in &field_accesses {
        // FA must be in clean scope within this val's RHS
        if !hir_in_clean_scope(rhs, fa) {
            continue;
        }
        // Check if this FA appears in another val's RHS, also in clean scope
        let mut shared = false;
        for (j, other) in items.iter().enumerate() {
            if j == current_idx {
                continue;
            }
            if let ExprKind::ValDef(vd) = &other.kind {
                if hir_expr_contains(&vd.rhs, fa) && hir_in_clean_scope(&vd.rhs, fa) {
                    shared = true;
                    break;
                }
            }
        }
        if !shared {
            continue;
        }

        // S73 anchor narrowing: drop the anchor only when the FA is doubly
        // anchored at MIR level — both:
        //   (a) the FA's object ValDef is multi-use (has at least one user
        //       outside this val), so it survives as a graph-IR sym, AND
        //   (b) the FA's smallest extractable wrapper (an Apply or wrapping
        //       FieldAccess) appears in at least one other ValDef's RHS in
        //       clean scope, so MIR CSE will extract that wrapper as its
        //       own ValDef.
        // Both conditions together mean MIR has two redundant anchors for
        // the FA chain; this single-use val is not load-bearing. If only one
        // condition holds, we keep the val to avoid orphaning ValUses.
        let object_anchored = match &fa.kind {
            ExprKind::FieldAccess(fa_inner) => match &fa_inner.object.kind {
                ExprKind::ValUse(vu) => {
                    let total = counts_all.get(&vu.id).copied().unwrap_or(0);
                    let in_mine = counts_in_mine.get(&vu.id).copied().unwrap_or(0);
                    total > in_mine
                }
                _ => false,
            },
            _ => false,
        };

        let wrapper_shared = smallest_extractable_wrapper(rhs, fa)
            .map(|wrapper| {
                items.iter().enumerate().any(|(j, other)| {
                    if j == current_idx {
                        return false;
                    }
                    if let ExprKind::ValDef(vd) = &other.kind {
                        hir_expr_contains(&vd.rhs, &wrapper)
                            && hir_in_clean_scope(&vd.rhs, &wrapper)
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false);

        if object_anchored && wrapper_shared {
            continue;
        }

        return true;
    }
    false
}

/// Walk up from `target` (a FieldAccess sub-expression of `rhs`) to find the
/// smallest enclosing expression that MIR CSE will extract as a ValDef when
/// duplicated. Currently this is the immediate parent expression if the
/// parent is an Apply (function/method call) or a wrapping FieldAccess
/// (e.g. `noteInput.tokens` is the func of `noteInput.tokens(0)`). Returns
/// None if no such parent exists in `rhs`.
fn smallest_extractable_wrapper(rhs: &Expr, target: &Expr) -> Option<Expr> {
    fn find_parent<'a>(e: &'a Expr, target: &Expr) -> Option<&'a Expr> {
        match &e.kind {
            ExprKind::FieldAccess(fa) => {
                if std::ptr::eq(fa.object.as_ref(), target) || hir_expr_eq(&fa.object, target) {
                    return Some(e);
                }
                find_parent(&fa.object, target)
            }
            ExprKind::Apply(app) => {
                if std::ptr::eq(app.func.as_ref(), target) || hir_expr_eq(&app.func, target) {
                    return Some(e);
                }
                if let Some(p) = find_parent(&app.func, target) {
                    return Some(p);
                }
                for a in &app.args {
                    if let Some(p) = find_parent(a, target) {
                        return Some(p);
                    }
                }
                None
            }
            ExprKind::Binary(bin) => {
                find_parent(&bin.lhs, target).or_else(|| find_parent(&bin.rhs, target))
            }
            ExprKind::Block(items) => items.iter().find_map(|i| find_parent(i, target)),
            ExprKind::ValDef(vd) => find_parent(&vd.rhs, target),
            ExprKind::If(if_expr) => find_parent(&if_expr.condition, target)
                .or_else(|| find_parent(&if_expr.then_branch, target))
                .or_else(|| find_parent(&if_expr.else_branch, target)),
            ExprKind::Lambda(lam) => find_parent(&lam.body, target),
            ExprKind::LogicalNot(inner)
            | ExprKind::Negation(inner)
            | ExprKind::BitInversion(inner) => find_parent(inner, target),
            ExprKind::Tuple(items) => items.iter().find_map(|i| find_parent(i, target)),
            ExprKind::Literal(_)
            | ExprKind::Ident(_)
            | ExprKind::GlobalVars(_)
            | ExprKind::ValUse(_)
            | ExprKind::Context => None,
        }
    }
    find_parent(rhs, target).cloned()
}

/// Inline single-use vals, remove dead vals, unwrap trivial blocks.
fn inline_single_use_vals(expr: Expr) -> Expr {
    match expr.kind {
        ExprKind::Block(items) => {
            // First, recursively optimize all items
            let items: Vec<Expr> = items.into_iter().map(inline_single_use_vals).collect();

            if items.len() <= 1 {
                // Single-item block or empty — nothing to inline
                return Expr {
                    kind: ExprKind::Block(items),
                    ..expr
                };
            }

            // Map-size optimization: replace x.size with coll.size when x = coll.map{f}.
            // Map preserves collection length, so this is safe. Matches the Scala
            // compiler's graph-level SizeOf(Map(coll, f)) → SizeOf(coll) rewrite.
            // Reducing use count of map vals enables subsequent inlining.
            let items = {
                let mut map_colls: HashMap<u32, Expr> = HashMap::new();
                for item in &items {
                    if let ExprKind::ValDef(vd) = &item.kind {
                        if let Some(id) = vd.id {
                            if let ExprKind::Apply(app) = &vd.rhs.kind {
                                if let ExprKind::FieldAccess(fa) = &app.func.kind {
                                    if fa.field == "map" && app.args.len() == 1 {
                                        map_colls.insert(id, *fa.object.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                if map_colls.is_empty() {
                    items
                } else {
                    items
                        .into_iter()
                        .map(|item| rewrite_map_size(item, &map_colls))
                        .collect()
                }
            };

            // Multi-pass inlining: dead-code elimination can reduce use counts
            // of other vals, making previously multi-use vals single-use.
            // Repeat until stable. (Scala's graph CSE handles this naturally
            // because it operates on a DAG, not a sequential val list.)
            let mut current_items = items;
            loop {
                let mut counts: HashMap<u32, usize> = HashMap::new();
                for item in &current_items {
                    count_val_uses(item, &mut counts);
                }

                let mut subs: HashMap<u32, Expr> = HashMap::new();
                let mut new_items: Vec<Expr> = Vec::new();

                let items_snapshot = current_items.clone();
                for (idx, item) in current_items.into_iter().enumerate() {
                    if let ExprKind::ValDef(ref vd) = item.kind {
                        if let Some(id) = vd.id {
                            let use_count = counts.get(&id).copied().unwrap_or(0);
                            let rhs = if !subs.is_empty() {
                                substitute_val_uses(*vd.rhs.clone(), &subs)
                            } else {
                                *vd.rhs.clone()
                            };
                            let is_constant = matches!(rhs.kind, ExprKind::Literal(_));
                            if use_count == 0 {
                                continue;
                            } else if is_constant {
                                subs.insert(id, rhs);
                                continue;
                            } else if use_count == 1 {
                                // Before inlining a single-use val, check if its
                                // RHS expression appears elsewhere in the block.
                                // If so, keep the val — inlining would create a
                                // duplicate that CSE can't properly re-extract
                                // (e.g. OptionGet is not graph-shared).
                                let rhs_appears_elsewhere = items_snapshot
                                    .iter()
                                    .enumerate()
                                    .any(|(j, other)| j != idx && hir_expr_contains(other, &rhs));
                                if rhs_appears_elsewhere {
                                    // Keep this val — its RHS appears elsewhere
                                    let rebuilt = Expr {
                                        kind: ExprKind::ValDef(ValDef {
                                            name: vd.name.clone(),
                                            id: vd.id,
                                            tpe: vd.tpe.clone(),
                                            rhs: Box::new(rhs),
                                        }),
                                        ..item
                                    };
                                    new_items.push(rebuilt);
                                    continue;
                                }
                                // Check if this val's RHS shares a FieldAccess
                                // sub-expression with another val's RHS, where
                                // both are in "clean scope" (not inside &&/||
                                // right arms). This preserves the scope-anchor
                                // effect that Scala's user vals provide in the
                                // graph IR — shared sub-expressions stay in main
                                // scope for CSE extraction.
                                if has_shared_field_in_clean_scope(&rhs, &items_snapshot, idx) {
                                    let rebuilt = Expr {
                                        kind: ExprKind::ValDef(ValDef {
                                            name: vd.name.clone(),
                                            id: vd.id,
                                            tpe: vd.tpe.clone(),
                                            rhs: Box::new(rhs),
                                        }),
                                        ..item
                                    };
                                    new_items.push(rebuilt);
                                    continue;
                                }
                                subs.insert(id, rhs);
                                continue;
                            }
                        }
                    }
                    new_items.push(item);
                }

                if !subs.is_empty() {
                    new_items = new_items
                        .into_iter()
                        .map(|item| substitute_val_uses(item, &subs))
                        .collect();
                }

                if subs.is_empty() {
                    // No changes this pass — stable
                    current_items = new_items;
                    break;
                }
                current_items = new_items;
            }

            // Dedup pass: for vals we kept because their RHS appears
            // elsewhere, replace those duplicate RHS occurrences with ValUse.
            // This is needed because CSE can't handle certain expression types
            // (e.g. OptionGet is not graph-shared) and won't substitute them.
            let current_items = {
                // Collect (id, rhs) pairs for all surviving ValDefs
                let val_rhs: Vec<(u32, Expr)> = current_items
                    .iter()
                    .filter_map(|item| {
                        if let ExprKind::ValDef(vd) = &item.kind {
                            vd.id.map(|id| (id, *vd.rhs.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();

                if val_rhs.is_empty() {
                    current_items
                } else {
                    // Dedup by RHS equality: keep only the FIRST val per
                    // unique RHS shape. Without this, two sibling vals with
                    // identical RHS rewrite each other into mutual aliases
                    // (val A's RHS → ValUse(B); val B's RHS → ValUse(A)),
                    // producing a structurally-broken cycle that survives
                    // every CSE pass. Direction-by-first-occurrence breaks
                    // the cycle: only the second val gets rewritten to a
                    // ValUse of the first.
                    let mut deduped: Vec<(u32, Expr)> = Vec::new();
                    let mut alias_map: HashMap<u32, u32> = HashMap::new();
                    for (id, rhs) in val_rhs.into_iter() {
                        if let Some((kept_id, _)) =
                            deduped.iter().find(|(_, r)| hir_expr_eq(&rhs, r))
                        {
                            alias_map.insert(id, *kept_id);
                        } else {
                            deduped.push((id, rhs));
                        }
                    }
                    let after_subst: Vec<Expr> = current_items
                        .into_iter()
                        .map(|item| substitute_duplicate_rhs(item, &deduped))
                        .collect();
                    if alias_map.is_empty() {
                        after_subst
                    } else {
                        // Drop alias ValDefs (those whose RHS got rewritten to
                        // ValUse(kept_id)) and rewrite ValUse(dup_id) → ValUse(kept_id)
                        // in the remaining items. Without this, the dedup pass leaves
                        // trivial alias ValDefs in the schedule that NODE never emits,
                        // producing extra bytes and shifted val IDs downstream.
                        let valuse_subs: HashMap<u32, Expr> = after_subst
                            .iter()
                            .filter_map(|item| {
                                if let ExprKind::ValDef(vd) = &item.kind {
                                    let id = vd.id?;
                                    if alias_map.contains_key(&id) {
                                        if let ExprKind::ValUse(vu) = &vd.rhs.kind {
                                            return Some((
                                                id,
                                                Expr {
                                                    kind: ExprKind::ValUse(super::ValUse {
                                                        id: vu.id,
                                                        tpe: vu.tpe.clone(),
                                                    }),
                                                    tpe: Some(vu.tpe.clone()),
                                                    span: item.span,
                                                },
                                            ));
                                        }
                                    }
                                }
                                None
                            })
                            .collect();
                        after_subst
                            .into_iter()
                            .filter_map(|item| {
                                if let ExprKind::ValDef(vd) = &item.kind {
                                    if let Some(id) = vd.id {
                                        if valuse_subs.contains_key(&id) {
                                            return None;
                                        }
                                    }
                                }
                                Some(substitute_val_uses(item, &valuse_subs))
                            })
                            .collect()
                    }
                }
            };

            // If block reduced to a single expression, unwrap it
            if current_items.len() == 1 {
                current_items.into_iter().next().unwrap()
            } else {
                Expr {
                    kind: ExprKind::Block(current_items),
                    ..expr
                }
            }
        }
        ExprKind::Apply(app) => {
            let new_func = inline_single_use_vals(*app.func);
            let new_args: Vec<Expr> = app.args.into_iter().map(inline_single_use_vals).collect();
            Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(new_func),
                    args: new_args,
                    type_arg: app.type_arg,
                }),
                ..expr
            }
        }
        ExprKind::Binary(bin) => {
            let new_lhs = inline_single_use_vals(*bin.lhs);
            let new_rhs = inline_single_use_vals(*bin.rhs);
            Expr {
                kind: ExprKind::Binary(Binary {
                    op: bin.op,
                    lhs: Box::new(new_lhs),
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::ValDef(vd) => {
            let new_rhs = inline_single_use_vals(*vd.rhs);
            Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: vd.name,
                    id: vd.id,
                    tpe: vd.tpe,
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::FieldAccess(fa) => {
            let new_obj = inline_single_use_vals(*fa.object);
            Expr {
                kind: ExprKind::FieldAccess(FieldAccessExpr {
                    object: Box::new(new_obj),
                    field: fa.field,
                    type_args: fa.type_args,
                }),
                ..expr
            }
        }
        ExprKind::If(if_expr) => {
            let new_cond = inline_single_use_vals(*if_expr.condition);
            let new_then = inline_single_use_vals(*if_expr.then_branch);
            let new_else = inline_single_use_vals(*if_expr.else_branch);
            Expr {
                kind: ExprKind::If(IfExprHir {
                    condition: Box::new(new_cond),
                    then_branch: Box::new(new_then),
                    else_branch: Box::new(new_else),
                }),
                ..expr
            }
        }
        ExprKind::Lambda(lam) => {
            let new_body = inline_single_use_vals(*lam.body);
            Expr {
                kind: ExprKind::Lambda(LambdaExpr {
                    params: lam.params,
                    param_ids: lam.param_ids,
                    body: Box::new(new_body),
                }),
                ..expr
            }
        }
        ExprKind::LogicalNot(inner) => {
            let new_inner = inline_single_use_vals(*inner);
            Expr {
                kind: ExprKind::LogicalNot(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Negation(inner) => {
            let new_inner = inline_single_use_vals(*inner);
            Expr {
                kind: ExprKind::Negation(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::BitInversion(inner) => {
            let new_inner = inline_single_use_vals(*inner);
            Expr {
                kind: ExprKind::BitInversion(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Tuple(items) => {
            let new_items: Vec<Expr> = items.into_iter().map(inline_single_use_vals).collect();
            Expr {
                kind: ExprKind::Tuple(new_items),
                ..expr
            }
        }
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => expr,
    }
}

// ---------------------------------------------------------------------------
// Pass 3: Negation Elimination
// ---------------------------------------------------------------------------

/// Transform `!(a > b)` → `a <= b`, etc.
fn eliminate_negation(expr: Expr) -> Expr {
    match expr.kind {
        ExprKind::LogicalNot(inner) => {
            let inner = eliminate_negation(*inner);
            if let ExprKind::Binary(ref bin) = inner.kind {
                let flipped = match bin.op.node {
                    BinaryOp::Gt => Some(BinaryOp::Le),
                    BinaryOp::Lt => Some(BinaryOp::Ge),
                    BinaryOp::Ge => Some(BinaryOp::Lt),
                    BinaryOp::Le => Some(BinaryOp::Gt),
                    BinaryOp::Eq => Some(BinaryOp::Neq),
                    BinaryOp::Neq => Some(BinaryOp::Eq),
                    _ => None,
                };
                if let Some(new_op) = flipped {
                    return Expr {
                        kind: ExprKind::Binary(Binary {
                            op: Spanned {
                                node: new_op,
                                span: bin.op.span,
                            },
                            lhs: bin.lhs.clone(),
                            rhs: bin.rhs.clone(),
                        }),
                        tpe: Some(SType::SBoolean),
                        ..expr
                    };
                }
            }
            Expr {
                kind: ExprKind::LogicalNot(Box::new(inner)),
                ..expr
            }
        }
        ExprKind::Binary(bin) => {
            let new_lhs = eliminate_negation(*bin.lhs);
            let new_rhs = eliminate_negation(*bin.rhs);
            Expr {
                kind: ExprKind::Binary(Binary {
                    op: bin.op,
                    lhs: Box::new(new_lhs),
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::Block(items) => {
            let new_items: Vec<Expr> = items.into_iter().map(eliminate_negation).collect();
            Expr {
                kind: ExprKind::Block(new_items),
                ..expr
            }
        }
        ExprKind::ValDef(vd) => {
            let new_rhs = eliminate_negation(*vd.rhs);
            Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: vd.name,
                    id: vd.id,
                    tpe: vd.tpe,
                    rhs: Box::new(new_rhs),
                }),
                ..expr
            }
        }
        ExprKind::Apply(app) => {
            let new_func = eliminate_negation(*app.func);
            let new_args: Vec<Expr> = app.args.into_iter().map(eliminate_negation).collect();
            Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(new_func),
                    args: new_args,
                    type_arg: app.type_arg,
                }),
                ..expr
            }
        }
        ExprKind::FieldAccess(fa) => {
            let new_obj = eliminate_negation(*fa.object);
            Expr {
                kind: ExprKind::FieldAccess(FieldAccessExpr {
                    object: Box::new(new_obj),
                    field: fa.field,
                    type_args: fa.type_args,
                }),
                ..expr
            }
        }
        ExprKind::If(if_expr) => {
            let new_cond = eliminate_negation(*if_expr.condition);
            let new_then = eliminate_negation(*if_expr.then_branch);
            let new_else = eliminate_negation(*if_expr.else_branch);
            Expr {
                kind: ExprKind::If(IfExprHir {
                    condition: Box::new(new_cond),
                    then_branch: Box::new(new_then),
                    else_branch: Box::new(new_else),
                }),
                ..expr
            }
        }
        ExprKind::Lambda(lam) => {
            let new_body = eliminate_negation(*lam.body);
            Expr {
                kind: ExprKind::Lambda(LambdaExpr {
                    params: lam.params,
                    param_ids: lam.param_ids,
                    body: Box::new(new_body),
                }),
                ..expr
            }
        }
        ExprKind::Negation(inner) => {
            let new_inner = eliminate_negation(*inner);
            Expr {
                kind: ExprKind::Negation(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::BitInversion(inner) => {
            let new_inner = eliminate_negation(*inner);
            Expr {
                kind: ExprKind::BitInversion(Box::new(new_inner)),
                ..expr
            }
        }
        ExprKind::Tuple(items) => {
            let new_items: Vec<Expr> = items.into_iter().map(eliminate_negation).collect();
            Expr {
                kind: ExprKind::Tuple(new_items),
                ..expr
            }
        }
        ExprKind::Literal(_)
        | ExprKind::Ident(_)
        | ExprKind::GlobalVars(_)
        | ExprKind::ValUse(_)
        | ExprKind::Context => expr,
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::compile;
    use crate::script_env::ScriptEnv;
    use ergotree_ir::serialization::SigmaSerializable;

    fn compile_hex(source: &str) -> String {
        let tree = compile(source, ScriptEnv::new()).unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn test_constant_fold_val_comparison() {
        // { val x = 5L; sigmaProp(x > 0L) } → sigmaProp(true)
        let hex = compile_hex("{ val x = 5L; sigmaProp(x > 0L) }");
        assert_eq!(hex, "10010101d17300");
    }

    #[test]
    fn test_constant_fold_bool_not() {
        // { val y = true; sigmaProp(!y) } → sigmaProp(false)
        let hex = compile_hex("{ val y = true; sigmaProp(!y) }");
        assert_eq!(hex, "10010100d17300");
    }

    #[test]
    fn test_negation_elimination() {
        // { sigmaProp(!(HEIGHT > 100)) } → sigmaProp(HEIGHT <= 100)
        let hex = compile_hex("{ sigmaProp(!(HEIGHT > 100)) }");
        assert_eq!(hex, "100104c801d190a37300");
    }

    #[test]
    fn test_inline_single_use_register() {
        // { val x = SELF.R5[Coll[Byte]].get; sigmaProp(x.size > 0) }
        let hex = compile_hex("{ val x = SELF.R5[Coll[Byte]].get; sigmaProp(x.size > 0) }");
        assert_eq!(hex, "10010400d191b1e4c6a7050e7300");
    }

    #[test]
    fn test_inline_single_use_value() {
        // { val x = SELF.value; sigmaProp(x > 1000000L) }
        let hex = compile_hex("{ val x = SELF.value; sigmaProp(x > 1000000L) }");
        assert_eq!(hex, "10010580897ad191c1a77300");
    }

    #[test]
    fn test_fold_and_inline_chain() {
        // { val fee = 1000000L; val minVal = fee + 1000000L; sigmaProp(SELF.value > minVal) }
        let hex = compile_hex(
            "{ val fee = 1000000L; val minVal = fee + 1000000L; sigmaProp(SELF.value > minVal) }",
        );
        assert_eq!(hex, "1001058092f401d191c1a77300");
    }

    #[test]
    fn test_multi_use_val_kept() {
        // Multi-use val should NOT be inlined
        let hex = compile_hex("{ val x = SELF.value; sigmaProp(x > 0L && x < 1000000L) }");
        assert_eq!(hex, "100205000580897ad801d601c1a7d1ed91720173008f72017301");
    }

    #[test]
    fn test_nested_single_use_chain() {
        // All single-use vals inlined, arithmetic folded where possible
        let hex = compile_hex(
            "{ val a = SELF.R4[Long].get; val b = a + 1L; val c = b * 2L; sigmaProp(c > 0L) }",
        );
        assert_eq!(hex, "1003050205040500d1919c9ae4c6a70405730073017302");
    }

    #[test]
    fn test_is_defined_val_inlined() {
        // Single-use val inlined
        let hex = compile_hex("{ val x = SELF.R4[Long]; sigmaProp(x.isDefined) }");
        assert_eq!(hex, "1000d1e6c6a70405");
    }

    #[test]
    fn test_filter_tokens_cse() {
        // CSE inside the lambda: b.tokens appears twice → lifted to ValDef
        let hex = compile_hex(
            r#"{ val found = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == SELF.tokens(0)._1 }; sigmaProp(found.size == 1) }"#,
        );
        assert_eq!(hex, "10040400040004000402d193b1b5a4d9010163d801d603db63087201ed91b172037300938cb27203730100018cb2db6308a7730200017303");
    }

    #[test]
    fn test_cse_self_tokens_twice() {
        // SELF.tokens appears 2x → CSE lifts to ValDef
        let hex = compile_hex("{ sigmaProp(SELF.tokens.size > 0 && SELF.tokens(0)._2 == 1L) }");
        assert_eq!(
            hex,
            "1003040004000502d801d601db6308a7d1ed91b172017300938cb27201730100027302"
        );
    }

    #[test]
    fn test_cse_outputs0_twice() {
        // OUTPUTS(0) appears 2x → CSE lifts to ValDef
        let hex = compile_hex("{ sigmaProp(OUTPUTS(0).value > SELF.value && OUTPUTS(0).propositionBytes == SELF.propositionBytes) }");
        assert_eq!(
            hex,
            "10010400d801d601b2a5730000d1ed91c17201c1a793c27201c2a7"
        );
    }

    #[test]
    fn test_cse_self_value_twice() {
        // SELF.value appears 2x → CSE lifts to ValDef
        let hex = compile_hex("{ sigmaProp(SELF.value > 0L && SELF.value < 100L) }");
        assert_eq!(hex, "1002050005c801d801d601c1a7d1ed91720173008f72017301");
    }

    #[test]
    fn test_cse_lambda_b_tokens() {
        // b.tokens appears 2x inside exists lambda → CSE lifts inside lambda
        let hex = compile_hex(
            "{ sigmaProp(INPUTS.exists { (b: Box) => b.tokens.size > 0 && b.tokens(0)._2 > 0L }) }",
        );
        assert_eq!(
            hex,
            "1003040004000500d1aea4d9010163d801d603db63087201ed91b172037300918cb27203730100027302"
        );
    }
}
