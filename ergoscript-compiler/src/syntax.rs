// Initial version is copied from https://github.com/arzg/eldiro
// Checkout https://arzg.github.io/lang/ for description
use super::lexer::TokenKind;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive, PartialOrd, Ord, Hash)]
pub enum SyntaxKind {
    Whitespace,
    FnKw,
    ValKw,
    TrueKw,
    FalseKw,
    IfKw,
    ElseKw,
    Ident,
    IntNumber,
    LongNumber,
    StringLiteral,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    And,
    Or,
    Bang,
    EqEq,
    NotEq,
    GtEq,
    LtEq,
    Gt,
    Lt,
    Equals,
    Dot,
    Arrow,
    Colon,
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comment,
    Error,
    // Composite nodes
    Root,
    InfixExpr,
    ParenExpr,
    PrefixExpr,
    VariableDef,
    BoolLiteral,
    FuncCall,
    BlockExpr,
    FieldAccess,
    Lambda,
    IfExpr,
    TupleExpr,
}

impl From<TokenKind> for SyntaxKind {
    fn from(token_kind: TokenKind) -> Self {
        match token_kind {
            TokenKind::Whitespace => Self::Whitespace,
            TokenKind::FnKw => Self::FnKw,
            TokenKind::ValKw => Self::ValKw,
            TokenKind::TrueKw => Self::TrueKw,
            TokenKind::FalseKw => Self::FalseKw,
            TokenKind::IfKw => Self::IfKw,
            TokenKind::ElseKw => Self::ElseKw,
            TokenKind::Ident => Self::Ident,
            TokenKind::IntNumber => Self::IntNumber,
            TokenKind::LongNumber => Self::LongNumber,
            TokenKind::StringLiteral => Self::StringLiteral,
            TokenKind::Plus => Self::Plus,
            TokenKind::Minus => Self::Minus,
            TokenKind::Star => Self::Star,
            TokenKind::Slash => Self::Slash,
            TokenKind::Percent => Self::Percent,
            TokenKind::And => Self::And,
            TokenKind::Or => Self::Or,
            TokenKind::Bang => Self::Bang,
            TokenKind::EqEq => Self::EqEq,
            TokenKind::NotEq => Self::NotEq,
            TokenKind::GtEq => Self::GtEq,
            TokenKind::LtEq => Self::LtEq,
            TokenKind::Gt => Self::Gt,
            TokenKind::Lt => Self::Lt,
            TokenKind::Equals => Self::Equals,
            TokenKind::Dot => Self::Dot,
            TokenKind::Arrow => Self::Arrow,
            TokenKind::Colon => Self::Colon,
            TokenKind::Comma => Self::Comma,
            TokenKind::Semicolon => Self::Semicolon,
            TokenKind::LParen => Self::LParen,
            TokenKind::RParen => Self::RParen,
            TokenKind::LBrace => Self::LBrace,
            TokenKind::RBrace => Self::RBrace,
            TokenKind::LBracket => Self::LBracket,
            TokenKind::RBracket => Self::RBracket,
            TokenKind::Comment => Self::Comment,
            TokenKind::Error => Self::Error,
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
