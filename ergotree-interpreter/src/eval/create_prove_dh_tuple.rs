use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::create_prove_dh_tuple::CreateProveDHTuple;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for CreateProveDHTuple {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let gv = self.gv.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let hv = self.hv.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let uv = self.uv.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        let vv = self.vv.eval(env, ctx)?.try_extract_into::<EcPoint>()?;
        Ok(ProveDhTuple {
            gv: gv.into(),
            hv: hv.into(),
            uv: uv.into(),
            vv: vv.into(),
        }
        .into())
    }
}
