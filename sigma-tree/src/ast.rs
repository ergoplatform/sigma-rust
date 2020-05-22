use crate::{data::SigmaProp, serialization::op_code::OpCode, types::*};
use core::fmt;
use Expr::*;

pub mod ops;

#[derive(PartialEq, Eq, Debug)]
pub struct RegisterId(u8);

#[derive(PartialEq, Eq, Debug)]
pub enum CollPrim {
    CollBoolean(Vec<bool>),
    CollByte(Vec<i8>),
    CollShort(Vec<i16>),
    CollInt(Vec<i32>),
    CollLong(Vec<i64>),
}

// TODO: extract and wrap ErgoBoxCandidate
#[derive(PartialEq, Eq, Debug)]
pub struct ErgoBox {}

#[derive(PartialEq, Eq, Debug)]
pub enum ConstantVal {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    BigInt,
    GroupElement,
    SigmaProp(Box<SigmaProp>),
    CBox(Box<ErgoBox>),
    AvlTree,
    CollPrim(CollPrim),
    Coll(Vec<ConstantVal>),
    Tup(Vec<ConstantVal>),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Constant {
    pub tpe: SType,
    pub v: ConstantVal,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    Const(Constant),
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
            Const { .. } => todo!(),
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
            Const(c) => &c.tpe,
            _ => todo!(),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum CollMethods {
    Fold {
        input: Box<Expr>,
        zero: Box<Expr>,
        fold_op: Box<Expr>,
    },
}

#[derive(PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Eq, Debug)]
pub enum ContextMethods {
    Inputs,
    Outputs,
}

#[derive(PartialEq, Eq, Debug)]
pub enum PredefFunc {
    Sha256 { input: Box<Expr> },
}
