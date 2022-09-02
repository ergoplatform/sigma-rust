use crate::error::pretty_error_desc;

use super::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
use text_size::TextRange;

#[derive(Debug, PartialEq, Eq)]
pub struct AstError {
    pub msg: String,
    pub span: TextRange,
}

impl AstError {
    pub fn new(msg: String, span: TextRange) -> Self {
        AstError { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

#[derive(Debug)]
pub struct Root(SyntaxNode);

impl Root {
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Root {
            Some(Self(node))
        } else {
            None
        }
    }

    pub fn children(&self) -> impl Iterator<Item = Expr> {
        self.0.children().filter_map(Expr::cast)
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub struct Ident(SyntaxNode);

impl Ident {
    pub fn name(&self) -> Result<SyntaxToken, AstError> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Ident)
            .ok_or_else(|| AstError::new(format!("Empty Ident.name in: {:?}", self.0), self.span()))
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Expr {
    Ident(Ident),
    BinaryExpr(BinaryExpr),
    Literal(Literal),
    // ParenExpr(ParenExpr),
    // UnaryExpr(UnaryExpr),
}

impl Expr {
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        let result = match node.kind() {
            SyntaxKind::Ident => Self::Ident(Ident(node)),
            SyntaxKind::InfixExpr => Self::BinaryExpr(BinaryExpr(node)),
            SyntaxKind::IntNumber => Self::Literal(Literal(node)),
            SyntaxKind::LongNumber => Self::Literal(Literal(node)),
            // SyntaxKind::ParenExpr => Self::ParenExpr(ParenExpr(node)),
            // SyntaxKind::PrefixExpr => Self::UnaryExpr(UnaryExpr(node)),
            _ => return None,
        };

        Some(result)
    }

    // pub fn span(&self) -> TextRange {
    //     match self {
    //         Expr::Ident(node) => node.0.text_range(),
    //         _ => todo!(),
    //     }
    // }
}

#[derive(Debug)]
pub struct BinaryExpr(SyntaxNode);

impl BinaryExpr {
    pub fn lhs(&self) -> Result<Expr, AstError> {
        self.0.children().find_map(Expr::cast).ok_or_else(|| {
            AstError::new(
                format!("Cannot find lhs in {:?}", self.0.children()),
                self.0.text_range(),
            )
        })
    }

    pub fn rhs(&self) -> Result<Expr, AstError> {
        self.0
            .children()
            .filter_map(Expr::cast)
            .nth(1)
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find rhs in {:?}", self.0.children()),
                    self.0.text_range(),
                )
            })
    }

    pub fn op(&self) -> Result<SyntaxToken, AstError> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| {
                matches!(
                    token.kind(),
                    SyntaxKind::Plus
                        | SyntaxKind::Minus
                        | SyntaxKind::Star
                        | SyntaxKind::Slash
                        | SyntaxKind::And,
                )
            })
            .ok_or_else(|| {
                AstError::new(
                    format!("Cannot find bin op in {:?}", self.0),
                    self.0.text_range(),
                )
            })
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

#[derive(Debug)]
pub enum LiteralValue {
    Int(i32),
    Long(i64),
}

#[derive(Debug)]
pub struct Literal(SyntaxNode);

impl Literal {
    pub fn parse(&self) -> Result<LiteralValue, AstError> {
        let text = self.0.first_token().unwrap().text().to_string();
        if text.ends_with('L') {
            text.strip_suffix('L')
                .unwrap()
                .parse()
                .ok()
                .map(LiteralValue::Long)
        } else {
            text.parse().ok().map(LiteralValue::Int)
        }
        .ok_or_else(|| {
            AstError::new(
                format!("Failed to parse Literal from: {:?}", self.0),
                self.span(),
            )
        })
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }
}

// #[derive(Debug)]
// pub struct ParenExpr(SyntaxNode);

// impl ParenExpr {
//     pub fn expr(&self) -> Option<Expr> {
//         self.0.children().find_map(Expr::cast)
//     }
// }

// #[derive(Debug)]
// pub struct UnaryExpr(SyntaxNode);

// impl UnaryExpr {
//     pub fn expr(&self) -> Option<Expr> {
//         self.0.children().find_map(Expr::cast)
//     }

//     pub fn op(&self) -> Option<SyntaxToken> {
//         self.0
//             .children_with_tokens()
//             .filter_map(SyntaxElement::into_token)
//             .find(|token| token.kind() == SyntaxKind::Minus)
//     }
// }
