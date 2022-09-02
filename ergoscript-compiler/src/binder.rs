use rowan::TextRange;

use crate::error::pretty_error_desc;
use crate::hir;
use crate::hir::Expr;
use crate::hir::ExprKind;
use crate::hir::GlobalVars;
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
        rewrite(expr, &self.env)
    }
}

fn rewrite(expr: Expr, env: &ScriptEnv) -> Result<Expr, BinderError> {
    hir::rewrite(expr, |e| {
        Ok(match &e.kind {
            ExprKind::Ident(ident) => match env.get(ident) {
                Some(_) => todo!(),
                None => match ident.as_ref() {
                    "HEIGHT" => {
                        let v = GlobalVars::Height;
                        let tpe = v.tpe();
                        Some(Expr {
                            kind: v.into(),
                            span: e.span,
                            tpe: tpe.into(),
                        })
                    }
                    _ => None,
                },
            },
            _ => None,
        })
    })
}
