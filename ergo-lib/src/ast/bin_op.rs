//! Operators in ErgoTree

use eval::costs::Costs;

use crate::eval;
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::types::stype::SType;

use super::expr::Expr;
use super::value::Value;

extern crate derive_more;
use derive_more::From;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
/// Operations for numerical types
pub enum NumOp {
    /// Addition
    Add,
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
    // Num(NumOp),
    Logic(LogicOp),
}

impl From<BinOpKind> for OpCode {
    fn from(op: BinOpKind) -> Self {
        match op {
            // BinOpKind::Num(o) => o.into(),
            BinOpKind::Logic(o) => o.into(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BinOp {
    pub kind: BinOpKind,
    pub left: Expr,
    pub right: Expr,
}

impl BinOp {
    pub fn op_code(&self) -> OpCode {
        self.kind.into()
    }

    pub fn tpe(&self) -> SType {
        match self.kind {
            BinOpKind::Logic(_) => SType::SBoolean,
        }
    }
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
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::rc::Rc;

    use crate::ast::constant::Constant;
    use crate::ast::expr::tests::ArbExprParams;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
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
                    .prop_map(|(kind, left, right)| BinOp { kind, left, right }),

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
        let eq_op: Expr = Box::new(BinOp {
            kind: BinOpKind::Logic(LogicOp::Eq),
            left: Box::new(left.clone()).into(),
            right: Box::new(right.clone()).into(),
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        let neq_op: Expr = Box::new(BinOp {
            kind: BinOpKind::Logic(LogicOp::NEq),
            left: Box::new(left).into(),
            right: Box::new(right).into(),
        })
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

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn test_eq(v in any::<Constant>()) {
            prop_assert![check_eq_neq(v.clone(), v)];
        }
    }
}
