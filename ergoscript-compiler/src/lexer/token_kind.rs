// Initial version is copied from https://github.com/arzg/eldiro
// Checkout https://arzg.github.io/lang/ for description
use logos::Logos;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Logos)]
pub enum TokenKind {
    #[regex("[ \n]+")]
    Whitespace,

    #[token("def")]
    FnKw,

    #[token("val")]
    ValKw,

    #[token("true")]
    TrueKw,

    #[token("false")]
    FalseKw,

    #[token("if")]
    IfKw,

    #[token("else")]
    ElseKw,

    #[regex("[A-Za-z_][A-Za-z0-9_]*")]
    Ident,

    #[regex("[0-9]+L")]
    LongNumber,

    #[regex("[0-9]+")]
    IntNumber,

    #[regex(r#""[^"]*""#)]
    StringLiteral,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("!")]
    Bang,

    #[token("==")]
    EqEq,

    #[token("!=")]
    NotEq,

    #[token(">=")]
    GtEq,

    #[token("<=")]
    LtEq,

    #[token(">")]
    Gt,

    #[token("<")]
    Lt,

    #[token("=")]
    Equals,

    #[token(".")]
    Dot,

    #[token("=>")]
    Arrow,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[regex("//.*")]
    Comment,

    #[error]
    Error,
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Whitespace => "whitespace",
            Self::FnKw => "'def'",
            Self::ValKw => "'val'",
            Self::TrueKw => "'true'",
            Self::FalseKw => "'false'",
            Self::IfKw => "'if'",
            Self::ElseKw => "'else'",
            Self::Ident => "identifier",
            Self::IntNumber => "number",
            Self::LongNumber => "number",
            Self::StringLiteral => "string",
            Self::Plus => "'+'",
            Self::Minus => "'-'",
            Self::Star => "'*'",
            Self::Slash => "'/'",
            Self::Percent => "'%'",
            Self::And => "'&&'",
            Self::Or => "'||'",
            Self::Bang => "'!'",
            Self::EqEq => "'=='",
            Self::NotEq => "'!='",
            Self::GtEq => "'>='",
            Self::LtEq => "'<='",
            Self::Gt => "'>'",
            Self::Lt => "'<'",
            Self::Equals => "'='",
            Self::Dot => "'.'",
            Self::Arrow => "'=>'",
            Self::Colon => "':'",
            Self::Comma => "','",
            Self::Semicolon => "';'",
            Self::LParen => "'('",
            Self::RParen => "')'",
            Self::LBrace => "'{'",
            Self::RBrace => "'}'",
            Self::LBracket => "'['",
            Self::RBracket => "']'",
            Self::Comment => "comment",
            Self::Error => "an unrecognized token",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::Lexer;
    use super::*;

    fn check(input: &str, kind: TokenKind) {
        let mut lexer = Lexer::new(input);

        let token = lexer.next().unwrap();
        assert_eq!(token.kind, kind);
        assert_eq!(token.text, input);
    }

    #[test]
    fn lex_spaces_and_newlines() {
        check("  \n ", TokenKind::Whitespace);
    }

    #[test]
    fn lex_fn_keyword() {
        check("def", TokenKind::FnKw);
    }

    #[test]
    fn lex_val_keyword() {
        check("val", TokenKind::ValKw);
    }

    #[test]
    fn lex_true_keyword() {
        check("true", TokenKind::TrueKw);
    }

    #[test]
    fn lex_false_keyword() {
        check("false", TokenKind::FalseKw);
    }

    #[test]
    fn lex_if_keyword() {
        check("if", TokenKind::IfKw);
    }

    #[test]
    fn lex_else_keyword() {
        check("else", TokenKind::ElseKw);
    }

    #[test]
    fn lex_alphabetic_identifier() {
        check("abcd", TokenKind::Ident);
    }

    #[test]
    fn lex_alphanumeric_identifier() {
        check("ab123cde456", TokenKind::Ident);
    }

    #[test]
    fn lex_mixed_case_identifier() {
        check("ABCdef", TokenKind::Ident);
    }

    #[test]
    fn lex_single_char_identifier() {
        check("x", TokenKind::Ident);
    }

    #[test]
    fn lex_number() {
        check("123456", TokenKind::IntNumber);
    }

    #[test]
    fn lex_long_number() {
        check("123L", TokenKind::LongNumber);
    }

    #[test]
    fn lex_string_literal() {
        check(r#""hello""#, TokenKind::StringLiteral);
    }

    #[test]
    fn lex_plus() {
        check("+", TokenKind::Plus);
    }

    #[test]
    fn lex_minus() {
        check("-", TokenKind::Minus);
    }

    #[test]
    fn lex_star() {
        check("*", TokenKind::Star);
    }

    #[test]
    fn lex_slash() {
        check("/", TokenKind::Slash);
    }

    #[test]
    fn lex_and() {
        check("&&", TokenKind::And);
    }

    #[test]
    fn lex_or() {
        check("||", TokenKind::Or);
    }

    #[test]
    fn lex_bang() {
        check("!", TokenKind::Bang);
    }

    #[test]
    fn lex_eq_eq() {
        check("==", TokenKind::EqEq);
    }

    #[test]
    fn lex_not_eq() {
        check("!=", TokenKind::NotEq);
    }

    #[test]
    fn lex_gt_eq() {
        check(">=", TokenKind::GtEq);
    }

    #[test]
    fn lex_lt_eq() {
        check("<=", TokenKind::LtEq);
    }

    #[test]
    fn lex_gt() {
        check(">", TokenKind::Gt);
    }

    #[test]
    fn lex_lt() {
        check("<", TokenKind::Lt);
    }

    #[test]
    fn lex_equals() {
        check("=", TokenKind::Equals);
    }

    #[test]
    fn lex_dot() {
        check(".", TokenKind::Dot);
    }

    #[test]
    fn lex_colon() {
        check(":", TokenKind::Colon);
    }

    #[test]
    fn lex_comma() {
        check(",", TokenKind::Comma);
    }

    #[test]
    fn lex_left_parenthesis() {
        check("(", TokenKind::LParen);
    }

    #[test]
    fn lex_right_parenthesis() {
        check(")", TokenKind::RParen);
    }

    #[test]
    fn lex_left_brace() {
        check("{", TokenKind::LBrace);
    }

    #[test]
    fn lex_right_brace() {
        check("}", TokenKind::RBrace);
    }

    #[test]
    fn lex_left_bracket() {
        check("[", TokenKind::LBracket);
    }

    #[test]
    fn lex_right_bracket() {
        check("]", TokenKind::RBracket);
    }

    #[test]
    fn lex_comment() {
        check("// foo", TokenKind::Comment);
    }
}
