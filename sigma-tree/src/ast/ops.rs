//! Operators in ErgoTree
#[derive(PartialEq, Eq, Debug, Clone)]
/// Operations for numerical types
pub enum NumOp {
    /// Addition
    Add,
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Binary operations
pub enum BinOp {
    /// Binary operations for numerical types
    Num(NumOp),
}
