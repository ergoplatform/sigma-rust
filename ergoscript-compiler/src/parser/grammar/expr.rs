use super::*;

pub(super) fn expr(p: &mut Parser) -> Option<CompletedMarker> {
    expr_binding_power(p, 0)
}

// Pratt parser with binding powers.
// ErgoScript precedence (low to high):
//   ||          : 1,2
//   &&          : 3,4
//   ==, !=      : 5,6
//   >, <, >=, <=: 7,8
//   +, -        : 9,10
//   *, /        : 11,12
//   unary -, !  : ((), 13)
//   postfix call: 15,16
fn expr_binding_power(p: &mut Parser, minimum_binding_power: u8) -> Option<CompletedMarker> {
    let mut lhs = lhs(p)?;

    loop {
        // Check for postfix: dot access `expr.ident`
        if p.at(TokenKind::Dot) {
            let left_binding_power = 17_u8;
            if left_binding_power < minimum_binding_power {
                break;
            }
            let m = lhs.precede(p);
            p.bump(); // eat '.'
            p.expect(TokenKind::Ident); // field name
            // Optional type args: [Type] or [(Type, Type)] or [Coll[Byte]]
            if p.at(TokenKind::LBracket) {
                p.bump(); // eat '['
                parse_type(p);
                p.expect(TokenKind::RBracket);
            }
            lhs = m.complete(p, SyntaxKind::FieldAccess);
            continue;
        }

        // Check for postfix: function call `expr(args)` or `expr[Type](args)`
        // Also handles type application: `Coll[Byte]()`, `getVar[Int](0)`
        // Don't treat ( or [ as postfix if preceded by newline
        if (p.at(TokenKind::LParen) || p.at(TokenKind::LBracket)) && !p.had_newline() {
            let left_binding_power = 15_u8;
            if left_binding_power < minimum_binding_power {
                break;
            }
            let m = lhs.precede(p);
            // Optional type args: expr[Type](args)
            if p.at(TokenKind::LBracket) {
                p.bump(); // eat '['
                parse_type(p);
                p.expect(TokenKind::RBracket);
            }
            // Args: (args)
            p.expect(TokenKind::LParen);
            if !p.at(TokenKind::RParen) {
                expr_binding_power(p, 0);
                while p.at(TokenKind::Comma) {
                    p.bump();
                    expr_binding_power(p, 0);
                }
            }
            p.expect(TokenKind::RParen);
            lhs = m.complete(p, SyntaxKind::FuncCall);
            continue;
        }

        // Check for postfix: method call with block arg `expr.method { lambda }`
        // Don't treat { as postfix block if preceded by newline
        if p.at(TokenKind::LBrace) && !p.had_newline() {
            let left_binding_power = 15_u8;
            if left_binding_power < minimum_binding_power {
                break;
            }
            let m = lhs.precede(p);
            // Parse the block/lambda as the single argument
            let arg = block_expr(p);
            lhs = m.complete(p, SyntaxKind::FuncCall);
            continue;
        }

        let op = if p.at(TokenKind::Plus) {
            BinaryOp::Add
        } else if p.at(TokenKind::Minus) {
            BinaryOp::Sub
        } else if p.at(TokenKind::Star) {
            BinaryOp::Mul
        } else if p.at(TokenKind::Slash) {
            BinaryOp::Div
        } else if p.at(TokenKind::Percent) {
            BinaryOp::Mod
        } else if p.at(TokenKind::And) {
            BinaryOp::And
        } else if p.at(TokenKind::Or) {
            BinaryOp::Or
        } else if p.at(TokenKind::EqEq) {
            BinaryOp::Eq
        } else if p.at(TokenKind::NotEq) {
            BinaryOp::Neq
        } else if p.at(TokenKind::Gt) {
            BinaryOp::Gt
        } else if p.at(TokenKind::Lt) {
            BinaryOp::Lt
        } else if p.at(TokenKind::GtEq) {
            BinaryOp::Ge
        } else if p.at(TokenKind::LtEq) {
            BinaryOp::Le
        } else {
            break;
        };

        let (left_binding_power, right_binding_power) = op.binding_power();

        if left_binding_power < minimum_binding_power {
            break;
        }

        // Eat the operator's token.
        p.bump();

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
    } else if p.at(TokenKind::TrueKw) {
        bool_literal(p)
    } else if p.at(TokenKind::FalseKw) {
        bool_literal(p)
    } else if p.at(TokenKind::StringLiteral) {
        string_literal(p)
    } else if p.at(TokenKind::Ident) {
        ident(p)
    } else if p.at(TokenKind::Minus) {
        prefix_expr(p)
    } else if p.at(TokenKind::Bang) {
        prefix_not(p)
    } else if p.at(TokenKind::LParen) {
        paren_expr(p)
    } else if p.at(TokenKind::IfKw) {
        if_expr(p)
    } else if p.at(TokenKind::LBrace) {
        block_expr(p)
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
    Mod,
    And,
    Or,
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le,
}

impl BinaryOp {
    fn binding_power(&self) -> (u8, u8) {
        match self {
            Self::Or => (1, 2),
            Self::And => (3, 4),
            Self::Eq | Self::Neq => (5, 6),
            Self::Gt | Self::Lt | Self::Ge | Self::Le => (7, 8),
            Self::Add | Self::Sub => (9, 10),
            Self::Mul | Self::Div | Self::Mod => (11, 12),
        }
    }
}

enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    fn binding_power(&self) -> ((), u8) {
        match self {
            Self::Neg | Self::Not => ((), 13),
        }
    }
}

