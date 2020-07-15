//! Operators in ErgoTree
#[derive(PartialEq, Eq, Debug)]
/// Operations for numerical types
pub enum NumOp {
    /// Addition
    Add,
}

#[derive(PartialEq, Eq, Debug)]
/// Binary operations
pub enum BinOp {
    /// Binary operations for numerical types
    Num(NumOp),
}
