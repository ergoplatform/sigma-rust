//! Conformance tests for bitwise infix operators on numeric types.
//!
//! Coverage:
//!   * `&` (bitwise AND) — BinOp(BitOp::BitAnd)
//!   * `|` (bitwise OR)  — BinOp(BitOp::BitOr)
//!   * `^` (bitwise XOR) — BinOp(BitOp::BitXor)
//!   * `~` (bitwise NOT) — BitInversion
//!
//! Per Scala SigmaParser, these all produce `BitOp` IR nodes for binary
//! forms and `BitInversion` for the unary form. We don't yet ship the shift
//! operators (`<<`/`>>`/`>>>`) because they require extending the Rust
//! `BitOp` enum with shift variants — that's queued separately.

use super::compile_ok;
use ergotree_ir::mir::bin_op::{BinOp, BinOpKind, BitOp};
use ergotree_ir::mir::expr::Expr;

fn unwrap_binop(e: &Expr) -> &BinOp {
    match e {
        Expr::BinOp(spanned) => &spanned.expr,
        Expr::BlockValue(spanned) => unwrap_binop(&spanned.expr.result),
        _ => panic!("expected BinOp, got {:?}", e),
    }
}

fn unwrap_result(e: &Expr) -> &Expr {
    match e {
        Expr::BlockValue(spanned) => unwrap_result(&spanned.expr.result),
        _ => e,
    }
}

// All tests use HEIGHT (Int, runtime) or HEIGHT.toLong to prevent the
// HIR constant-fold pass from collapsing the whole expression to a Const.

#[test]
fn infix_bitwise_and_long() {
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 10L; a & b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitAnd));
}

#[test]
fn infix_bitwise_or_long() {
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 10L; a | b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitOr));
}

#[test]
fn infix_bitwise_xor_long() {
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 10L; a ^ b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitXor));
}

#[test]
fn infix_bitwise_and_int() {
    let e = compile_ok("{ val a: Int = HEIGHT; val b: Int = 10; a & b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitAnd));
}

#[test]
fn prefix_bit_inversion_long() {
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; ~a }");
    let res = unwrap_result(&e);
    matches!(res, Expr::BitInversion(_))
        .then_some(())
        .unwrap_or_else(|| panic!("expected BitInversion, got {:?}", res));
}

#[test]
fn prefix_bit_inversion_int() {
    let e = compile_ok("{ val a: Int = HEIGHT; ~a }");
    let res = unwrap_result(&e);
    matches!(res, Expr::BitInversion(_))
        .then_some(())
        .unwrap_or_else(|| panic!("expected BitInversion, got {:?}", res));
}

#[test]
fn precedence_amp_higher_than_pipe() {
    // `&` (9,10) is tighter than `|` (3,4). `a | b & c` parses as
    // `a | (b & c)`.
    let e = compile_ok(
        "{ val a: Long = HEIGHT.toLong; val b: Long = 2L; val c: Long = 4L; a | b & c }",
    );
    let outer = unwrap_binop(&e);
    assert_eq!(outer.kind, BinOpKind::Bit(BitOp::BitOr));
    let inner = unwrap_binop(&outer.right);
    assert_eq!(inner.kind, BinOpKind::Bit(BitOp::BitAnd));
}

#[test]
fn precedence_caret_between_pipe_and_amp() {
    // `^` (5,6) sits between `|` (3,4) and `&` (9,10). So `a | b ^ c & d`
    // parses as `a | (b ^ (c & d))`.
    let e = compile_ok(
        "{ val a: Long = HEIGHT.toLong; val b: Long = 2L; val c: Long = 4L; val d: Long = 8L; a | b ^ c & d }",
    );
    let outer = unwrap_binop(&e); // |
    assert_eq!(outer.kind, BinOpKind::Bit(BitOp::BitOr));
    let mid = unwrap_binop(&outer.right); // ^
    assert_eq!(mid.kind, BinOpKind::Bit(BitOp::BitXor));
    let inner = unwrap_binop(&mid.right); // &
    assert_eq!(inner.kind, BinOpKind::Bit(BitOp::BitAnd));
}

