//! Ergo constant values

/// Ergo constant(evaluated) values
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant(pub ergo_lib::ergotree_ir::mir::constant::Constant);
pub type ConstantPtr = *mut Constant;
pub type ConstConstantPtr = *const Constant;
