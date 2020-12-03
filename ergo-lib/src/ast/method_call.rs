use crate::eval::context::Context;
use crate::eval::cost_accum::CostAccumulator;
use crate::eval::Env;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodCall {
    pub tpe: SType,
    pub obj: Box<Expr>,
    pub method: SMethod,
    pub args: Vec<Expr>,
}

impl Evaluable for MethodCall {
    fn eval(&self, env: &Env, ca: &mut CostAccumulator, ctx: &Context) -> Result<Value, EvalError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::ergo_box::ErgoBox;
    use crate::eval::tests::eval_out;
    use crate::test_util::force_any_val;
    use crate::types::scontext;

    use super::*;

    #[test]
    fn eval_context_data_inputs() {
        let mc = MethodCall {
            tpe: scontext::DATA_INPUTS_METHOD.tpe().clone(),
            obj: Box::new(Expr::Context),
            method: scontext::DATA_INPUTS_METHOD.clone(),
            args: vec![],
        };
        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<Vec<ErgoBox>>(&mc.into(), &ctx), ctx.data_inputs);
    }
}
