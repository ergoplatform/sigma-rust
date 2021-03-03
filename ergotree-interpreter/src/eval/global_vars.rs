use crate::eval::Env;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::value::Value;

use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for GlobalVars {
    fn eval(&self, _env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        match self {
            GlobalVars::Height => Ok(ectx.ctx.height.clone().into()),
            GlobalVars::SelfBox => Ok(ectx.ctx.self_box.clone().into()),
            GlobalVars::Outputs => Ok(ectx.ctx.outputs.clone().into()),
            _ => Err(EvalError::UnexpectedExpr(format!(
                "Don't know how to eval GlobalVars: {0:?}",
                self
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergotree_ir::ir_ergo_box::IrBoxId;
    use test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_height() {
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i32>(&GlobalVars::Height.into(), ctx.clone()),
            ctx.height
        );
    }

    #[test]
    fn eval_self_box() {
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<IrBoxId>(&GlobalVars::SelfBox.into(), ctx.clone()),
            ctx.self_box
        );
    }

    #[test]
    fn eval_outputs() {
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<IrBoxId>>(&GlobalVars::Outputs.into(), ctx.clone()),
            ctx.outputs
        );
    }
}
