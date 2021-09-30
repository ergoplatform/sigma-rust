use ergotree_ir::mir::method_call::MethodCall;
use ergotree_ir::mir::value::Value;

use super::smethod_eval_fn;
use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for MethodCall {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        let ov = self.obj.eval(env, ectx)?;
        let argsv: Result<Vec<Value>, EvalError> =
            self.args.iter().map(|arg| arg.eval(env, ectx)).collect();
        smethod_eval_fn(&self.method)?(env, ectx, ov, argsv?)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::rc::Rc;

    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::option_get::OptionGet;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::types::sbox;
    use sigma_test_util::force_any_val;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;

    use super::*;

    #[test]
    fn eval_box_get_reg() {
        let mc: Expr = MethodCall::new(
            GlobalVars::SelfBox.into(),
            sbox::GET_REG_METHOD.clone(),
            vec![Constant::from(0i8).into()],
        )
        .unwrap()
        .into();
        let option_get_expr: Expr = OptionGet::try_build(mc).unwrap().into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i64>(&option_get_expr, ctx.clone()),
            ctx.self_box.value()
        );
    }
}
