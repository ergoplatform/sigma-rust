use super::*;

pub(super) fn expr(p: &mut Parser) -> Option<CompletedMarker> {
    expr_binding_power(p, 0)
}

// from https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
//
// From Precedence to Binding Power
// I have a confession to make: I am always confused by “high precedence” and “low precedence”. In
// a + b * c, addition has a lower precedence, but it is at the top of the parse tree…​
//
// So instead, I find thinking in terms of binding power more intuitive.
//
// expr:   A       +       B       *       C
// power:      3       3       5       5
// The * is stronger, it has more power to hold together B and C, and so the expression is parsed
// as A + (B * C).
//
// What about associativity though? In A + B + C all operators seem to have the same power, and it
// is unclear which + to fold first. But this can also be modelled with power, if we make it
// slightly asymmetric:
//
// expr:      A       +       B       +       C
// power:  0      3      3.1      3      3.1     0
// Here, we pumped the right power of + just a little bit, so that it holds the right operand
// tighter. We also added zeros at both ends, as there are no operators to bind from the sides.
// Here, the first (and only the first) + holds both of its arguments tighter than the neighbors,
// so we can reduce it:
//
// expr:     (A + B)     +     C
// power:  0          3    3.1    0
// Now we can fold the second plus and get (A + B) + C. Or, in terms of the syntax tree, the second
// + really likes its right operand more than the left one, so it rushes to get hold of C. While he
// does that, the first + captures both A and B, as they are uncontested.
//
// What Pratt parsing does is that it finds these badass, stronger than neighbors operators, by
// processing the string left to right. We are almost at a point where we finally start writing
// some code, but let’s first look at the other running example. We will use function composition
// operator, . (dot) as a right associative operator with a high binding power. That is, f . g . h
// is parsed as f . (g . h), or, in terms of power
//
//   f     .    g     .    h
//   0   8.5    8   8.5    8   0
//
// ...
//
// And now comes the tricky bit, where we introduce recursion into the picture. Let’s think about
// this example (with powers below):
//
// a   +   b   *   c   *   d   +   e
//   1   2   3   4   3   4   1   2
//   The cursor is at the first +, we know that the left bp is 1 and the right one is 2. The lhs
//   stores a. The next operator after + is *, so we shouldn’t add b to a. The problem is that we
//   haven’t yet seen the next operator, we are just past +. Can we add a lookahead? Looks like
//   no — we’d have to look past all of b, c and d to find the next operator with lower binding
//   power, which sounds pretty unbounded. But we are onto something! Our current right priority is
//   2, and, to be able to fold the expression, we need to find the next operator with lower
//   priority. So let’s recursively call expr_bp starting at b, but also tell it to stop as soon as
//   bp drops below 2. This necessitates the addition of min_bp argument to the main function.
//
fn expr_binding_power(p: &mut Parser, minimum_binding_power: u8) -> Option<CompletedMarker> {
    let mut lhs = lhs(p)?;

    loop {
        let op = if p.at(TokenKind::Plus) {
            BinaryOp::Add
        } else if p.at(TokenKind::Minus) {
            BinaryOp::Sub
        } else if p.at(TokenKind::Star) {
            BinaryOp::Mul
        } else if p.at(TokenKind::Slash) {
            BinaryOp::Div
        } else {
            // We’re not at an operator; we don’t know what to do next, so we return and let the
            // caller decide.
            break;
        };

        let (left_binding_power, right_binding_power) = op.binding_power();

        if left_binding_power < minimum_binding_power {
            break;
        }

        // Eat the operator’s token.
        p.bump();

        //  And here we bump past the operator itself and make the recursive call. Note how we use
        //  left_binding_power to check against minimum_binding_power, and right_binding_power
        //  as the new minimum_binding_power of the recursive call. So, you can think
        //  about minimum_binding_power as the binding power of the operator to the left of the current expressions.
        //  https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
        let m = lhs.precede(p);
        let parsed_rhs = expr_binding_power(p, right_binding_power).is_some();
        lhs = m.complete(p, SyntaxKind::InfixExpr);

        if !parsed_rhs {
            break;
        }
    }

    Some(lhs)
}

