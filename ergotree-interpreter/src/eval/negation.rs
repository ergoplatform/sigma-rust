use ergotree_ir::mir::negation::Negation;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use num_traits::CheckedNeg;

impl Evaluable for Negation {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;

        fn overflow_err<T: core::fmt::Display>(v: &T) -> EvalError {
            EvalError::ArithmeticException(format!("Overflow on Negation of value {}", *v))
        }
        match input_v {
            // Fixed-width signed ints negate with two's-complement wrap
            // (`-MIN == MIN`), matching sigma-state's unchecked numeric
            // `negate` (`ast/trees.scala` Negation.eval) — negating MIN is
            // not an error on the JVM.
            Value::Byte(v) => Ok(v.wrapping_neg().into()),
            Value::Short(v) => Ok(v.wrapping_neg().into()),
            Value::Int(v) => Ok(v.wrapping_neg().into()),
            Value::Long(v) => Ok(v.wrapping_neg().into()),
            // BigInt256 stays checked: sigma-state's 256-bit BigInt also
            // overflows negating its MIN (`-(2^255)`), so both sides error.
            Value::BigInt(v) => v
                .checked_neg()
                .map(|v| v.into())
                .ok_or_else(|| overflow_err(&v)),
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
    use crate::eval::test_util::try_eval_out_wo_ctx;
    use ergotree_ir::bigint256::BigInt256;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::constant::TryExtractFrom;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use num_traits::{Bounded, Num};

    fn try_run_eval<T: Num + Into<Constant> + TryExtractFrom<Value<'static>> + 'static>(
        input: T,
    ) -> Result<T, EvalError> {
        let expr: Expr = Negation::try_build(Expr::Const(input.into()))
            .unwrap()
            .into();
        try_eval_out_wo_ctx::<T>(&expr)
    }
    fn run_eval<T: Num + Into<Constant> + TryExtractFrom<Value<'static>> + 'static>(input: T) -> T {
        try_run_eval(input).unwrap()
    }

    #[test]
    fn eval() {
        assert_eq!(run_eval(1i8), -1i8);
        assert_eq!(run_eval(1i16), -1i16);
        assert_eq!(run_eval(1i32), -1i32);
        assert_eq!(run_eval(1i64), -1i64);
        assert_eq!(run_eval(BigInt256::from(1i64)), BigInt256::from(-1i64));
    }

    // JVM parity (sigma-state 6.0.3, LanguageSpecificationV5): negating a
    // fixed-width MIN_VALUE two's-complement-wraps to itself (`-MIN == MIN`)
    // with no error — sigma-state's `Negation.eval` uses the unchecked
    // numeric `negate`. Repros: santa `vectors/eval/v5/Numeric_Negation_
    // equivalence.json` (`-128#0`, `-32768#9`, `-2147483648#18`,
    // `-9223372036854775808#26`). `BigInt256` is the exception — its 256-bit
    // MIN (`-(2^255)`) overflows on both sides, so it still errors.
    #[test]
    fn negation_of_min_value_wraps_to_self() {
        assert_eq!(run_eval(i8::MIN), i8::MIN);
        assert_eq!(run_eval(i16::MIN), i16::MIN);
        assert_eq!(run_eval(i32::MIN), i32::MIN);
        assert_eq!(run_eval(i64::MIN), i64::MIN);
        assert!(try_run_eval(BigInt256::min_value()).is_err());
    }
}
