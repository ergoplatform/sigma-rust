use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::value::Lambda;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for FuncValue {
    fn eval<'ctx>(&self, _env: &mut Env, _ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        // Sigma-state's type check also rejects a one-item tuple lambda argument.
        if let Some(arg) = self
            .args()
            .iter()
            .find(|a| matches!(&a.tpe, SType::STuple(t) if t.items.len() < 2))
        {
            return Err(EvalError::Misc(format!(
                "Unsupported tuple type {:?} of lambda argument",
                arg.tpe
            )));
        }
        Ok(Value::Lambda(Lambda {
            args: self.args().to_vec(),
            body: self.body().clone().into(),
        }))
    }
}
