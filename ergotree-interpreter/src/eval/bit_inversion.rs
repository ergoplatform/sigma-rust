use ergotree_ir::mir::bit_inversion::BitInversion;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for BitInversion {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::Byte(v) => Ok(Value::Byte(!v)),
            Value::Short(v) => Ok(Value::Short(!v)),
            Value::Int(v) => Ok(Value::Int(!v)),
            Value::Long(v) => Ok(Value::Long(!v)),
            Value::BigInt(v) => Ok(Value::BigInt(!v)),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected BitInversion input to be numeric value, got {:?}",
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
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use num_traits::Num;

    fn run_eval<T: Num + Into<Constant> + TryExtractFrom<Value>>(input: T) -> T {
        let expr: Expr = BitInversion::try_build(Expr::Const(input.into()))
            .unwrap()
            .into();
        try_eval_out_wo_ctx::<T>(&expr).unwrap()
    }

    #[test]
    fn eval() {
        assert_eq!(run_eval(0i8), -1i8);
        assert_eq!(run_eval(1i8), -2i8);
        assert_eq!(run_eval(i8::MIN), 127i8);
        assert_eq!(run_eval(i8::MAX), -128i8);

        assert_eq!(run_eval(0i16), -1i16);
        assert_eq!(run_eval(1i16), -2i16);
        assert_eq!(run_eval(i16::MIN), 32767i16);
        assert_eq!(run_eval(i16::MAX), -32768i16);

        assert_eq!(run_eval(0i32), -1i32);
        assert_eq!(run_eval(1i32), -2i32);
        assert_eq!(run_eval(i32::MIN), 2147483647i32);
        assert_eq!(run_eval(i32::MAX), -2147483648i32);

        assert_eq!(run_eval(0i64), -1i64);
        assert_eq!(run_eval(1i64), -2i64);
        assert_eq!(run_eval(i64::MIN), 9223372036854775807i64);
        assert_eq!(run_eval(i64::MAX), -9223372036854775808i64);

        assert_eq!(run_eval(BigInt256::from(0i64)), BigInt256::from(-1i64));
        assert_eq!(run_eval(BigInt256::from(1i64)), BigInt256::from(-2i64));
        assert_eq!(
            run_eval(BigInt256::from(i64::MIN)),
            BigInt256::from(9223372036854775807i64)
        );
        assert_eq!(
            run_eval(BigInt256::from(i64::MAX)),
            BigInt256::from(-9223372036854775808i64)
        );
    }
}
