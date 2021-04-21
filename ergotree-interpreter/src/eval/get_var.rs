// use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::get_var::GetVar;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::mir::constant::TryExtractFromError;

impl Evaluable for GetVar {
    fn eval(&self, _env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        match ctx.ctx.var_map.get(&self.var_id) {
            None => Ok(Value::Opt(None.into())),
            Some(v) if v.tpe == self.var_tpe => Ok((Some(v.v.clone())).into()),
            Some(v) => Err(TryExtractFromError(format!(
                "GetVar: expected {:?}, found {:?}",
                self.var_tpe, v.tpe
            ))
            .into()),
        }
    }
}
