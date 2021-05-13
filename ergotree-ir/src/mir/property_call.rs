use crate::serialization::op_code::OpCode;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use crate::has_opcode::HasStaticOpCode;

/// Invocation of object's property
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PropertyCall {
    /// Object on which property will be invoked
    pub obj: Box<Expr>,
    /// Property to be invoked
    pub method: SMethod,
}

impl PropertyCall {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(obj: Expr, method: SMethod) -> Result<Self, InvalidArgumentError> {
        if method.tpe().t_dom.len() != 1 {
            return Err(InvalidArgumentError(format!(
                "PropertyCall: expected method.t_dom to have 1 element, got {}",
                method.tpe().t_dom.len()
            )));
        }
        let expected_types = vec![obj.tpe()];
        if !method
            .tpe()
            .t_dom
            .iter()
            .zip(&expected_types)
            .all(|(expected, actual)| expected == actual)
        {
            return Err(InvalidArgumentError(format!(
                "MethodCall: expected types {:?} do not match provided obj and args types {:?}",
                method.tpe().t_dom,
                expected_types
            )));
        }
        Ok(Self {
            obj: obj.into(),
            method,
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        *self.method.tpe().t_range.clone()
    }
}

impl HasStaticOpCode for PropertyCall {
    const OP_CODE: OpCode = OpCode::PROPERTY_CALL;
}
