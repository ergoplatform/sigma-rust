//! Operators in ErgoTree

use eval::costs::Costs;
use num::CheckedAdd;
use num::CheckedDiv;
use num::CheckedMul;
use num::CheckedSub;
use num::Num;

use crate::eval;
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::constant::TryExtractFrom;
use super::constant::TryExtractInto;
use super::expr::Expr;
use super::value::Value;

extern crate derive_more;
use derive_more::From;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
/// Operations for numerical types
pub enum NumOp {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl From<NumOp> for OpCode {
    fn from(_: NumOp) -> Self {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum LogicOp {
    Eq,
    NEq,
    // GE,
    // GT,
    // LE,
    // LT,
}

impl From<LogicOp> for OpCode {
    fn from(op: LogicOp) -> Self {
        match op {
            LogicOp::Eq => OpCode::EQ,
            LogicOp::NEq => OpCode::NEQ,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, From)]
/// Binary operations
pub enum BinOpKind {
    /// Binary operations for numerical types
    Num(NumOp),
    Logic(LogicOp),
}

impl From<BinOpKind> for OpCode {
    fn from(op: BinOpKind) -> Self {
        match op {
            BinOpKind::Num(o) => o.into(),
            BinOpKind::Logic(o) => o.into(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BinOp {
    pub kind: BinOpKind,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl BinOp {
    pub fn op_code(&self) -> OpCode {
        self.kind.into()
    }

    pub fn tpe(&self) -> SType {
        match self.kind {
            BinOpKind::Logic(_) => SType::SBoolean,
            BinOpKind::Num(_) => self.left.tpe(),
        }
    }
}

fn eval_plus<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedAdd + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_add(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) + ({1}) resulted in overflow",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

fn eval_minus<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedSub + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_sub(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) - ({1}) resulted in overflow",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

fn eval_mul<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedMul + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_mul(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) * ({1}) resulted in overflow",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

fn eval_div<T>(lv_raw: T, rv: Value) -> Result<Value, EvalError>
where
    T: Num + CheckedDiv + TryExtractFrom<Value> + Into<Value> + std::fmt::Display,
{
    let rv_raw = rv.try_extract_into::<T>()?;
    Ok((lv_raw.checked_div(&rv_raw).ok_or_else(|| {
        EvalError::ArithmeticException(format!(
            "({0}) / ({1}) resulted in exception",
            lv_raw, rv_raw
        ))
    })?)
    .into())
}

impl Evaluable for BinOp {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let lv = self.left.eval(env, ctx)?;
        let rv = self.right.eval(env, ctx)?;
        ctx.cost_accum.add(Costs::DEFAULT.eq_const_size)?;
        match self.kind {
            BinOpKind::Logic(op) => match op {
                LogicOp::Eq => Ok(Value::Boolean(lv == rv)),
                LogicOp::NEq => Ok(Value::Boolean(lv != rv)),
            },
            BinOpKind::Num(op) => match op {
                NumOp::Plus => match lv {
                    Value::Byte(lv_raw) => eval_plus(lv_raw, rv),
                    Value::Short(lv_raw) => eval_plus(lv_raw, rv),
                    Value::Int(lv_raw) => eval_plus(lv_raw, rv),
                    Value::Long(lv_raw) => eval_plus(lv_raw, rv),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                NumOp::Minus => match lv {
                    Value::Byte(lv_raw) => eval_minus(lv_raw, rv),
                    Value::Short(lv_raw) => eval_minus(lv_raw, rv),
                    Value::Int(lv_raw) => eval_minus(lv_raw, rv),
                    Value::Long(lv_raw) => eval_minus(lv_raw, rv),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                NumOp::Multiply => match lv {
                    Value::Byte(lv_raw) => eval_mul(lv_raw, rv),
                    Value::Short(lv_raw) => eval_mul(lv_raw, rv),
                    Value::Int(lv_raw) => eval_mul(lv_raw, rv),
                    Value::Long(lv_raw) => eval_mul(lv_raw, rv),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
                NumOp::Divide => match lv {
                    Value::Byte(lv_raw) => eval_div(lv_raw, rv),
                    Value::Short(lv_raw) => eval_div(lv_raw, rv),
                    Value::Int(lv_raw) => eval_div(lv_raw, rv),
                    Value::Long(lv_raw) => eval_div(lv_raw, rv),
                    Value::BigInt => todo!(),
                    _ => Err(EvalError::UnexpectedValue(format!(
                        "expected BinOp::left to be numeric value, got {0:?}",
                        lv
                    ))),
                },
            },
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::rc::Rc;

    use crate::ast::constant::Constant;
    use crate::ast::constant::TryExtractFrom;
    use crate::ast::expr::tests::ArbExprParams;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::eval::tests::try_eval_out;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    use super::*;

    impl Arbitrary for BinOp {
        type Parameters = ArbExprParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            match args.tpe {
                SType::SBoolean => (
                    any::<LogicOp>().prop_map_into(),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                    any_with::<Expr>(ArbExprParams {
                        tpe: SType::SAny,
                        depth: args.depth,
                    }),
                )
                    .prop_map(|(kind, left, right)| BinOp {
                        kind,
                        left: Box::new(left),
                        right: Box::new(right),
                    }),

                _ => todo!(),
                // SType::SByte => {}
                // SType::SShort => {}
                // SType::SInt => {}
                // SType::SLong => {}
                // SType::SBigInt => {}
            }
            .boxed()
        }
    }

    fn check_eq_neq(left: Constant, right: Constant) -> bool {
        let eq_op: Expr = BinOp {
            kind: BinOpKind::Logic(LogicOp::Eq),
            left: Box::new(left.clone().into()),
            right: Box::new(right.clone().into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let neq_op: Expr = BinOp {
            kind: BinOpKind::Logic(LogicOp::NEq),
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
        .into();
        let ctx1 = Rc::new(force_any_val::<Context>());
        eval_out::<bool>(&eq_op, ctx) && !eval_out::<bool>(&neq_op, ctx1)
    }

    #[test]
    fn num_eq() {
        assert!(check_eq_neq(1i64.into(), 1i64.into()));
    }

    #[test]
    fn num_neq() {
        assert!(!check_eq_neq(2i64.into(), 1i64.into()));
    }

    #[test]
    fn option_eq() {
        assert!(check_eq_neq(Some(1i64).into(), Some(1i64).into()));
        let none: Option<i64> = None;
        assert!(check_eq_neq(none.into(), none.into()));
        // Option<Vec<i8>>
        assert!(check_eq_neq(
            Some(vec![1i8, 2i8]).into(),
            Some(vec![1i8, 2i8]).into()
        ));
        // Vec<Option<i64>>
        assert!(check_eq_neq(
            vec![Some(1i64), Some(1i64)].into(),
            vec![Some(1i64), Some(1i64)].into()
        ));
    }

    #[test]
    fn option_neq() {
        assert!(!check_eq_neq(Some(2i64).into(), Some(1i64).into()));
        let none: Option<i64> = None;
        assert!(!check_eq_neq(none.into(), Some(1i64).into()));
        // Option<Vec<i8>>
        assert!(!check_eq_neq(
            Some(vec![1i8, 2i8]).into(),
            Some(vec![2i8, 2i8]).into()
        ));
        // Vec<Option<i64>>
        assert!(!check_eq_neq(
            vec![Some(1i64), Some(1i64)].into(),
            vec![Some(2i64), Some(1i64)].into()
        ));
    }

    #[test]
    fn tuple_eq() {
        assert!(check_eq_neq((1i64, true).into(), (1i64, true).into()));
    }

    fn eval_num_op<T: TryExtractFrom<Value>>(
        op: NumOp,
        left: Constant,
        right: Constant,
    ) -> Result<T, EvalError> {
        let expr: Expr = BinOp {
            kind: BinOpKind::Num(op),
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        try_eval_out::<T>(&expr, ctx)
    }

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn test_eq(v in any::<Constant>()) {
            prop_assert![check_eq_neq(v.clone(), v)];
        }

        #[test]
        fn test_arith_slong(l in any::<i64>(), r in any::<i64>()) {
            prop_assert_eq!(eval_num_op(NumOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(NumOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(NumOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(NumOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
        }

        #[test]
        fn test_arith_sint(l in any::<i32>(), r in any::<i32>()) {
            prop_assert_eq!(eval_num_op(NumOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(NumOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(NumOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(NumOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
        }

        #[test]
        fn test_arith_sshort(l in any::<i16>(), r in any::<i16>()) {
            prop_assert_eq!(eval_num_op(NumOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(NumOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(NumOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(NumOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
        }

        #[test]
        fn test_arith_sbyte(l in any::<i8>(), r in any::<i8>()) {
            prop_assert_eq!(eval_num_op(NumOp::Plus, l.into(), r.into()).ok(), l.checked_add(r));
            prop_assert_eq!(eval_num_op(NumOp::Minus, l.into(), r.into()).ok(), l.checked_sub(r));
            prop_assert_eq!(eval_num_op(NumOp::Multiply, l.into(), r.into()).ok(), l.checked_mul(r));
            prop_assert_eq!(eval_num_op(NumOp::Divide, l.into(), r.into()).ok(), l.checked_div(r));
        }
    }
}
