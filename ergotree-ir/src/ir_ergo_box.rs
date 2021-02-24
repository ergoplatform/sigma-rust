use crate::mir::constant::Constant;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IrBoxId([u8; 32]);

impl IrBoxId {
    pub fn new(id: [u8; 32]) -> Self {
        IrBoxId(id)
    }

    pub fn get_box(
        &self,
        arena: &Rc<dyn IrErgoBoxArena>,
    ) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError> {
        arena.get(self)
    }
}

pub trait IrErgoBoxArena: Debug {
    fn get(&self, id: &IrBoxId) -> Result<Rc<dyn IrErgoBox>, IrErgoBoxArenaError>;
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("IrErgoBoxArenaError: {0}")]
pub struct IrErgoBoxArenaError(pub String);

pub trait IrErgoBox: Debug {
    fn id(&self) -> &[u8; 32];
    fn value(&self) -> i64;
    fn tokens(&self) -> Vec<(Vec<i8>, i64)>;
    /// R4-R9 optional registere, where element with index 0 is R4, etc.
    fn additional_registers(&self) -> &[Constant];
    fn get_register(&self, id: i8) -> Option<Constant>;
    fn creation_height(&self) -> i32;
    fn script_bytes(&self) -> Vec<u8>;
}
