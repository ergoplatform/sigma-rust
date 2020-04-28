#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::{
    data::{ConstantKind, RegisterId},
    types::*,
};
use serializer::SerializationError;
use sigma_ser::{
    serializer::{self, SigmaSerializable},
    vlq_encode,
};
use std::{io, marker::PhantomData};
use ExprKind::*;

pub struct OpCode(u8);

pub struct Expr {
    pub tpe: SType,
    pub kind: ExprKind,
}
pub enum ExprKind {
    Constant(ConstantKind),
    Coll(Vec<Expr>),
    Tup(Vec<Expr>),
    PredefFunc(PredefFunc),
    CollM(CollMethods),
    BoxM(BoxMethods),
    CtxM(ContextMethods),
    MethodCall {
        obj: Box<Expr>,
        method: SMethod,
        args: Vec<Expr>,
    },
}

impl Expr {
    pub fn op_code(&self) -> OpCode {
        match &self.kind {
            Constant(_) => todo!(),
            Coll(_) => todo!(),
            Tup(_) => todo!(),
            BoxM(boxm) => boxm.op_code(),
            CollM(_) => todo!(),
            CtxM(_) => todo!(),
            MethodCall { .. } => todo!(),
            PredefFunc(_) => todo!(),
        }
    }
}

pub enum CollMethods {
    Fold {
        input: Box<Expr>,
        zero: Box<Expr>,
        fold_op: Box<Expr>,
    },
}

pub enum BoxMethods {
    ExtractRegisterAs {
        input: Box<Expr>,
        register_id: RegisterId,
    },
}

impl BoxMethods {
    pub fn op_code(&self) -> OpCode {
        todo!()
    }
}

pub enum ContextMethods {
    Inputs,
    Outputs,
}

pub enum PredefFunc {
    Sha256 { input: Box<Expr> },
}

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        todo!()
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError> {
        todo!();
    }
}
