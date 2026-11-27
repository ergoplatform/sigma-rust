use ergotree_ir::mir::bit_inversion::BitInversion;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for BitInversion {
    fn eval<'ctx>(
        &self,
        _env: &mut Env<'ctx>,
        _ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        // Matches JVM `sigma.ast.BitInversion`: does not override `Value.eval`
        // (v6 bitwise inversion uses `SNumeric.bitwiseInverse` MethodCall).
        Err(EvalError::NotFound(
            "BitInversion evaluation is not supported".into(),
        ))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::try_eval_out_wo_ctx;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;

    #[test]
    fn eval_is_unsupported() {
        let expr: Expr = BitInversion::try_build(Expr::Const(1i32.into()))
            .unwrap()
            .into();
        assert!(try_eval_out_wo_ctx::<i32>(&expr).is_err());
    }
}
