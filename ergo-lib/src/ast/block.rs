use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

use super::expr::Expr;
use super::val_def::ValDef;
use super::value::Value;

/** The order of ValDefs in the block is used to assign ids to ValUse(id) nodes
 * For all i: items(i).id == {number of ValDefs preceded in a graph} with respect to topological order.
 * Specific topological order doesn't really matter, what is important is to preserve semantic linkage
 * between ValUse(id) and ValDef with the corresponding id.
 * This convention allow to valid serializing ids because we always serializing and deserializing
 * in a fixed well defined order.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockValue {
    pub items: Vec<ValDef>,
    pub result: Expr,
}

impl Evaluable for BlockValue {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let mut cur_env = env.clone();
        for i in self.items.iter() {
            let v: Value = i.rhs.eval(&cur_env, ctx)?;
            cur_env.insert(i.id, v);
        }
        self.result.eval(&cur_env, ctx)
    }
}
