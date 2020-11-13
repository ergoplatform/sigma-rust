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
