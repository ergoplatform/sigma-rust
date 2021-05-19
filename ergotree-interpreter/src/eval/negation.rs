use ergotree_ir::mir::negation::Negation;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Negation {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::Byte(v) => Ok((-v).into()),
            Value::Short(v) => Ok((-v).into()),
            Value::Int(v) => Ok((-v).into()),
            Value::Long(v) => Ok((-v).into()),
            Value::BigInt(v) => Ok((-v).into()),
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
    use crate::eval::tests::eval_out_wo_ctx;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::constant::TryExtractFrom;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::unary_op::UnaryOpTryBuild;
    use num_bigint::ToBigInt;
    use num_traits::Num;

    fn run_eval<T: Num + Into<Constant> + TryExtractFrom<Value>>(input: T) -> T {
        let expr: Expr = Negation::try_build(Expr::Const(input.into()))
            .unwrap()
            .into();
        eval_out_wo_ctx::<T>(&expr)
    }

    #[test]
    fn eval() {
        assert_eq!(run_eval(1i8), -1i8);
        assert_eq!(run_eval(1i16), -1i16);
        assert_eq!(run_eval(1i32), -1i32);
        assert_eq!(run_eval(1i64), -1i64);
        assert_eq!(
            run_eval(1i64.to_bigint().unwrap()),
            (-1i64).to_bigint().unwrap()
        );
    }
}
