use super::*;

pub(super) fn stmt(p: &mut Parser) -> Option<CompletedMarker> {
    if p.at(TokenKind::ValKw) {
        Some(variable_def(p))
    } else {
        expr::expr(p)
    }
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
