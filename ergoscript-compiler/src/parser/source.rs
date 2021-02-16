use text_size::TextRange;

use crate::lexer::Token;
use crate::lexer::TokenKind;

pub struct Source<'t, 'input> {
    tokens: &'t [Token<'input>],
    cursor: usize,
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

    pub fn peek_token(&mut self) -> Option<&Token> {
        self.eat_trivia();
        self.peek_token_raw()
    }

    fn eat_trivia(&mut self) {
        while self.at_trivia() {
            self.cursor += 1;
        }
    }

    fn at_trivia(&self) -> bool {
        self.peek_kind_raw().map_or(false, TokenKind::is_trivia)
    }

    pub fn last_token_range(&self) -> Option<TextRange> {
        self.tokens.last().map(|Token { range, .. }| *range)
    }

    fn peek_kind_raw(&self) -> Option<TokenKind> {
        self.peek_token_raw().map(|Token { kind, .. }| *kind)
    }

    fn peek_token_raw(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }
}
