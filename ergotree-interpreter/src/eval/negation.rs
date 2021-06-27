use ergotree_ir::mir::negation::Negation;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use num_traits::CheckedNeg;

impl Evaluable for Negation {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;

        fn overflow_err<T: std::fmt::Display>(v: &T) -> EvalError {
            EvalError::ArithmeticException(format!("Overflow on Negation of value {}", *v))
        }
        fn neg<T: CheckedNeg + Into<Value> + std::fmt::Display>(v: &T) -> Result<Value, EvalError> {
            v.checked_neg()
                .map(|v| v.into())
                .ok_or_else(|| overflow_err(v))
        }
        match input_v {
            Value::Byte(v) => neg(&v),
            Value::Short(v) => neg(&v),
            Value::Int(v) => neg(&v),
            Value::Long(v) => neg(&v),
            Value::BigInt(v) => neg(&v),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected Negation input to be numeric value, got {:?}",
                input_v
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::eval::tests::try_eval_out_wo_ctx;
    use ergotree_ir::bigint256::BigInt256;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::constant::TryExtractFrom;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::unary_op::UnaryOpTryBuild;
    use num_traits::{Bounded, Num};
    use std::convert::TryFrom;

    fn try_run_eval<T: Num + Into<Constant> + TryExtractFrom<Value>>(
        input: T,
    ) -> Result<T, EvalError> {
        let expr: Expr = Negation::try_build(Expr::Const(input.into()))
            .unwrap()
            .into();
        try_eval_out_wo_ctx::<T>(&expr)
    }
    fn run_eval<T: Num + Into<Constant> + TryExtractFrom<Value>>(input: T) -> T {
        try_run_eval(input).unwrap()
    }

    #[test]
    fn eval() {
        assert_eq!(run_eval(1i8), -1i8);
        assert!(try_run_eval(i8::MIN).is_err());
        assert_eq!(run_eval(1i16), -1i16);
        assert!(try_run_eval(i16::MIN).is_err());
        assert_eq!(run_eval(1i32), -1i32);
        assert!(try_run_eval(i32::MIN).is_err());
        assert_eq!(run_eval(1i64), -1i64);
        assert!(try_run_eval(i64::MIN).is_err());
        assert_eq!(
            run_eval(BigInt256::try_from(1i64).unwrap()),
            BigInt256::try_from(-1i64).unwrap()
        );
        assert!(try_run_eval(BigInt256::min_value()).is_err());
    }
}
