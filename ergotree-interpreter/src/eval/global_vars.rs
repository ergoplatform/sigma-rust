use crate::eval::Env;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::dlog_group;

use super::EvalContext;
use super::EvalError;
use super::Evaluable;

impl Evaluable for GlobalVars {
    fn eval(&self, _env: &Env, ectx: &mut EvalContext) -> Result<Value, EvalError> {
        match self {
            GlobalVars::Height => Ok((ectx.ctx.height as i32).into()),
            GlobalVars::SelfBox => Ok(ectx.ctx.self_box.clone().into()),
            GlobalVars::Outputs => Ok(ectx.ctx.outputs.clone().into()),
            GlobalVars::Inputs => Ok(ectx.ctx.inputs.clone().into()),
            GlobalVars::MinerPubKey => {
                Ok(ectx.ctx.pre_header.miner_pk.sigma_serialize_bytes()?.into())
            }
            GlobalVars::GroupGenerator => Ok(dlog_group::generator().into()),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use ergoscript_compiler::compiler::compile_expr;
    use ergoscript_compiler::script_env::ScriptEnv;
    use ergotree_ir::ir_ergo_box::IrErgoBox;
    use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
    use sigma_test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_height() {
        let ctx = Rc::new(force_any_val::<Context>());
        let expr = compile_expr("HEIGHT", ScriptEnv::new()).unwrap();
        assert_eq!(eval_out::<i32>(&expr, ctx.clone()), ctx.height as i32);
    }

    #[test]
    fn eval_self_box() {
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Rc<dyn IrErgoBox>>(&GlobalVars::SelfBox.into(), ctx.clone()).as_ref(),
            ctx.self_box.as_ref()
        );
    }

    #[test]
    fn eval_outputs() {
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<Rc<dyn IrErgoBox>>>(&GlobalVars::Outputs.into(), ctx.clone()),
            ctx.outputs
        );
    }

    #[test]
    fn eval_inputs() {
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<Rc<dyn IrErgoBox>>>(&GlobalVars::Inputs.into(), ctx.clone()),
            ctx.inputs
        );
    }

    #[test]
    fn eval_group_generator() {
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<EcPoint>(&GlobalVars::GroupGenerator.into(), ctx),
            dlog_group::generator()
        );
    }
}