fn if_expr(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::IfKw));

    let m = p.start();
    p.bump(); // eat 'if'
    p.expect(TokenKind::LParen);
    expr_binding_power(p, 0); // condition
    p.expect(TokenKind::RParen);
    expr_binding_power(p, 0); // then branch (could be block or single expr)
    p.expect(TokenKind::ElseKw);
    expr_binding_power(p, 0); // else branch
    m.complete(p, SyntaxKind::IfExpr)
}

fn string_literal(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::StringLiteral));
    let m = p.start();
    p.bump();
    m.complete(p, SyntaxKind::StringLiteral)
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

fn bool_literal(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::TrueKw) || p.at(TokenKind::FalseKw));
    let m = p.start();
    p.bump();
    m.complete(p, SyntaxKind::BoolLiteral)
}

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
    p.bump();
    expr_binding_power(p, right_binding_power);
    m.complete(p, SyntaxKind::PrefixExpr)
}

fn prefix_not(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::Bang));

    let m = p.start();
    let op = UnaryOp::Not;
    let ((), right_binding_power) = op.binding_power();
    p.bump();
    expr_binding_power(p, right_binding_power);
    m.complete(p, SyntaxKind::PrefixExpr)
}

fn paren_expr(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::LParen));

    let m = p.start();
    p.bump(); // eat '('
    expr_binding_power(p, 0);
    if p.at(TokenKind::Comma) {
        // Tuple literal: (expr, expr, ...)
        while p.at(TokenKind::Comma) {
            p.bump();
            expr_binding_power(p, 0);
        }
        p.expect(TokenKind::RParen);
        m.complete(p, SyntaxKind::TupleExpr)
    } else {
        p.expect(TokenKind::RParen);
        m.complete(p, SyntaxKind::ParenExpr)
    }
}

fn block_expr(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(TokenKind::LBrace));

    let m = p.start();
    p.bump(); // eat '{'

    // Check if this is a lambda: { (params: Type) => body }
    // Only attempt lambda if ( is followed by Ident then : (param type annotation)
    // or by ) then => (empty param list)
    if p.at(TokenKind::LParen) && is_lambda_start(p) {
        p.bump(); // eat '('
        // Parse comma-separated params: ident : Type
        if !p.at(TokenKind::RParen) {
            p.expect(TokenKind::Ident); // param name
            if p.at(TokenKind::Colon) {
                p.bump();
                parse_type(p);
            }
            while p.at(TokenKind::Comma) {
                p.bump();
                p.expect(TokenKind::Ident); // param name
                if p.at(TokenKind::Colon) {
                    p.bump();
                    parse_type(p);
                }
            }
        }
        p.expect(TokenKind::RParen);

        if p.at(TokenKind::Arrow) {
            p.bump(); // eat '=>'
            expr_binding_power(p, 0); // parse body
            p.expect(TokenKind::RBrace);
            return m.complete(p, SyntaxKind::Lambda);
        }
        // Not a lambda — fall through to regular block
    }

    // Regular block
    while !p.at(TokenKind::RBrace) && !p.at_end() {
        super::stmt::stmt(p);
        while p.at(TokenKind::Semicolon) {
            p.bump();
        }
    }

    p.expect(TokenKind::RBrace);
    m.complete(p, SyntaxKind::BlockExpr)
}

/// Check if the current position looks like the start of a lambda: ( ident : ... )
/// Does NOT consume any tokens — uses raw peek on the token array.
fn is_lambda_start(p: &Parser) -> bool {
    // Look at tokens from current cursor: skip trivia, expect (, skip trivia, expect Ident or ), skip trivia
    let tokens = &p.source.tokens;
    let mut i = p.source.cursor;
    // Skip trivia to find (
    while i < tokens.len() && tokens[i].kind.is_trivia() { i += 1; }
    if i >= tokens.len() || tokens[i].kind != TokenKind::LParen { return false; }
    i += 1;
    // Skip trivia after (
    while i < tokens.len() && tokens[i].kind.is_trivia() { i += 1; }
    if i >= tokens.len() { return false; }
    // If ) follows immediately → could be empty lambda () =>
    if tokens[i].kind == TokenKind::RParen {
        i += 1;
        while i < tokens.len() && tokens[i].kind.is_trivia() { i += 1; }
        return i < tokens.len() && tokens[i].kind == TokenKind::Arrow;
    }
    // Expect Ident (param name)
    if tokens[i].kind != TokenKind::Ident { return false; }
    i += 1;
    // Skip trivia after Ident
    while i < tokens.len() && tokens[i].kind.is_trivia() { i += 1; }
    if i >= tokens.len() { return false; }
    // Must be : for this to be a lambda param
    tokens[i].kind == TokenKind::Colon
}

