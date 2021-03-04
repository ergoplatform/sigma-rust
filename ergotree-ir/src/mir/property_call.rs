use crate::serialization::op_code::OpCode;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;

/// Invocation of object's property
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PropertyCall {
    /// Object on which property will be invoked
    pub obj: Box<Expr>,
    /// Property to be invoked
    pub method: SMethod,
}

impl PropertyCall {
    /// Type
    pub fn tpe(&self) -> SType {
        self.method.tpe().clone()
    }

    pub(crate) fn op_code(&self) -> OpCode {
        OpCode::PROPERTY_CALL
    }
}
