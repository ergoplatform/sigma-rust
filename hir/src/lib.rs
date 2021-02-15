//! High-level Intermediate Representation
//! Refered as frontend representation in sigmastate
use syntax::SyntaxKind;
use text_size::TextRange;

extern crate derive_more;
use derive_more::From;

#[derive(Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: TextRange,
}

// TODO: refine: span, expected, found?
pub struct HirError(pub String);

impl Expr {
    pub fn lower(expr: ast::Expr) -> Result<Expr, HirError> {
        match &expr {
            ast::Expr::BinaryExpr(ast) => Ok(Expr {
                kind: Binary::lower(ast)?.into(),
                span: ast.span(),
            }),
            ast::Expr::Ident(ast) => ast
                .name()
                .map(|node| Expr {
                    kind: ExprKind::Ident(node.text().to_string()),
                    span: ast.span(),
                })
                .ok_or_else(|| HirError("".to_string())),
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: TextRange,
}

#[derive(Debug, PartialEq)]
pub struct Binary {
    pub op: Spanned<BinaryOp>,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Binary {
    fn lower(ast: &ast::BinaryExpr) -> Result<Binary, HirError> {
        // TODO: unwraps -> errors
        let op = match ast.op().unwrap().kind() {
            SyntaxKind::Plus => BinaryOp::Add,
            SyntaxKind::Minus => BinaryOp::Sub,
            SyntaxKind::Star => BinaryOp::Mul,
            SyntaxKind::Slash => BinaryOp::Div,
            _ => unreachable!(),
        };

        let lhs = Expr::lower(ast.lhs().unwrap());
        let rhs = Expr::lower(ast.rhs().unwrap());

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

#[derive(Debug, PartialEq, From)]
pub enum ExprKind {
    Ident(String),
    Binary(Binary),
    // ...
    // Block
    // ValNode
    // Select
    // ApplyTypes
    // MethodCallLike
    // Lambda
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Neg,
}
