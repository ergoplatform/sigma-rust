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
use SType::*;

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
