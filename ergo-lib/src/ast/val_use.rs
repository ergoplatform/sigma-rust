use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::types::stype::SType;

use super::value::Value;

/** Special node which represents a reference to ValDef in was introduced as result of CSE. */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValUse {
    pub val_id: u32,
    pub tpe: SType,
}

impl Evaluable for ValUse {
    fn eval(&self, env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        env.get(self.val_id)
            .cloned()
            .ok_or_else(|| EvalError::NotFound(format!("no value in env for id: {0}", self.val_id)))
    }
}
