use ergo_chain_types::EcPoint;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDhTuple;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for CreateProveDhTuple {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(20)?; // CreateProveDHTuple = Fixed(20)
        let g = self.g.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let h = self.h.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let u = self.u.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let v = self.v.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        Ok(ProveDhTuple::new(g, h, u, v).into())
    }
}