fn lhs(p: &mut Parser) -> Option<CompletedMarker> {
    let cm = if p.at(TokenKind::IntNumber) {
        int_number(p)
    } else if p.at(TokenKind::LongNumber) {
        long_number(p)
    } else if p.at(TokenKind::Ident) {
        ident(p)
        // variable_ref(p)
        // } else if p.at(TokenKind::ValKw) {
        //     variable_ref(p)
    } else if p.at(TokenKind::Minus) {
        prefix_expr(p)
    } else if p.at(TokenKind::LParen) {
        paren_expr(p)
    } else {
        p.error();
        return None;
    };

    Some(cm)
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinaryOp {
    fn binding_power(&self) -> (u8, u8) {
        match self {
            Self::Add | Self::Sub => (1, 2),
            Self::Mul | Self::Div => (3, 4),
        }
    }
}

enum UnaryOp {
    Neg,
}

impl UnaryOp {
    fn binding_power(&self) -> ((), u8) {
        match self {
            Self::Neg => ((), 5),
        }
    }
}

fn int_number(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::IntNumber));
    let m = p.start();
    p.bump();
    m.complete(p, SyntaxKind::IntNumber)
}

fn long_number(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::LongNumber));
    let m = p.start();
    p.bump();
    m.complete(p, SyntaxKind::LongNumber)
}

// fn variable_ref(p: &mut Parser) -> CompletedMarker {
//     assert!(p.at(TokenKind::Ident));

//     let m = p.start();
//     p.bump();
//     m.complete(p, SyntaxKind::VariableRef)
// }

fn ident(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::Ident));

    let m = p.start();
    p.bump();
    m.complete(p, SyntaxKind::Ident)
}

fn prefix_expr(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::Minus));

    let m = p.start();

    let op = UnaryOp::Neg;
    let ((), right_binding_power) = op.binding_power();

    // Eat the operator’s token.
    p.bump();

    expr_binding_power(p, right_binding_power);

    m.complete(p, SyntaxKind::PrefixExpr)
}

fn paren_expr(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::LParen));

    let m = p.start();
    p.bump();
    expr_binding_power(p, 0);
    p.expect(TokenKind::RParen);

    m.complete(p, SyntaxKind::ParenExpr)
}

#[cfg(test)]
mod tests {
    use crate::parser::check;
    use expect_test::expect;

