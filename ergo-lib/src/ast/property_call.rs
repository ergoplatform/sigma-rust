use crate::serialization::op_code::OpCode;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PropertyCall {
    pub obj: Box<Expr>,
    pub method: SMethod,
}

impl PropertyCall {
    pub fn tpe(&self) -> &SType {
        self.method.tpe()
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::PROPERTY_CALL
    }
}