#[test]
fn precedence_arith_higher_than_bitand() {
    // `+` (17,18) is higher than `&` (9,10). So `a & b + c` parses as
    // `a & (b + c)`. Use HEIGHT to defeat constant folding.
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 2L; a & b + a }");
    let outer = unwrap_binop(&e);
    assert_eq!(outer.kind, BinOpKind::Bit(BitOp::BitAnd));
    let inner = unwrap_binop(&outer.right);
    assert_eq!(
        inner.kind,
        BinOpKind::Arith(ergotree_ir::mir::bin_op::ArithOp::Plus)
    );
}

#[test]
fn precedence_tilde_higher_than_binop() {
    // `~a & b` should parse as `(~a) & b` — prefix unary has higher precedence
    // than any infix binary.
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 2L; ~a & b }");
    let outer = unwrap_binop(&e);
    assert_eq!(outer.kind, BinOpKind::Bit(BitOp::BitAnd));
    matches!(*outer.left, Expr::BitInversion(_))
        .then_some(())
        .unwrap_or_else(|| panic!("expected BitInversion as lhs, got {:?}", outer.left));
}

#[test]
fn infix_shl_long() {
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 4L; a << b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitShiftLeft));
}

#[test]
fn infix_shr_long() {
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 4L; a >> b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitShiftRight));
}

#[test]
fn infix_ushr_long() {
    // `>>>` must be tokenized as URShift, not RShift+Gt.
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; val b: Long = 4L; a >>> b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitShiftRightZeroed));
}

#[test]
fn infix_shl_int() {
    let e = compile_ok("{ val a: Int = HEIGHT; val b: Int = 4; a << b }");
    let bin = unwrap_binop(&e);
    assert_eq!(bin.kind, BinOpKind::Bit(BitOp::BitShiftLeft));
}

#[test]
fn precedence_shift_higher_than_comparison() {
    // Shifts (15,16) are tighter than comparisons (13,14). So `a << b > c`
    // parses as `(a << b) > c`.
    let e = compile_ok(
        "{ val a: Long = HEIGHT.toLong; val b: Long = 2L; val c: Long = 5L; a << b > c }",
    );
    let outer = unwrap_binop(&e); // >
    assert_eq!(
        outer.kind,
        BinOpKind::Relation(ergotree_ir::mir::bin_op::RelationOp::Gt)
    );
    let inner = unwrap_binop(&outer.left); // <<
    assert_eq!(inner.kind, BinOpKind::Bit(BitOp::BitShiftLeft));
}

#[test]
fn precedence_arith_higher_than_shift() {
    // `+` (17,18) is tighter than `<<` (15,16). So `a << b + c` parses as
    // `a << (b + c)`. Use HEIGHT-derived bindings on both sides of `+` so
    // constant_fold can't collapse the addend.
    let e = compile_ok(
        "{ val a: Long = HEIGHT.toLong; val b: Long = HEIGHT.toLong; val c: Long = 2L; a << b + c }",
    );
    let outer = unwrap_binop(&e); // <<
    assert_eq!(outer.kind, BinOpKind::Bit(BitOp::BitShiftLeft));
    let inner = unwrap_binop(&outer.right); // +
    assert_eq!(
        inner.kind,
        BinOpKind::Arith(ergotree_ir::mir::bin_op::ArithOp::Plus)
    );
}

#[test]
fn double_tilde_collapses_to_two_inversions() {
    // `~~a` should parse as `~(~a)` — two stacked BitInversion nodes.
    let e = compile_ok("{ val a: Long = HEIGHT.toLong; ~~a }");
    let res = unwrap_result(&e);
    let outer = match res {
        Expr::BitInversion(b) => b,
        _ => panic!("expected outer BitInversion, got {:?}", res),
    };
    matches!(&*outer.input, Expr::BitInversion(_))
        .then_some(())
        .unwrap_or_else(|| panic!("expected inner BitInversion, got {:?}", outer.input));
}
