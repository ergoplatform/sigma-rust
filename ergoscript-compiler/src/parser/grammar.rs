mod expr;
mod stmt;

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

use super::marker::CompletedMarker;

pub fn root(p: &mut Parser) -> CompletedMarker {
    let m = p.start();

    while !p.at_end() {
        stmt::stmt(p);
    }

    m.complete(p, SyntaxKind::Root)
}

#[cfg(test)]
mod tests {
    use crate::parser::check;
    use expect_test::expect;

    // #[test]
    // fn parse_multiple_statements() {
    //     check("val a = 1;\na", expect![[]]);
    // }

    #[test]
    fn parse_ident() {
        check("HEIGHT", expect![[]]);
    }
}
