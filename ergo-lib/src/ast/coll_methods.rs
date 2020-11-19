use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Methods for Collection type instance
pub enum CollM {
    /// Fold method
    Fold {
        /// Collection
        input: Box<Expr>,
        /// Initial value for accumulator
        zero: Box<Expr>,
        /// Function (lambda)
        fold_op: Box<Expr>,
    },
}
