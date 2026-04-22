use super::*;

pub(super) fn stmt(p: &mut Parser) -> Option<CompletedMarker> {
    if p.at(TokenKind::ValKw) {
        Some(variable_def(p))
    } else if p.at(TokenKind::FnKw) {
        Some(function_def(p))
    } else {
        expr::expr(p)
    }
}

/// Parse `def name(params): ReturnType = body` as a VariableDef with lambda RHS.
/// ErgoScript `def` is syntactic sugar for `val name = { (params) => body }`.
fn function_def(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::FnKw));
    let m = p.start();
    // Rewrite: emit ValKw token position but consume FnKw
    p.bump(); // eat 'def'

    p.expect(TokenKind::Ident); // function name

    // Parse parameter list: (param1: Type1, param2: Type2, ...)
    p.expect(TokenKind::LParen);
    if !p.at(TokenKind::RParen) {
        loop {
            p.expect(TokenKind::Ident); // param name
            if p.at(TokenKind::Colon) {
                p.bump(); // eat ':'
                expr::parse_type(p);
            }
            if !p.at(TokenKind::Comma) {
                break;
            }
            p.bump(); // eat ','
        }
    }
    p.expect(TokenKind::RParen);

    // Optional return type annotation: `: ReturnType`
    if p.at(TokenKind::Colon) {
        p.bump();
        expr::parse_type(p);
    }

    p.expect(TokenKind::Equals);

    // Body expression
    expr::expr(p);

    m.complete(p, SyntaxKind::VariableDef)
}

fn variable_def(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::ValKw));
    let m = p.start();
    p.bump();

    p.expect(TokenKind::Ident);

    // Optional type annotation: `: Type` or `: Type[Param]` or `: (Type, Type)`
    if p.at(TokenKind::Colon) {
        p.bump(); // eat ':'
        expr::parse_type(p);
    }

    p.expect(TokenKind::Equals);

    expr::expr(p);

    m.complete(p, SyntaxKind::VariableDef)
}

#[cfg(test)]
mod tests {
    use crate::parser::check;
    use expect_test::expect;

    #[test]
    fn parse_variable_definition() {
        check(
            "val foo = bar",
            expect![[r#"
            Root@0..13
              VariableDef@0..13
                ValKw@0..3 "val"
                Whitespace@3..4 " "
                Ident@4..7 "foo"
                Whitespace@7..8 " "
                Equals@8..9 "="
                Whitespace@9..10 " "
                Ident@10..13
                  Ident@10..13 "bar""#]],
        );
    }

    #[test]
    fn parse_variable_definition_with_type() {
        check(
            "val x: Long = 5L",
            expect![[r#"
                Root@0..16
                  VariableDef@0..16
                    ValKw@0..3 "val"
                    Whitespace@3..4 " "
                    Ident@4..5 "x"
                    Colon@5..6 ":"
                    Whitespace@6..7 " "
                    Ident@7..11 "Long"
                    Whitespace@11..12 " "
                    Equals@12..13 "="
                    Whitespace@13..14 " "
                    LongNumber@14..16
                      LongNumber@14..16 "5L""#]],
        );
    }
}
