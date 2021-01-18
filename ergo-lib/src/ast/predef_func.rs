use crate::types::stype::SType;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Predefined (global) functions
pub enum PredefFunc {
    /// SHA256
    Sha256 {
        /// Byte array
        input: Box<Expr>,
    },
}

impl PredefFunc {
    pub fn tpe(&self) -> SType {
        match self {
            PredefFunc::Sha256 { input: _ } => todo!(),
        }
    }
}
