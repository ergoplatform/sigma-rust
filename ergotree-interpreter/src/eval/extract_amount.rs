use ergotree_ir::mir::extract_amount::ExtractAmount;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractAmount {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(Value::Long(b.value())),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractAmount input to be Value::CBox, got {0:?}",
                input_v
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use sigma_test_util::force_any_val;
    use std::rc::Rc;

    #[test]
    fn eval() {
        let e: Expr = ExtractAmount {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(eval_out::<i64>(&e, ctx.clone()), ctx.self_box.value());
    }
}
