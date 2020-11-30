use crate::eval::cost_accum::CostAccumulator;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;

use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Context type instance
pub enum ContextM {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
    /// ErgoBox instance, which script is being evaluated
    SelfBox,
    /// Tx data inputs
    DataInputs,
}

impl ContextM {
    pub fn op_code(&self) -> OpCode {
        match self {
            ContextM::SelfBox => OpCode::SELF_BOX,
            _ => todo!(),
        }
    }
}

impl Evaluable for ContextM {
    fn eval(
        &self,
        _env: &crate::eval::Env,
        _ca: &mut CostAccumulator,
        ctx: &crate::eval::context::Context,
    ) -> Result<Value, EvalError> {
        match self {
            ContextM::Height => Ok(ctx.height.clone().into()),
            ContextM::SelfBox => Ok(ctx.self_box.clone().into()),
            ContextM::Outputs => Ok(ctx.outputs.clone().into()),
            ContextM::DataInputs => Ok(ctx.data_inputs.clone().into()),
            _ => Err(EvalError::UnexpectedExpr),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::constant::TryExtractFrom;
    use crate::chain::ergo_box::ErgoBox;
    use crate::eval::context::Context;
    use crate::eval::Env;
    use crate::test_util::force_any_val;

    use super::*;

    fn eval<T: TryExtractFrom<Value>>(v: ContextM, ctx: &Context) -> T {
        use crate::ast::constant::TryExtractInto;
        let mut ca = CostAccumulator::new(0, None);
        v.eval(&Env::empty(), &mut ca, ctx)
            .unwrap()
            .try_extract_into::<T>()
            .unwrap()
    }

    #[test]
    fn eval_height() {
        let ctx = force_any_val::<Context>();
        assert_eq!(eval::<i32>(ContextM::Height, &ctx), ctx.height);
    }

    #[test]
    fn eval_self_box() {
        let ctx = force_any_val::<Context>();
        assert_eq!(eval::<ErgoBox>(ContextM::SelfBox, &ctx), ctx.self_box);
    }

    #[test]
    fn eval_outputs() {
        let ctx = force_any_val::<Context>();
        assert_eq!(eval::<Vec<ErgoBox>>(ContextM::Outputs, &ctx), ctx.outputs);
    }

    #[test]
    fn eval_data_inputs() {
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval::<Vec<ErgoBox>>(ContextM::DataInputs, &ctx),
            ctx.data_inputs
        );
    }
}
