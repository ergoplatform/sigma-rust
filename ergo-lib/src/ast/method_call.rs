use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodCall {
    pub tpe: SType,
    pub obj: Box<Expr>,
    pub method: SMethod,
    pub args: Vec<Expr>,
}
