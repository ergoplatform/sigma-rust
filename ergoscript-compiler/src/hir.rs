//! High-level Intermediate Representation
//! Refered as frontend representation in sigmastate

mod rewrite;

use ergotree_ir::types::stype::SType;
pub use rewrite::rewrite;

use super::ast;
use crate::error::pretty_error_desc;
use crate::syntax::SyntaxKind;
use text_size::TextRange;

extern crate derive_more;
use derive_more::From;

pub fn lower(ast: ast::Root) -> Result<Expr, HirLoweringError> {
    // TODO: return error if more than one expr is found
    let first_expr = ast.children().next().unwrap();
    Expr::lower(&first_expr)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: TextRange,
    pub tpe: Option<SType>,
}

#[derive(Debug, PartialEq)]
pub struct HirLoweringError {
    msg: String,
    span: TextRange,
}

impl HirLoweringError {
    pub fn new(msg: String, span: TextRange) -> Self {
        HirLoweringError { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(&source, self.span, &self.msg)
    }
}

impl Expr {
    pub fn lower(expr: &ast::Expr) -> Result<Expr, HirLoweringError> {
        match expr {
            ast::Expr::BinaryExpr(ast) => Ok(Expr {
                kind: Binary::lower(ast)?.into(),
                span: ast.span(),
                tpe: None,
            }),
            ast::Expr::Ident(ast) => ast
                .name()
                .map(|node| Expr {
                    kind: ExprKind::Ident(node.text().to_string()),
                    span: ast.span(),
                    tpe: None,
                })
                .ok_or_else(|| {
                    HirLoweringError::new(format!("Empty Ident.name: {:?}", ast), ast.span())
                }),
            ast::Expr::Literal(ast) => {
                let v = ast.parse().ok_or_else(|| {
                    HirLoweringError::new(
                        format!("Failed to parse Literal from: {:?}", ast),
                        ast.span(),
                    )
                })?;
                let expr = match v {
                    ast::LiteralValue::Int(v) => Expr {
                        kind: Literal::Int(v).into(),
                        span: ast.span(),
                        tpe: Some(SType::SInt),
                    },
                    ast::LiteralValue::Long(v) => Expr {
                        kind: Literal::Long(v).into(),
                        span: ast.span(),
                        tpe: Some(SType::SLong),
                    },
                };
                Ok(expr)
            }
        }
    }

    pub fn debug_tree(&self) -> String {
        let tree = format!("{:#?}", self);
        tree
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Spanned<T: Clone> {
    pub node: T,
    pub span: TextRange,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Binary {
    pub op: Spanned<BinaryOp>,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Binary {
    fn lower(ast: &ast::BinaryExpr) -> Result<Binary, HirLoweringError> {
        // TODO: unwraps -> errors
        let op = match ast.op().unwrap().kind() {
            SyntaxKind::Plus => BinaryOp::Plus,
            SyntaxKind::Minus => BinaryOp::Minus,
            SyntaxKind::Star => BinaryOp::Multiply,
            SyntaxKind::Slash => BinaryOp::Divide,
            _ => {
                return Err(HirLoweringError::new(
                    format!("unknown binary operator: {:?}", ast.op()),
                    ast.op().unwrap().text_range(),
                ))
            }
        };

        let lhs = Expr::lower(&ast.lhs().unwrap());
        let rhs = Expr::lower(&ast.rhs().unwrap());

        Ok(Binary {
            op: Spanned {
                node: op,
                span: ast.op().unwrap().text_range(),
            },
            lhs: Box::new(lhs?),
            rhs: Box::new(rhs?),
        })
    }
}

#[derive(Debug, PartialEq, From, Clone)]
pub enum ExprKind {
    Ident(String),
    Binary(Binary),
    GlobalVars(GlobalVars),
    Literal(Literal),
    // ...
    // Block
    // ValNode
    // Select
    // ApplyTypes
    // MethodCallLike
    // Lambda
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
}

// #[derive(Debug, PartialEq, Clone)]
// pub enum UnaryOp {
//     Neg,
// }

#[derive(Debug, PartialEq, Clone)]
pub enum GlobalVars {
    Height,
}

impl GlobalVars {
    /// Type
    pub fn tpe(&self) -> SType {
        match self {
            GlobalVars::Height => SType::SInt,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(i32),
    Long(i64),
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::compiler::compile_hir;

    fn check(input: &str, expected_tree: expect_test::Expect) {
        let res = compile_hir(input);

        let expected_out = res
            .map(|tree| tree.debug_tree())
            .unwrap_or_else(|e| e.pretty_desc(input));
        expected_tree.assert_eq(&expected_out);
    }

    #[test]
    fn long_literal() {
        check(
            "42L",
            expect![[r#"
            Expr {
                kind: Literal(
                    Long(
                        42,
                    ),
                ),
                span: 0..3,
                tpe: Some(
                    SLong,
                ),
            }"#]],
        );
    }
}
