use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for CreateProveDhTuple {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let g = self.g.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let h = self.h.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let u = self.u.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let v = self.v.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        Ok(ProveDhTuple::new(g, h, u, v).into())
    }
}
