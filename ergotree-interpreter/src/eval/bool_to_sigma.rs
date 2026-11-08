use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for BoolToSigmaProp {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(15)?; // BoolToSigmaProp = Fixed(15)
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bool = input_v.try_extract_into::<bool>()?;
        Ok((SigmaProp::new(SigmaBoolean::TrivialProp(input_v_bool))).into())
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out_wo_ctx;
    use ergotree_ir::mir::expr::Expr;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn eval(b in any::<bool>()) {
            let expr: Expr = BoolToSigmaProp {input: Expr::Const(b.into()).into()}.into();
            let res = eval_out_wo_ctx::<SigmaProp>(&expr);
            prop_assert_eq!(res, SigmaProp::new(SigmaBoolean::TrivialProp(b)));
        }
    }
}