    #[test]
    fn parse_number() {
        check(
            "123",
            expect![[r#"
                Root@0..3
                  IntNumber@0..3
                    IntNumber@0..3 "123""#]],
        );
    }

    #[test]
    fn parse_number_preceded_by_whitespace() {
        check(
            "  9876",
            expect![[r#"
                Root@0..6
                  Whitespace@0..2 "  "
                  IntNumber@2..6
                    IntNumber@2..6 "9876""#]],
        );
    }

    #[test]
    fn parse_number_followed_by_whitespace() {
        check(
            "999  ",
            expect![[r#"
                Root@0..5
                  IntNumber@0..5
                    IntNumber@0..3 "999"
                    Whitespace@3..5 "  ""#]],
        );
    }

    #[test]
    fn parse_number_surrounded_by_whitespace() {
        check(
            " 123    ",
            expect![[r#"
                Root@0..8
                  Whitespace@0..1 " "
                  IntNumber@1..8
                    IntNumber@1..4 "123"
                    Whitespace@4..8 "    ""#]],
        );
    }

    #[test]
    fn parse_simple_infix_expression() {
        check(
            "1+3",
            expect![[r#"
                Root@0..3
                  InfixExpr@0..3
                    IntNumber@0..1
                      IntNumber@0..1 "1"
                    Plus@1..2 "+"
                    IntNumber@2..3
                      IntNumber@2..3 "3""#]],
        );
    }

    #[test]
    fn parse_left_associative_infix_expression() {
        check(
            "1+2+3",
            expect![[r#"
                Root@0..5
                  InfixExpr@0..5
                    InfixExpr@0..3
                      IntNumber@0..1
                        IntNumber@0..1 "1"
                      Plus@1..2 "+"
                      IntNumber@2..3
                        IntNumber@2..3 "2"
                    Plus@3..4 "+"
                    IntNumber@4..5
                      IntNumber@4..5 "3""#]],
        );
    }

    #[test]
    fn parse_infix_expression_with_mixed_binding_power() {
        check(
            "1+2*3-5",
            expect![[r#"
                Root@0..7
                  InfixExpr@0..7
                    InfixExpr@0..5
                      IntNumber@0..1
                        IntNumber@0..1 "1"
                      Plus@1..2 "+"
                      InfixExpr@2..5
                        IntNumber@2..3
                          IntNumber@2..3 "2"
                        Star@3..4 "*"
                        IntNumber@4..5
                          IntNumber@4..5 "3"
                    Minus@5..6 "-"
                    IntNumber@6..7
                      IntNumber@6..7 "5""#]],
        );
    }

    #[test]
    fn parse_infix_expression_with_whitespace() {
        check(
            " 1 +  2* 3 ",
            expect![[r#"
                Root@0..11
                  Whitespace@0..1 " "
                  InfixExpr@1..11
                    IntNumber@1..3
                      IntNumber@1..2 "1"
                      Whitespace@2..3 " "
                    Plus@3..4 "+"
                    Whitespace@4..6 "  "
                    InfixExpr@6..11
                      IntNumber@6..7
                        IntNumber@6..7 "2"
                      Star@7..8 "*"
                      Whitespace@8..9 " "
                      IntNumber@9..11
                        IntNumber@9..10 "3"
                        Whitespace@10..11 " ""#]],
        );
    }

    #[test]
    fn parse_infix_expression_interspersed_with_comments() {
        check(
            "
1
  + 1 // Add one
  + 10 // Add ten",
            expect![[r#"
                Root@0..37
                  Whitespace@0..1 "\n"
                  InfixExpr@1..37
                    InfixExpr@1..22
                      IntNumber@1..5
                        IntNumber@1..2 "1"
                        Whitespace@2..5 "\n  "
                      Plus@5..6 "+"
                      Whitespace@6..7 " "
                      IntNumber@7..22
                        IntNumber@7..8 "1"
                        Whitespace@8..9 " "
                        Comment@9..19 "// Add one"
                        Whitespace@19..22 "\n  "
                    Plus@22..23 "+"
                    Whitespace@23..24 " "
                    IntNumber@24..37
                      IntNumber@24..26 "10"
                      Whitespace@26..27 " "
                      Comment@27..37 "// Add ten""#]],
        );
    }

    #[test]
    fn do_not_parse_operator_if_gettting_rhs_failed() {
        check(
            "(2+",
            expect![[r#"
                Root@0..3
                  ParenExpr@0..3
                    LParen@0..1 "("
                    InfixExpr@1..3
                      IntNumber@1..2
                        IntNumber@1..2 "2"
                      Plus@2..3 "+"
                error: expected number, number, identifier, ‘-’ or ‘(’
                error: expected ‘)’"#]],
        );
    }

    #[test]
    fn parse_negation() {
        check(
            "-11",
            expect![[r#"
                Root@0..3
                  PrefixExpr@0..3
                    Minus@0..1 "-"
                    IntNumber@1..3
                      IntNumber@1..3 "11""#]],
        );
    }

    #[test]
    fn negation_has_higher_binding_power_than_binary_operators() {
        check(
            "-20+21",
            expect![[r#"
                Root@0..6
                  InfixExpr@0..6
                    PrefixExpr@0..3
                      Minus@0..1 "-"
                      IntNumber@1..3
                        IntNumber@1..3 "20"
                    Plus@3..4 "+"
                    IntNumber@4..6
                      IntNumber@4..6 "21""#]],
        );
    }

    #[test]
    fn parse_nested_parentheses() {
        check(
            "((((((11))))))",
            expect![[r#"
                Root@0..14
                  ParenExpr@0..14
                    LParen@0..1 "("
                    ParenExpr@1..13
                      LParen@1..2 "("
                      ParenExpr@2..12
                        LParen@2..3 "("
                        ParenExpr@3..11
                          LParen@3..4 "("
                          ParenExpr@4..10
                            LParen@4..5 "("
                            ParenExpr@5..9
                              LParen@5..6 "("
                              IntNumber@6..8
                                IntNumber@6..8 "11"
                              RParen@8..9 ")"
                            RParen@9..10 ")"
                          RParen@10..11 ")"
                        RParen@11..12 ")"
                      RParen@12..13 ")"
                    RParen@13..14 ")""#]],
        );
    }

    #[test]
    fn parentheses_affect_precedence() {
        check(
            "5*(2+3)",
            expect![[r#"
                Root@0..7
                  InfixExpr@0..7
                    IntNumber@0..1
                      IntNumber@0..1 "5"
                    Star@1..2 "*"
                    ParenExpr@2..7
                      LParen@2..3 "("
                      InfixExpr@3..6
                        IntNumber@3..4
                          IntNumber@3..4 "2"
                        Plus@4..5 "+"
                        IntNumber@5..6
                          IntNumber@5..6 "3"
                      RParen@6..7 ")""#]],
        );
    }
}
