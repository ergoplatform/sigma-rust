//! ErgoBox representation in IR
use crate::mir::constant::Constant;
use crate::serialization::SigmaSerializationError;
use sigma_util::DIGEST32_SIZE;
use std::fmt::Debug;

/// Ergo box id
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct IrBoxId(pub [i8; DIGEST32_SIZE]);

impl IrBoxId {
    /// Make new box id
    pub fn new(id: [i8; DIGEST32_SIZE]) -> Self {
        IrBoxId(id)
    }

    /// Returns id as byte array
    pub fn to_bytes(&self) -> Vec<i8> {
        self.0.to_vec()
    }
}

/// Ergo box properties
pub trait IrErgoBox: Debug {
    /// Box id
    fn id(&self) -> IrBoxId;
    /// Box value
    fn value(&self) -> i64;
    /// Box tokens
    fn tokens_raw(&self) -> Vec<(Vec<i8>, i64)>;
    /// R4-R9 optional registers, where element with index 0 is R4, etc.
    fn additional_registers(&self) -> &[Constant];
    /// Returns a register value for the given register index (0 is R0, 9 is R9)
    fn get_register(&self, id: i8) -> Option<Constant>;
    /// Box creation height
    fn creation_height(&self) -> i32;
    /// Box guarding script serialized
    fn script_bytes(&self) -> Result<Vec<i8>, SigmaSerializationError>;
    /// Tuple of height when block got included into the blockchain and transaction identifier with
    /// box index in the transaction outputs serialized to the byte array.
    fn creation_info(&self) -> (i32, Vec<i8>);
    /// Box serialized bytes
    fn bytes(&self) -> Result<Vec<i8>, SigmaSerializationError>;
    /// Box serialized bytes excluding txId & index
    fn bytes_without_ref(&self) -> Result<Vec<i8>, SigmaSerializationError>;
}

impl PartialEq for dyn IrErgoBox + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
