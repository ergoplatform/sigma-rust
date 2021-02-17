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
}
