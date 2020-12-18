use crate::ast::method_call::MethodCall;
use crate::ast::value::Value;

use super::Env;
use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for MethodCall {
    fn eval(&self, env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        let ov = (*self.obj).eval(env, ectx)?;
        let argsv: Result<Vec<Value>, EvalError> =
            self.args.iter().map(|arg| arg.eval(env, ectx)).collect();
        self.method.eval_fn()(ov, argsv?)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::constant::Constant;
    use crate::ast::expr::Expr;
    use crate::ast::global_vars::GlobalVars;
    use crate::ast::option_get::OptionGet;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
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
        let option_get_expr: Expr = Box::new(OptionGet { input: mc }).into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i64>(&option_get_expr, ctx.clone()),
            ctx.self_box.value.as_i64()
        );
    }
}
