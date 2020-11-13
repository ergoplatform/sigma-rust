use crate::serialization::op_code::OpCode;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
/// newtype for box register id
pub struct RegisterId(u8); // should be a sum of NonMandatoryRegisterId and MandatoryRegisterId

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Box type instance
pub enum BoxM {
    /// Box.RX methods
    ExtractRegisterAs {
        /// Box
        input: Box<Expr>,
        /// Register id to extract value from
        register_id: RegisterId,
    },
}

impl BoxM {
    /// Code (serialization)
    pub fn op_code(&self) -> OpCode {
        todo!()
    }
}
