#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::data::{SigmaBox, SigmaProp};
use crate::{serialization::op_code::OpCode, types::*};
use core::fmt;
use io::{Read, Write};
use serializer::SerializationError;
use sigma_ser::{
    serializer::{self, SigmaSerializable},
    vlq_encode,
};
use std::{collections::HashMap, io, marker::PhantomData, rc::Rc};
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use Expr::*;

pub mod ops;

pub struct RegisterId(u8);

pub enum CollPrim {
    CollBoolean(Vec<bool>),
    CollByte(Vec<i8>),
    CollShort(Vec<i16>),
    CollInt(Vec<i32>),
    CollLong(Vec<i64>),
}

pub enum Const {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    BigInt,       // TODO: find underlying type
    GroupElement, // TODO: find/make underlying type
    SigmaProp(Box<dyn SigmaProp>),
    CBox(Box<dyn SigmaBox>),
    AvlTree, // TODO: make underlying type
    CollPrim(CollPrim),
    Coll(Vec<Const>),
    Tup(Vec<Const>),
}

pub enum Expr {
    Constant {
        tpe: SType,
        v: Const,
    },
    Coll {
        tpe: SType,
        v: Vec<Expr>,
    },
    Tup {
        tpe: SType,
        v: Vec<Expr>,
    },
    PredefFunc(PredefFunc),
    CollM(CollMethods),
    BoxM(BoxMethods),
    CtxM(ContextMethods),
    MethodCall {
        tpe: SType,
        obj: Box<Expr>,
        method: SMethod,
        args: Vec<Expr>,
    },
    BinOp(ops::BinOp, Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn op_code(&self) -> OpCode {
        match self {
            Constant { .. } => todo!(),
            Coll { .. } => todo!(),
            Tup { .. } => todo!(),
            BoxM(boxm) => boxm.op_code(),
            CollM(_) => todo!(),
            CtxM(_) => todo!(),
            MethodCall { .. } => todo!(),
            PredefFunc(_) => todo!(),
            BinOp(_, _, _) => todo!(),
        }
    }

    pub fn tpe(&self) -> &SType {
        match self {
            Constant { tpe, .. } => tpe,
            _ => todo!(),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
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