/// Parse a type expression: Ident, Ident[Type], or (Type, Type, ...)
pub(super) fn parse_type(p: &mut Parser) {
    if p.at(TokenKind::LParen) {
        // Tuple type: (Type, Type, ...)
        p.bump(); // eat '('
        parse_type(p);
        while p.at(TokenKind::Comma) {
            p.bump();
            parse_type(p);
        }
        p.expect(TokenKind::RParen);
    } else {
        // Simple or generic type: Ident or Ident[Type]
        p.expect(TokenKind::Ident);
        if p.at(TokenKind::LBracket) {
            p.bump(); // eat '['
            parse_type(p);
            p.expect(TokenKind::RBracket);
        }
    }
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

    #[test]
    fn parse_comparison_gt() {
        check(
            "HEIGHT > 0",
            expect![[r#"
                Root@0..10
                  InfixExpr@0..10
                    Ident@0..7
                      Ident@0..6 "HEIGHT"
                      Whitespace@6..7 " "
                    Gt@7..8 ">"
                    Whitespace@8..9 " "
                    IntNumber@9..10
                      IntNumber@9..10 "0""#]],
        );
    }

    #[test]
    fn parse_and_binds_lower_than_comparison() {
        check(
            "HEIGHT > 0 && HEIGHT < 100",
            expect![[r#"
                Root@0..26
                  InfixExpr@0..26
                    InfixExpr@0..11
                      Ident@0..7
                        Ident@0..6 "HEIGHT"
                        Whitespace@6..7 " "
                      Gt@7..8 ">"
                      Whitespace@8..9 " "
                      IntNumber@9..11
                        IntNumber@9..10 "0"
                        Whitespace@10..11 " "
                    And@11..13 "&&"
                    Whitespace@13..14 " "
                    InfixExpr@14..26
                      Ident@14..21
                        Ident@14..20 "HEIGHT"
                        Whitespace@20..21 " "
                      Lt@21..22 "<"
                      Whitespace@22..23 " "
                      IntNumber@23..26
                        IntNumber@23..26 "100""#]],
        );
    }

    #[test]
    fn parse_func_call() {
        check(
            "sigmaProp(true)",
            expect![[r#"
                Root@0..15
                  FuncCall@0..15
                    Ident@0..9
                      Ident@0..9 "sigmaProp"
                    LParen@9..10 "("
                    BoolLiteral@10..14
                      TrueKw@10..14 "true"
                    RParen@14..15 ")""#]],
        );
    }

    #[test]
    fn parse_block_expr() {
        check(
            "{ 1 + 2 }",
            expect![[r#"
                Root@0..9
                  BlockExpr@0..9
                    LBrace@0..1 "{"
                    Whitespace@1..2 " "
                    InfixExpr@2..8
                      IntNumber@2..4
                        IntNumber@2..3 "1"
                        Whitespace@3..4 " "
                      Plus@4..5 "+"
                      Whitespace@5..6 " "
                      IntNumber@6..8
                        IntNumber@6..7 "2"
                        Whitespace@7..8 " "
                    RBrace@8..9 "}""#]],
        );
    }

    #[test]
    fn parse_bool_literal_true() {
        check(
            "true",
            expect![[r#"
                Root@0..4
                  BoolLiteral@0..4
                    TrueKw@0..4 "true""#]],
        );
    }

    #[test]
    fn parse_bool_literal_false() {
        check(
            "false",
            expect![[r#"
                Root@0..5
                  BoolLiteral@0..5
                    FalseKw@0..5 "false""#]],
        );
    }

    #[test]
    fn parse_full_session1_target() {
        // Just verify it parses without error
        check(
            "{ sigmaProp(HEIGHT > 0 && HEIGHT < 100) }",
            expect![[r#"
                Root@0..41
                  BlockExpr@0..41
                    LBrace@0..1 "{"
                    Whitespace@1..2 " "
                    FuncCall@2..40
                      Ident@2..11
                        Ident@2..11 "sigmaProp"
                      LParen@11..12 "("
                      InfixExpr@12..38
                        InfixExpr@12..23
                          Ident@12..19
                            Ident@12..18 "HEIGHT"
                            Whitespace@18..19 " "
                          Gt@19..20 ">"
                          Whitespace@20..21 " "
                          IntNumber@21..23
                            IntNumber@21..22 "0"
                            Whitespace@22..23 " "
                        And@23..25 "&&"
                        Whitespace@25..26 " "
                        InfixExpr@26..38
                          Ident@26..33
                            Ident@26..32 "HEIGHT"
                            Whitespace@32..33 " "
                          Lt@33..34 "<"
                          Whitespace@34..35 " "
                          IntNumber@35..38
                            IntNumber@35..38 "100"
                      RParen@38..39 ")"
                      Whitespace@39..40 " "
                    RBrace@40..41 "}""#]],
        );
    }
}
