use std::collections::HashMap;
use std::sync::Arc;

use ergotree_ir::types::stuple::STuple;
use ergotree_ir::types::stype::SType;
use rowan::TextRange;

use crate::error::pretty_error_desc;
use crate::hir;
use crate::hir::Apply;
use crate::hir::Binary;
use crate::hir::Expr;
use crate::hir::ExprKind;
use crate::hir::FieldAccessExpr;
use crate::hir::LambdaExpr;
use crate::hir::ValDef;
use crate::hir::ValUse;

#[derive(Debug, PartialEq, Eq)]
pub struct TypeInferenceError {
    msg: String,
    span: TextRange,
}

impl TypeInferenceError {
    pub fn new(msg: String, span: TextRange) -> Self {
        Self { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

pub fn assign_type(expr: Expr) -> Result<Expr, TypeInferenceError> {
    assign_type_with_scope(expr, &HashMap::new())
}

fn assign_type_with_scope(
    expr: Expr,
    val_types: &HashMap<u32, SType>,
) -> Result<Expr, TypeInferenceError> {
    match &expr.kind {
        ExprKind::Binary(Binary { op, lhs, rhs }) => {
            let l = assign_type_with_scope(*lhs.clone(), val_types)?;
            let r = assign_type_with_scope(*rhs.clone(), val_types)?;
            let tpe = match op.node {
                hir::BinaryOp::Plus
                | hir::BinaryOp::Minus
                | hir::BinaryOp::Multiply
                | hir::BinaryOp::Divide
                | hir::BinaryOp::Modulo => l.tpe.clone(),
                hir::BinaryOp::Eq
                | hir::BinaryOp::Neq
                | hir::BinaryOp::Gt
                | hir::BinaryOp::Lt
                | hir::BinaryOp::Ge
                | hir::BinaryOp::Le => Some(SType::SBoolean),
                hir::BinaryOp::And | hir::BinaryOp::Or => {
                    // SigmaProp-level: if both operands are SSigmaProp, result is SSigmaProp
                    if l.tpe.as_ref() == Some(&SType::SSigmaProp)
                        && r.tpe.as_ref() == Some(&SType::SSigmaProp)
                    {
                        Some(SType::SSigmaProp)
                    } else {
                        Some(SType::SBoolean)
                    }
                }
            };
            Ok(Expr {
                kind: Binary {
                    op: op.clone(),
                    lhs: l.into(),
                    rhs: r.into(),
                }
                .into(),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::Apply(apply) => {
            let typed_func = assign_type_with_scope(*apply.func.clone(), val_types)?;
            let typed_args: Result<Vec<Expr>, TypeInferenceError> = apply
                .args
                .iter()
                .map(|arg| assign_type_with_scope(arg.clone(), val_types))
                .collect();
            let typed_args = typed_args?;
            let tpe = match &typed_func.kind {
                ExprKind::Ident(name) => match name.as_str() {
                    "sigmaProp" => Some(SType::SSigmaProp),
                    "fromBase16" | "fromBase58" => Some(SType::SColl(SType::SByte.into())),
                    "blake2b256" => Some(SType::SColl(SType::SByte.into())),
                    "proveDlog" => Some(SType::SSigmaProp),
                    "atLeast" => Some(SType::SSigmaProp),
                    "longToByteArray" => Some(SType::SColl(SType::SByte.into())),
                    "min" | "max" => typed_args.first().and_then(|a| a.tpe.clone()),
                    "substConstants" => Some(SType::SColl(SType::SByte.into())),
                    "byteArrayToLong" => Some(SType::SLong),
                    "byteArrayToBigInt" => Some(SType::SBigInt),
                    "xor" => Some(SType::SColl(SType::SByte.into())),
                    "xorOf" => Some(SType::SBoolean),
                    "allOf" => Some(SType::SBoolean),
                    "anyOf" => Some(SType::SBoolean),
                    "decodePoint" => Some(SType::SGroupElement),
                    "getVar" => {
                        // getVar[T](n) → SOption(T)
                        apply
                            .type_arg
                            .as_ref()
                            .map(|t| SType::SOption(t.clone().into()))
                    }
                    "Coll" => {
                        // Coll[Type](items) or Coll(items)
                        apply
                            .type_arg
                            .as_ref()
                            .map(|t| SType::SColl(t.clone().into()))
                            .or_else(|| {
                                typed_args
                                    .first()
                                    .and_then(|a| a.tpe.clone())
                                    .map(|t| SType::SColl(t.into()))
                            })
                            .or(Some(SType::SColl(SType::SByte.into())))
                    }
                    _ => None,
                },
                ExprKind::FieldAccess(fa) => {
                    // If the FieldAccess resolves to SColl, this is indexing
                    if let Some(SType::SColl(elem_tpe)) = typed_func.tpe.as_ref() {
                        Some(elem_tpe.as_ref().clone())
                    } else {
                        // Method call on collection object
                        match fa.object.tpe.as_ref() {
                            Some(SType::SAvlTree) => match fa.field.as_str() {
                                "insert" | "update" | "remove" => {
                                    Some(SType::SOption(SType::SAvlTree.into()))
                                }
                                "get" => {
                                    Some(SType::SOption(SType::SColl(SType::SByte.into()).into()))
                                }
                                "getMany" => Some(SType::SColl(
                                    SType::SOption(SType::SColl(SType::SByte.into()).into()).into(),
                                )),
                                "contains" => Some(SType::SBoolean),
                                "updateDigest" | "updateOperations" => Some(SType::SAvlTree),
                                _ => None,
                            },
                            Some(SType::SColl(elem_tpe)) => match fa.field.as_str() {
                                "filter" | "slice" | "append" => {
                                    Some(SType::SColl(elem_tpe.clone()))
                                }
                                "exists" | "forall" => Some(SType::SBoolean),
                                "getOrElse" => Some(elem_tpe.as_ref().clone()),
                                "fold" => {
                                    // Fold: result type = zero (first arg) type
                                    typed_args.first().and_then(|a| a.tpe.clone())
                                }
                                "map" => typed_args.first().and_then(|arg| {
                                    arg.tpe.as_ref().and_then(|t| {
                                        if let SType::SFunc(sf) = t {
                                            Some(SType::SColl(Arc::from(
                                                sf.t_range.as_ref().clone(),
                                            )))
                                        } else {
                                            None
                                        }
                                    })
                                }),
                                "flatMap" => typed_args.first().and_then(|arg| {
                                    // flatMap: (A => Coll[B]) — return type is the mapper's return type directly
                                    arg.tpe.as_ref().and_then(|t| {
                                        if let SType::SFunc(sf) = t {
                                            Some(sf.t_range.as_ref().clone())
                                        } else {
                                            None
                                        }
                                    })
                                }),
                                _ => None,
                            },
                            _ => None,
                        }
                    }
                }
                _ => match typed_func.tpe.as_ref() {
                    Some(SType::SColl(elem_tpe)) => Some(elem_tpe.as_ref().clone()),
                    Some(SType::SFunc(sfunc)) => Some(sfunc.t_range.as_ref().clone()),
                    _ => None,
                },
            };
            Ok(Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(typed_func),
                    args: typed_args,
                    type_arg: apply.type_arg.clone(),
                }),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::ValDef(val_def) => {
            let typed_rhs = assign_type_with_scope(*val_def.rhs.clone(), val_types)?;
            // Prefer inferred type over SAny placeholder
            let tpe = match val_def.tpe.as_ref() {
                Some(SType::SAny) | None => typed_rhs.tpe.clone(),
                Some(explicit) => Some(explicit.clone()),
            };
            Ok(Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: val_def.name.clone(),
                    id: val_def.id,
                    tpe: tpe.clone(),
                    rhs: Box::new(typed_rhs),
                }),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::ValUse(val_use) => {
            // Check if we have a refined type from the val_types scope
            let tpe = if val_use.tpe == SType::SAny {
                val_types
                    .get(&val_use.id)
                    .cloned()
                    .unwrap_or(val_use.tpe.clone())
            } else {
                val_use.tpe.clone()
            };
            Ok(Expr {
                kind: ExprKind::ValUse(ValUse {
                    id: val_use.id,
                    tpe: tpe.clone(),
                }),
                span: expr.span,
                tpe: Some(tpe),
            })
        }
        ExprKind::FieldAccess(fa) => {
            let typed_obj = assign_type_with_scope(*fa.object.clone(), val_types)?;
            let tpe = match typed_obj.tpe.as_ref() {
                Some(SType::SBox) => match fa.field.as_str() {
                    "value" => Some(SType::SLong),
                    "propositionBytes" | "id" | "bytes" | "bytesWithNoRef" | "scriptBytes" => {
                        Some(SType::SColl(SType::SByte.into()))
                    }
                    "creationInfo" => {
                        let tuple =
                            STuple::try_from(vec![SType::SInt, SType::SColl(SType::SByte.into())])
                                .unwrap();
                        Some(SType::STuple(tuple))
                    }
                    "tokens" => {
                        let inner_tuple =
                            STuple::try_from(vec![SType::SColl(SType::SByte.into()), SType::SLong])
                                .unwrap();
                        Some(SType::SColl(SType::STuple(inner_tuple).into()))
                    }
                    field
                        if field.len() >= 2
                            && field.starts_with('R')
                            && field[1..].parse::<u8>().is_ok() =>
                    {
                        let elem_tpe = fa.type_args.first().cloned().unwrap_or(SType::SAny);
                        Some(SType::SOption(elem_tpe.into()))
                    }
                    _ => None,
                },
                Some(SType::SColl(_)) => match fa.field.as_str() {
                    "size" => Some(SType::SInt),
                    _ => None,
                },
                Some(SType::STuple(stuple)) => {
                    if fa.field.starts_with('_') {
                        if let Ok(idx) = fa.field[1..].parse::<usize>() {
                            if idx >= 1 {
                                stuple.items.as_vec().get(idx - 1).cloned()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Some(SType::SOption(inner)) => match fa.field.as_str() {
                    "get" => Some(inner.as_ref().clone()),
                    "isDefined" => Some(SType::SBoolean),
                    _ => None,
                },
                Some(SType::SSigmaProp) => match fa.field.as_str() {
                    "propBytes" => Some(SType::SColl(SType::SByte.into())),
                    _ => None,
                },
                Some(SType::SContext) => match fa.field.as_str() {
                    "dataInputs" => Some(SType::SColl(SType::SBox.into())),
                    "preHeader" => Some(SType::SPreHeader),
                    "selfBoxIndex" => Some(SType::SInt),
                    _ => None,
                },
                Some(SType::SAvlTree) => match fa.field.as_str() {
                    "digest" => Some(SType::SColl(SType::SByte.into())),
                    "enabledOperations" => Some(SType::SByte),
                    "keyLength" => Some(SType::SInt),
                    "isInsertAllowed" | "isUpdateAllowed" | "isRemoveAllowed" => {
                        Some(SType::SBoolean)
                    }
                    _ => None,
                },
                Some(SType::SPreHeader) => match fa.field.as_str() {
                    "timestamp" => Some(SType::SLong),
                    "height" => Some(SType::SInt),
                    "version" => Some(SType::SByte),
                    "minerPk" => Some(SType::SGroupElement),
                    _ => None,
                },
                // Numeric .toLong / .toInt
                Some(SType::SInt) => match fa.field.as_str() {
                    "toLong" => Some(SType::SLong),
                    "toBigInt" => Some(SType::SBigInt),
                    "toByte" => Some(SType::SByte),
                    "toShort" => Some(SType::SShort),
                    _ => None,
                },
                Some(SType::SLong) => match fa.field.as_str() {
                    "toInt" => Some(SType::SInt),
                    "toBigInt" => Some(SType::SBigInt),
                    "toByte" => Some(SType::SByte),
                    "toShort" => Some(SType::SShort),
                    _ => None,
                },
                Some(SType::SByte) => match fa.field.as_str() {
                    "toLong" => Some(SType::SLong),
                    "toInt" => Some(SType::SInt),
                    "toBigInt" => Some(SType::SBigInt),
                    _ => None,
                },
                Some(SType::SShort) => match fa.field.as_str() {
                    "toLong" => Some(SType::SLong),
                    "toInt" => Some(SType::SInt),
                    "toBigInt" => Some(SType::SBigInt),
                    _ => None,
                },
                Some(SType::SBigInt) => match fa.field.as_str() {
                    "toBigInt" => Some(SType::SBigInt), // no-op identity
                    _ => None,
                },
                _ => None,
            };
            Ok(Expr {
                kind: ExprKind::FieldAccess(FieldAccessExpr {
                    object: Box::new(typed_obj),
                    field: fa.field.clone(),
                    type_args: fa.type_args.clone(),
                }),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::Block(items) => {
            let mut typed_items = Vec::new();
            let mut scope = val_types.clone();
            for item in items {
                let typed = assign_type_with_scope(item.clone(), &scope)?;
                // Track ValDef types for subsequent items
                if let ExprKind::ValDef(vd) = &typed.kind {
                    if let Some(id) = vd.id {
                        if let Some(tpe) = &typed.tpe {
                            scope.insert(id, tpe.clone());
                        }
                    }
                }
                typed_items.push(typed);
            }
            let tpe = typed_items.last().and_then(|e| e.tpe.clone());
            Ok(Expr {
                kind: ExprKind::Block(typed_items),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::Lambda(lambda) => {
            let typed_body = assign_type_with_scope(*lambda.body.clone(), val_types)?;
            let t_dom: Vec<SType> = lambda.params.iter().map(|(_, t)| t.clone()).collect();
            let t_range = typed_body.tpe.clone().unwrap_or(SType::SAny);
            let sfunc = ergotree_ir::types::sfunc::SFunc {
                t_dom,
                t_range: t_range.into(),
                tpe_params: vec![],
            };
            Ok(Expr {
                kind: ExprKind::Lambda(LambdaExpr {
                    params: lambda.params.clone(),
                    param_ids: lambda.param_ids.clone(),
                    body: Box::new(typed_body),
                }),
                span: expr.span,
                tpe: Some(SType::SFunc(sfunc)),
            })
        }
        ExprKind::If(if_expr) => {
            let cond = assign_type_with_scope(*if_expr.condition.clone(), val_types)?;
            let then_br = assign_type_with_scope(*if_expr.then_branch.clone(), val_types)?;
            let else_br = assign_type_with_scope(*if_expr.else_branch.clone(), val_types)?;
            let tpe = then_br.tpe.clone();
            Ok(Expr {
                kind: ExprKind::If(hir::IfExprHir {
                    condition: Box::new(cond),
                    then_branch: Box::new(then_br),
                    else_branch: Box::new(else_br),
                }),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::Context => Ok(Expr {
            tpe: Some(SType::SContext),
            ..expr
        }),
        ExprKind::Tuple(items) => {
            let typed_items: Result<Vec<Expr>, TypeInferenceError> = items
                .iter()
                .map(|item| assign_type_with_scope(item.clone(), val_types))
                .collect();
            let typed_items = typed_items?;
            let item_types: Vec<SType> = typed_items.iter().filter_map(|e| e.tpe.clone()).collect();
            let tpe = if item_types.len() == typed_items.len() {
                STuple::try_from(item_types).ok().map(SType::STuple)
            } else {
                None
            };
            Ok(Expr {
                kind: ExprKind::Tuple(typed_items),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::Negation(inner) => {
            let typed = assign_type_with_scope(*inner.clone(), val_types)?;
            let tpe = typed.tpe.clone();
            Ok(Expr {
                kind: ExprKind::Negation(Box::new(typed)),
                span: expr.span,
                tpe,
            })
        }
        ExprKind::LogicalNot(inner) => {
            let typed = assign_type_with_scope(*inner.clone(), val_types)?;
            Ok(Expr {
                kind: ExprKind::LogicalNot(Box::new(typed)),
                span: expr.span,
                tpe: Some(SType::SBoolean),
            })
        }
        // Leaf nodes — pass through
        ExprKind::Ident(_) | ExprKind::GlobalVars(_) | ExprKind::Literal(_) => Ok(expr),
    }
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = super::parser::parse(input);
    let syntax = parse.syntax();
    let root = crate::ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root).unwrap();
    let binder = crate::binder::Binder::new(crate::script_env::ScriptEnv::new());
    let bind = binder.bind(hir).unwrap();
    let res = assign_type(bind).unwrap();
    expected_tree.assert_eq(&res.debug_tree());
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn bin_smoke() {
        check(
            "HEIGHT + HEIGHT",
            expect![[r#"
            Expr {
                kind: Binary(
                    Binary {
                        op: Spanned {
                            node: Plus,
                            span: 7..8,
                        },
                        lhs: Expr {
                            kind: GlobalVars(
                                Height,
                            ),
                            span: 0..7,
                            tpe: Some(
                                SInt,
                            ),
                        },
                        rhs: Expr {
                            kind: GlobalVars(
                                Height,
                            ),
                            span: 9..15,
                            tpe: Some(
                                SInt,
                            ),
                        },
                    },
                ),
                span: 0..15,
                tpe: Some(
                    SInt,
                ),
            }"#]],
        );
    }
}
