use ergotree_ir::mir::bin_op::ArithOp;
use ergotree_ir::mir::bin_op::BinOp;
use ergotree_ir::mir::bin_op::BinOpKind;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::global_vars::GlobalVars;
use hir::BinaryOp;
use rowan::TextRange;

use crate::error::pretty_error_desc;
use crate::hir;

#[derive(Debug, PartialEq, Eq)]
pub struct MirLoweringError {
    msg: String,
    span: TextRange,
}

impl MirLoweringError {
    pub fn new(msg: String, span: TextRange) -> Self {
        Self { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

pub fn lower(hir_expr: hir::Expr) -> Result<Expr, MirLoweringError> {
    let mir: Expr = match &hir_expr.kind {
        hir::ExprKind::GlobalVars(hir) => match hir {
            hir::GlobalVars::Height => GlobalVars::Height.into(),
        },
        hir::ExprKind::Ident(_) => {
            return Err(MirLoweringError::new(
                format!("MIR error: Unresolved Ident {0:?}", hir_expr),
                hir_expr.span,
            ))
        }
        hir::ExprKind::Binary(hir) => {
            let l = lower(*hir.lhs.clone())?;
            let r = lower(*hir.rhs.clone())?;
            BinOp {
                kind: hir.op.node.clone().into(),
                left: l.into(),
                right: r.into(),
            }
            .into()
        }
        hir::ExprKind::Literal(hir) => {
            let constant: Constant = match *hir {
                hir::Literal::Int(v) => v.into(),
                hir::Literal::Long(v) => v.into(),
            };
            constant.into()
        }
    };
    let hir_tpe = hir_expr.tpe.clone().ok_or_else(|| {
        MirLoweringError::new(
            format!("MIR error: missing tpe for HIR: {0:?}", hir_expr),
            hir_expr.span,
        )
    })?;
    if mir.tpe() == hir_tpe {
        Ok(mir)
    } else {
        Err(MirLoweringError::new(
            format!(
                "MIR error: lowered MIR type != HIR type ({0:?} != {1:?})",
                mir.tpe(),
                hir_expr.tpe
            ),
            hir_expr.span,
        ))
    }
}

impl From<hir::BinaryOp> for BinOpKind {
    fn from(op: hir::BinaryOp) -> Self {
        match op {
            BinaryOp::Plus => ArithOp::Plus.into(),
            BinaryOp::Minus => ArithOp::Minus.into(),
            BinaryOp::Multiply => ArithOp::Multiply.into(),
            BinaryOp::Divide => ArithOp::Divide.into(),
        }
    }
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = crate::parser::parse(input);
    let syntax = parse.syntax();
    let root = crate::ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root).unwrap();
    let binder = crate::binder::Binder::new(crate::script_env::ScriptEnv::new());
    let bind = binder.bind(hir).unwrap();
    let typed = crate::type_infer::assign_type(bind).unwrap();
    let res = lower(typed).unwrap();
    expected_tree.assert_eq(&res.debug_tree());
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn bin_smoke() {
        check(
            "HEIGHT + HEIGHT",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Arith(
                                Plus,
                            ),
                            left: GlobalVars(
                                Height,
                            ),
                            right: GlobalVars(
                                Height,
                            ),
                        },
                    },
                )"#]],
        )
    }

    #[test]
    fn literal_int() {
        check(
            "42",
            expect![[r#"
                Const(
                    "42: SInt",
                )"#]],
        );
    }

    #[test]
    fn literal_long() {
        check(
            "42L",
            expect![[r#"
                Const(
                    "42: SLong",
                )"#]],
        );
    }

    #[test]
    fn bin_numeric_int() {
        check(
            "4+2",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Arith(
                                Plus,
                            ),
                            left: Const(
                                "4: SInt",
                            ),
                            right: Const(
                                "2: SInt",
                            ),
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn bin_numeric_long() {
        check(
            "4L+2L",
            expect![[r#"
                BinOp(
                    Spanned {
                        source_span: SourceSpan {
                            offset: 0,
                            length: 0,
                        },
                        expr: BinOp {
                            kind: Arith(
                                Plus,
                            ),
                            left: Const(
                                "4: SLong",
                            ),
                            right: Const(
                                "2: SLong",
                            ),
                        },
                    },
                )"#]],
        );
    }
}
