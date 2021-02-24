use crate::mir::method_call::MethodCall;
use crate::mir::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for MethodCall {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        let ov = self.obj.eval(env, ectx)?;
        let argsv: Result<Vec<Value>, EvalError> =
            self.args.iter().map(|arg| arg.eval(env, ectx)).collect();
        self.method.eval_fn()(ectx.ctx.clone(), ov, argsv?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::mir::constant::Constant;
    use crate::mir::expr::Expr;
    use crate::mir::global_vars::GlobalVars;
    use crate::mir::option_get::OptionGet;
    use crate::test_util::force_any_val;
    use crate::types::sbox;

    use super::*;

    #[test]
    fn eval_box_get_reg() {
        let mc: Expr = MethodCall {
            obj: Box::new(GlobalVars::SelfBox.into()),
            method: sbox::GET_REG_METHOD.clone(),
            args: vec![Constant::from(0i8).into()],
        }
        .into();
        let option_get_expr: Expr = OptionGet::new(mc).unwrap().into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i64>(&option_get_expr, ctx.clone()),
            ctx.self_box.get_box(&ctx.box_arena).unwrap().value()
        );
    }
}
