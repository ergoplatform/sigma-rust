use super::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
use text_size::TextRange;

// #[derive(Debug)]
// pub struct Root(SyntaxNode);

// impl Root {
//     pub fn cast(node: SyntaxNode) -> Option<Self> {
//         if node.kind() == SyntaxKind::Root {
//             Some(Self(node))
//         } else {
//             None
//         }
//     }

//     pub fn expr(&self) -> impl Iterator<Item = Stmt> {
//         self.0.children().filter_map(Stmt::cast)
//     }
// }

// #[derive(Debug)]
// pub enum Stmt {
//     VariableDef(VariableDef),
//     Expr(Expr),
// }

// impl Stmt {
//     pub fn cast(node: SyntaxNode) -> Option<Self> {
//         let result = match node.kind() {
//             SyntaxKind::VariableDef => Self::VariableDef(VariableDef(node)),
//             _ => Self::Expr(Expr::cast(node)?),
//         };

//         Some(result)
//     }
// }

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
}

#[derive(Debug)]
pub struct Ident(SyntaxNode);

impl Ident {
    pub fn name(&self) -> Option<SyntaxToken> {
        self.0
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|token| token.kind() == SyntaxKind::Ident)
    }

    pub fn span(&self) -> TextRange {
        self.0.text_range()
    }

    // pub fn value(&self) -> Option<Expr> {
    //     self.0.children().find_map(Expr::cast)
    // }
}

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
    pub fn lhs(&self) -> Option<Expr> {
        self.0.children().find_map(Expr::cast)
    }

    pub fn rhs(&self) -> Option<Expr> {
        self.0.children().filter_map(Expr::cast).nth(1)
    }

    pub fn op(&self) -> Option<SyntaxToken> {
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
    // pub fn cast(node: SyntaxNode) -> Option<Self> {
    //     if node.kind() == SyntaxKind::Literal {
    //         Some(Self(node))
    //     } else {
    //         None
    //     }
    // }

    pub fn parse(&self) -> Option<LiteralValue> {
        let text = self.0.first_token().unwrap().text().to_string();
        if text.ends_with('L') {
            text.strip_suffix("L")
                .unwrap()
                .parse()
                .ok()
                .map(LiteralValue::Long)
        } else {
            text.parse().ok().map(LiteralValue::Int)
        }
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
