use text_size::TextRange;

use crate::lexer::Token;
use crate::lexer::TokenKind;

pub struct Source<'t, 'input> {
    pub(crate) tokens: &'t [Token<'input>],
    pub(crate) cursor: usize,
}

impl<'t, 'input> Source<'t, 'input> {
    pub fn new(tokens: &'t [Token<'input>]) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn next_token(&mut self) -> Option<&'t Token<'input>> {
        self.eat_trivia();

        let token = self.tokens.get(self.cursor)?;
        self.cursor += 1;

        Some(token)
    }

    pub fn peek_kind(&mut self) -> Option<TokenKind> {
        self.eat_trivia();
        self.peek_kind_raw()
    }

    pub fn peek_token(&mut self) -> Option<&Token<'_>> {
        self.eat_trivia();
        self.peek_token_raw()
    }

    fn eat_trivia(&mut self) {
        while self.at_trivia() {
            self.cursor += 1;
        }
    }

    fn at_trivia(&self) -> bool {
        self.peek_kind_raw().is_some_and(TokenKind::is_trivia)
    }

    pub fn last_token_range(&self) -> Option<TextRange> {
        self.tokens.last().map(|Token { range, .. }| *range)
    }

    fn peek_kind_raw(&self) -> Option<TokenKind> {
        self.peek_token_raw().map(|Token { kind, .. }| *kind)
    }

    fn peek_token_raw(&self) -> Option<&Token<'_>> {
        self.tokens.get(self.cursor)
    }

    /// Check if there's a newline in the trivia immediately before the current cursor.
    /// Looks backward from the cursor position to find the most recent trivia tokens.
    pub fn newline_before_current(&self) -> bool {
        let mut i = self.cursor;
        while i > 0 {
            i -= 1;
            if let Some(token) = self.tokens.get(i) {
                if token.kind.is_trivia() {
                    if token.text.contains('\n') {
                        return true;
                    }
                } else {
                    break;
                }
            }
        }
        false
    }
}
