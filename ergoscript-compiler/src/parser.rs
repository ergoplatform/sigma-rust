#![allow(dead_code)]

mod event;
mod grammar;
mod marker;
mod parse;
pub(crate) mod parse_error;
mod sink;
mod source;

pub(crate) use parse::parse;

use std::mem;

use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::syntax::SyntaxKind;

use self::event::Event;
use self::marker::Marker;
use self::parse_error::ParseError;
use self::source::Source;

// const RECOVERY_SET: [TokenKind; 1] = [TokenKind::ValKw];

pub struct Parser<'t, 'input> {
    pub source: Source<'t, 'input>,
    pub events: Vec<Event>,
    pub expected_kinds: Vec<TokenKind>,
}

impl<'t, 'input> Parser<'t, 'input> {
    pub fn new(source: Source<'t, 'input>) -> Self {
        Self {
            source,
            events: Vec::new(),
            expected_kinds: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Vec<Event> {
        grammar::root(&mut self);
        self.events
    }

    fn start(&mut self) -> Marker {
        let pos = self.events.len();
        self.events.push(Event::Placeholder);

        Marker::new(pos)
    }

    fn expect(&mut self, kind: TokenKind) {
        if self.at(kind) {
            self.bump();
        } else {
            self.error();
        }
    }

    fn error(&mut self) {
        let current_token = self.source.peek_token();

        let (found, range) = if let Some(Token { kind, range, .. }) = current_token {
            (Some(*kind), *range)
        } else {
            // If we’re at the end of the input we use the range of the very last token in the
            // input.
            (None, self.source.last_token_range().unwrap())
        };

        self.events.push(Event::Error(ParseError {
            expected: mem::take(&mut self.expected_kinds),
            found,
            span: range,
        }));

        // if !self.at_set(&RECOVERY_SET) && !self.at_end() {
        if !self.at_end() {
            let m = self.start();
            self.bump();
            m.complete(self, SyntaxKind::Error);
        }
    }

    fn bump(&mut self) {
        self.expected_kinds.clear();
        self.source.next_token().unwrap();
        self.events.push(Event::AddToken);
    }

    fn at(&mut self, kind: TokenKind) -> bool {
        self.expected_kinds.push(kind);
        self.peek() == Some(kind)
    }

    fn at_set(&mut self, set: &[TokenKind]) -> bool {
        self.peek().map_or(false, |k| set.contains(&k))
    }

    fn at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.source.peek_kind()
    }
}

#[cfg(test)]
fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = parse::parse(input);
    expected_tree.assert_eq(&parse.debug_tree());
}

#[cfg(test)]
mod tests {
    use crate::parser::check;
    use expect_test::expect;

    #[test]
    fn parse_nothing() {
        check("", expect![[r#"Root@0..0"#]]);
    }

    #[test]
    fn parse_whitespace() {
        check(
            "   ",
            expect![[r#"
            Root@0..3
              Whitespace@0..3 "   ""#]],
        );
    }

    #[test]
    fn parse_comment() {
        check(
            "// hello!",
            expect![[r#"
            Root@0..9
              Comment@0..9 "// hello!""#]],
        );
    }

    #[test]
    fn parse_int_literal() {
        check(
            "42",
            expect![[r#"
                Root@0..2
                  IntNumber@0..2
                    IntNumber@0..2 "42""#]],
        );
    }

    #[test]
    fn parse_long_literal() {
        check(
            "42L",
            expect![[r#"
                Root@0..3
                  LongNumber@0..3
                    LongNumber@0..3 "42L""#]],
        );
    }
}
