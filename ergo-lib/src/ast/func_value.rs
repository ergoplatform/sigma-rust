use crate::types::stype::SType;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncArg {
    pub idx: i32,
    pub tpe: SType,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncValue {
    pub args: Vec<FuncArg>,
    pub body: Expr,
}
