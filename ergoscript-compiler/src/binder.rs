use crate::hir;
use crate::hir::Expr;
use crate::hir::ExprKind;
use crate::hir::GlobalVars;
use crate::ScriptEnv;

#[derive(Debug, PartialEq)]
pub struct BinderError();

pub struct Binder {
    env: ScriptEnv,
}

impl Binder {
    pub fn new(env: ScriptEnv) -> Self {
        Binder { env }
    }

    pub fn bind(&self, expr: Expr) -> Result<Expr, BinderError> {
        rewrite(expr, &self.env)
    }
}

fn rewrite(expr: Expr, env: &ScriptEnv) -> Result<Expr, BinderError> {
    hir::rewrite(expr, |e| {
        Ok(match &e.kind {
            ExprKind::Ident(ident) => match env.get(ident) {
                Some(_) => todo!(),
                None => match ident.as_ref() {
                    "HEIGHT" => Some(Expr {
                        kind: GlobalVars::Height.into(),
                        span: e.span,
                    }),
                    _ => None,
                },
            },
            _ => None,
        })
    })
}
