use crate::types::stype_param::STypeVar;

use super::expr::Expr;

/** IR node for let-bound expressions `let x = rhs` which is ValDef, or `let f[T] = rhs` which is FunDef.
 * These nodes are used to represent ErgoTrees after common sub-expression elimination.
 * This representation is more compact in serialized form.
 * @param id unique identifier of the variable in the current scope. */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValDef {
    pub id: i32,
    pub tpe_args: Vec<STypeVar>,
    pub rhs: Expr,
}
