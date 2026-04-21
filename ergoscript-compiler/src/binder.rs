use std::collections::HashMap;

use ergotree_ir::types::stype::SType;
use rowan::TextRange;

use crate::error::pretty_error_desc;
use crate::hir;
use crate::hir::Expr;
use crate::hir::ExprKind;
use crate::hir::Apply;
use crate::hir::GlobalVars;
use crate::hir::ValDef;
use crate::hir::ValUse;
use crate::script_env::ScriptEnv;

#[derive(Debug, PartialEq, Eq)]
pub struct BinderError {
    msg: String,
    span: TextRange,
}

impl BinderError {
    pub fn new(msg: String, span: TextRange) -> Self {
        Self { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

pub struct Binder {
    env: ScriptEnv,
}

impl Binder {
    pub fn new(env: ScriptEnv) -> Self {
        Binder { env }
    }

    pub fn bind(&self, expr: Expr) -> Result<Expr, BinderError> {
        let mut scope = Scope::new();
        bind_expr(expr, &self.env, &mut scope)
    }
}

struct Scope {
    /// Maps variable name → (ValId, SType)
    vars: HashMap<String, (u32, SType)>,
    /// Next available ValId
    next_id: u32,
}

impl Scope {
    fn new() -> Self {
        Scope {
            vars: HashMap::new(),
            next_id: 1, // Match Scala compiler's 1-based ValIds
        }
    }

    fn define(&mut self, name: String, tpe: SType) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.vars.insert(name, (id, tpe));
        id
    }

    fn lookup(&self, name: &str) -> Option<(u32, SType)> {
        self.vars.get(name).cloned()
    }
}

fn bind_expr(expr: Expr, env: &ScriptEnv, scope: &mut Scope) -> Result<Expr, BinderError> {
    match &expr.kind {
        ExprKind::Ident(ident) => {
            // Check scope first (val bindings)
            if let Some((id, tpe)) = scope.lookup(ident) {
                return Ok(Expr {
                    kind: ExprKind::ValUse(ValUse { id, tpe }),
                    span: expr.span,
                    tpe: expr.tpe.clone(),
                });
            }
            // Check script env
            if env.get(ident).is_some() {
                todo!("ScriptEnv variable binding");
            }
            // Check built-in globals
            let global = match ident.as_ref() {
                "HEIGHT" => Some(GlobalVars::Height),
                "SELF" => Some(GlobalVars::SelfBox),
                "INPUTS" => Some(GlobalVars::Inputs),
                "OUTPUTS" => Some(GlobalVars::Outputs),
                "CONTEXT" => {
                    return Ok(Expr {
                        kind: ExprKind::Context,
                        span: expr.span,
                        tpe: Some(SType::SContext),
                    });
                }
                _ => None,
            };
            if let Some(v) = global {
                let tpe = v.tpe();
                Ok(Expr {
                    kind: v.into(),
                    span: expr.span,
                    tpe: tpe.into(),
                })
            } else {
                Ok(expr)
            }
        }
        ExprKind::Binary(binary) => {
            let bound_lhs = bind_expr(*binary.lhs.clone(), env, scope)?;
            let bound_rhs = bind_expr(*binary.rhs.clone(), env, scope)?;
            Ok(Expr {
                kind: hir::Binary {
                    op: binary.op.clone(),
                    lhs: Box::new(bound_lhs),
                    rhs: Box::new(bound_rhs),
                }
                .into(),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::Apply(apply) => {
            let bound_func = bind_expr(*apply.func.clone(), env, scope)?;
            let bound_args: Result<Vec<Expr>, BinderError> = apply
                .args
                .iter()
                .map(|arg| bind_expr(arg.clone(), env, scope))
                .collect();
            Ok(Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(bound_func),
                    args: bound_args?,
                    type_arg: apply.type_arg.clone(),
                }),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::Block(items) => {
            let mut bound_items = Vec::new();
            for item in items {
                bound_items.push(bind_expr(item.clone(), env, scope)?);
            }
            Ok(Expr {
                kind: ExprKind::Block(bound_items),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::ValDef(val_def) => {
            let bound_rhs = bind_expr(*val_def.rhs.clone(), env, scope)?;
            // Determine type: explicit annotation or inferred from RHS
            let tpe = val_def
                .tpe
                .clone()
                .or_else(|| bound_rhs.tpe.clone())
                .unwrap_or(SType::SAny);
            let id = scope.define(val_def.name.clone(), tpe.clone());
            Ok(Expr {
                kind: ExprKind::ValDef(ValDef {
                    name: val_def.name.clone(),
                    id: Some(id),
                    tpe: Some(tpe),
                    rhs: Box::new(bound_rhs),
                }),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::Lambda(lambda) => {
            // Create a new scope for lambda params
            // Note: we need to preserve the outer scope's next_id
            let saved_next_id = scope.next_id;
            let saved_vars = scope.vars.clone();
            // Add lambda params to scope
            let mut param_ids = Vec::new();
            for (name, tpe) in &lambda.params {
                let id = scope.define(name.clone(), tpe.clone());
                param_ids.push((name.clone(), id, tpe.clone()));
            }
            // Bind body with extended scope
            let bound_body = bind_expr(*lambda.body.clone(), env, scope)?;
            // Restore outer scope
            scope.vars = saved_vars;
            // Don't restore next_id — IDs must be globally unique
            let ids: Vec<u32> = param_ids.iter().map(|(_, id, _)| *id).collect();
            Ok(Expr {
                kind: ExprKind::Lambda(hir::LambdaExpr {
                    params: lambda.params.clone(),
                    param_ids: ids,
                    body: Box::new(bound_body),
                }),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::If(if_expr) => {
            let cond = bind_expr(*if_expr.condition.clone(), env, scope)?;
            let then_br = bind_expr(*if_expr.then_branch.clone(), env, scope)?;
            let else_br = bind_expr(*if_expr.else_branch.clone(), env, scope)?;
            Ok(Expr {
                kind: ExprKind::If(hir::IfExprHir {
                    condition: Box::new(cond),
                    then_branch: Box::new(then_br),
                    else_branch: Box::new(else_br),
                }),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::FieldAccess(fa) => {
            let bound_obj = bind_expr(*fa.object.clone(), env, scope)?;
            Ok(Expr {
                kind: ExprKind::FieldAccess(hir::FieldAccessExpr {
                    object: Box::new(bound_obj),
                    field: fa.field.clone(),
                    type_args: fa.type_args.clone(),
                }),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::Tuple(items) => {
            let bound: Result<Vec<Expr>, BinderError> = items
                .iter()
                .map(|item| bind_expr(item.clone(), env, scope))
                .collect();
            Ok(Expr {
                kind: ExprKind::Tuple(bound?),
                span: expr.span,
                tpe: expr.tpe.clone(),
            })
        }
        ExprKind::Negation(inner) => {
            let bound = bind_expr(*inner.clone(), env, scope)?;
            Ok(Expr { kind: ExprKind::Negation(Box::new(bound)), span: expr.span, tpe: expr.tpe.clone() })
        }
        ExprKind::LogicalNot(inner) => {
            let bound = bind_expr(*inner.clone(), env, scope)?;
            Ok(Expr { kind: ExprKind::LogicalNot(Box::new(bound)), span: expr.span, tpe: expr.tpe.clone() })
        }
        ExprKind::GlobalVars(_) | ExprKind::Literal(_) | ExprKind::ValUse(_)
        | ExprKind::Context => Ok(expr),
    }
}
