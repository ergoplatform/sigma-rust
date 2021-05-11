use ergotree_ir::mir::select_field::SelectField;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SelectField {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            Value::Tup(items) => items
                .get(self.field_index.zero_based_index())
                .cloned()
                .ok_or_else(|| {
                    EvalError::NotFound(format!(
                        "SelectField field index is out of bounds. Index: {0:?}, tuple: {1:?}",
                        self.field_index, items
                    ))
                }),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected SelectField input to be Value::Tup, got: {0:?}",
                input_v
            ))),
        }
    }
}
