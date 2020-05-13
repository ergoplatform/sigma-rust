#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
//! Sigma data
use crate::{ast::Expr, types::SType};
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::{any::Any, io};
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use ConstantKind::*;
use SType::*;

pub struct RegisterId(u8);

pub enum CCollPrim {
    CCollBoolean(Vec<bool>),
    CCollByte(Vec<i8>),
    CCollShort(Vec<i16>),
    CCollInt(Vec<i32>),
    CCollLong(Vec<i64>),
}

pub enum ConstantKind {
    CBoolean(bool),
    CByte(i8),
    CShort(i16),
    CInt(i32),
    CLong(i64),
    CBigInt,       // TODO: find underlying type
    CGroupElement, // TODO: find/make underlying type
    CSigmaProp(Box<dyn SigmaProp>),
    CBox(Box<dyn SigmaBox>),
    CAvlTree, // TODO: make underlying type
    CCollPrim(CCollPrim),
    CColl(Vec<ConstantKind>),
    CTup(Vec<ConstantKind>),
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
