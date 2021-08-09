use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::deserialize_context::DeserializeContext;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for DeserializeContext {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        match ctx.ctx.extension.values.get(&self.id) {
            Some(c) => {
                if c.tpe != SType::SColl(SType::SByte.into()) {
                    Err(EvalError::UnexpectedExpr(format!("DeserializeContext:: expected extension value to have type SColl(SByte), got {:?}", c.tpe)))
                } else {
                    let bytes = c.v.clone().try_extract_into::<Vec<u8>>()?;
                    let expr = Expr::sigma_parse_bytes(bytes.as_slice())?;
                    expr.eval(env, ctx)
                }
            }
            None => Err(EvalError::NotFound(format!(
                "DeserializeContext: no value with id {} in context extension",
                self.id
            ))),
        }
    }
}
