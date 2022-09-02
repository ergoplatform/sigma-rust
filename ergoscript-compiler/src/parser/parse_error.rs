use std::fmt;
use text_size::TextRange;

use crate::error::pretty_error_desc;
use crate::lexer::TokenKind;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub expected: Vec<TokenKind>,
    pub found: Option<TokenKind>,
    pub span: TextRange,
}

impl ParseError {
    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.to_string())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: expected ")?;

        let num_expected = self.expected.len();
        let is_first = |idx| idx == 0;
        let is_last = |idx| idx == num_expected - 1;

        for (idx, expected_kind) in self.expected.iter().enumerate() {
            if is_first(idx) {
                write!(f, "{}", expected_kind)?;
            } else if is_last(idx) {
                write!(f, " or {}", expected_kind)?;
            } else {
                write!(f, ", {}", expected_kind)?;
            }
        }

        if let Some(found) = self.found {
            write!(f, ", but found {}", found)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range as StdRange;

    fn check(
        expected: Vec<TokenKind>,
        found: Option<TokenKind>,
        range: StdRange<u32>,
        output: &str,
    ) {
        let error = ParseError {
            expected,
            found,
            span: {
                let start = range.start.into();
                let end = range.end.into();
                TextRange::new(start, end)
            },
        };

        assert_eq!(format!("{}", error), output);
    }

    #[test]
    fn one_expected_did_find() {
        check(
            vec![TokenKind::Equals],
            Some(TokenKind::Ident),
            10..20,
            "error: expected ‘=’, but found identifier",
        );
    }

    #[test]
    fn one_expected_did_not_find() {
        check(vec![TokenKind::RParen], None, 5..6, "error: expected ‘)’");
    }

    #[test]
    fn two_expected_did_find() {
        check(
            vec![TokenKind::Plus, TokenKind::Minus],
            Some(TokenKind::Equals),
            0..1,
            "error: expected ‘+’ or ‘-’, but found ‘=’",
        );
    }

    #[test]
    fn multiple_expected_did_find() {
        check(
            vec![
                TokenKind::IntNumber,
                TokenKind::Ident,
                TokenKind::Minus,
                TokenKind::LParen,
            ],
            Some(TokenKind::ValKw),
            100..105,
            "error: expected number, identifier, ‘-’ or ‘(’, but found ‘val’",
        );
    }
}
