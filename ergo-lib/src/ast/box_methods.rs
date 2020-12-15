use crate::{serialization::op_code::OpCode, types::stype::SType};

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
/// newtype for box register id
pub struct RegisterId(u8); // should be a sum of NonMandatoryRegisterId and MandatoryRegisterId

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Box type instance
pub enum BoxM {
    /// Box.RX methods (get register value)
    ExtractRegisterAs {
        /// Box
        input: Box<Expr>,
        /// Register id to extract value from
        register_id: RegisterId,
        /// Type
        tpe: SType,
    },
}

impl BoxM {
    /// Code (serialization)
    pub fn op_code(&self) -> OpCode {
        todo!()
    }
}
