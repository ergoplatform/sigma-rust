//! ErgoBox representation in IR
use crate::mir::constant::Constant;
use sigma_util::DIGEST32_SIZE;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error;

/// Ergo box id
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct IrBoxId(pub [u8; DIGEST32_SIZE]);

impl IrBoxId {
    /// Make new box id
    pub fn new(id: [u8; DIGEST32_SIZE]) -> Self {
        IrBoxId(id)
    }

    /// Gets box with this id from box arena
    pub fn get_box(
        &self,
        arena: &Rc<dyn IrErgoBoxArena>,
    ) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError> {
        arena.get(self)
    }
}

/// Arena (store) for boxes
pub trait IrErgoBoxArena: Debug {
    /// Returns a box with the given id
    fn get(&self, id: &IrBoxId) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError>;
}

/// Box arena error
#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("IrErgoBoxArenaError: {0}")]
pub struct IrErgoBoxArenaError(pub String);

/// Ergo box properties
pub trait IrErgoBox: Debug {
    /// Box id
    fn id(&self) -> &[u8; DIGEST32_SIZE];
    /// Box value
    fn value(&self) -> i64;
    /// Box tokens
    fn tokens(&self) -> Vec<(Vec<i8>, i64)>;
    /// R4-R9 optional registers, where element with index 0 is R4, etc.
    fn additional_registers(&self) -> &[Constant];
    /// Returns a register value for the given register index (0 is R0, 9 is R9)
    fn get_register(&self, id: i8) -> Option<Constant>;
    /// Box creation height
    fn creation_height(&self) -> i32;
    /// Box guarding script serialized
    fn script_bytes(&self) -> Vec<u8>;
}
