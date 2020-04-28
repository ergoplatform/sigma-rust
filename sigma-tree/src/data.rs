#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
//! Sigma data
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::io;
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

pub struct RegisterId(u8);

pub enum ConstantKind {
    SBoolean(bool),
    SByte(i8),
    SShort(i16),
    SInt(i32),
    SLong(i64),
    SBigInt,       // TODO: find underlying type
    SGroupElement, // TODO: find/make underlying type
    SSigmaProp(Box<dyn SigmaProp>),
    SBox(Box<dyn SigmaBox>),
    SAvlTree, // TODO: make underlying type
    SColl(Vec<ConstantKind>),
    STup(Vec<ConstantKind>),
}

impl SigmaSerializable for ConstantKind {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        todo!()
    }
    fn sigma_parse<R: ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError> {
        todo!()
    }
}

pub enum SigmaBoolean {
    ProveDlog(u64),
    CAND(Vec<SigmaBoolean>),
}

pub trait SigmaProp {
    fn is_valid(&self) -> bool;
}

pub struct CSigmaProp {
    pub sigma_tree: SigmaBoolean,
}

impl SigmaProp for CSigmaProp {
    fn is_valid(&self) -> bool {
        todo!()
    }
}

pub trait SigmaBox {
    fn value(&self) -> u64;
}
pub struct CSigmaBox {}
impl SigmaBox for CSigmaBox {
    fn value(&self) -> u64 {
        0
    }
}
