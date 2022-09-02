// Initial version is copied from https://github.com/arzg/eldiro
// Checkout https://arzg.github.io/lang/ for description
use super::lexer::TokenKind;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum SyntaxKind {
    Whitespace,
    FnKw,
    ValKw,
    Ident,
    IntNumber,
    LongNumber,
    Plus,
    Minus,
    Star,
    Slash,
    And,
    Equals,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comment,
    Error,
    Root,
    InfixExpr,
    ParenExpr,
    PrefixExpr,
    VariableDef,
}

impl From<TokenKind> for SyntaxKind {
    fn from(token_kind: TokenKind) -> Self {
        match token_kind {
            TokenKind::Whitespace => Self::Whitespace,
            TokenKind::FnKw => Self::FnKw,
            TokenKind::ValKw => Self::ValKw,
            TokenKind::Ident => Self::Ident,
            TokenKind::IntNumber => Self::IntNumber,
            TokenKind::LongNumber => Self::LongNumber,
            TokenKind::Plus => Self::Plus,
            TokenKind::Minus => Self::Minus,
            TokenKind::Star => Self::Star,
            TokenKind::Slash => Self::Slash,
            TokenKind::Equals => Self::Equals,
            TokenKind::LParen => Self::LParen,
            TokenKind::RParen => Self::RParen,
            TokenKind::LBrace => Self::LBrace,
            TokenKind::RBrace => Self::RBrace,
            TokenKind::Comment => Self::Comment,
            TokenKind::Error => Self::Error,
            TokenKind::And => Self::And,
        }
    }
}

pub type SyntaxNode = rowan::SyntaxNode<ErgoScriptLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<ErgoScriptLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<ErgoScriptLanguage>;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ErgoScriptLanguage {}

impl rowan::Language for ErgoScriptLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}
