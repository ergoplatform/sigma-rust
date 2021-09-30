use ergotree_ir::mir::extract_bytes_with_no_ref::ExtractBytesWithNoRef;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ExtractBytesWithNoRef {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::CBox(b) => Ok(b.bytes_without_ref()?.into()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Expected ExtractBytesWithNoRef input to be Value::CBox, got {0:?}",
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
        let e: Expr = ExtractBytesWithNoRef {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<i8>>(&e, ctx.clone()),
            ctx.self_box.bytes_without_ref().unwrap()
        );
    }
}
