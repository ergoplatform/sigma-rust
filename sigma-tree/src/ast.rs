#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::types::*;
use serializer::SerializationError;
use sigma_ser::{
    serializer::{self, SigmaSerializable},
    vlq_encode,
};
use std::{io, marker::PhantomData};

pub struct RegisterId(u8);
pub struct OpCode(u8);
pub struct DataEnv();
pub struct ErgoTreeEvaluator {}

pub struct EvalResult();

pub struct Expr {
    pub tpe: SType,
    pub op_code: OpCode,
    pub kind: ExprKind,
}

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
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        todo!()
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError> {
        todo!()
    }
}

pub enum ExprKind {
    Constant(ConstantKind),
    Coll(Vec<Expr>),
    Tup(Vec<Expr>),
}

pub trait Value {
    fn tpe(&self) -> &SType;
    fn op_code(&self) -> OpCode;
    fn eval(&self, e: ErgoTreeEvaluator, env: DataEnv) -> EvalResult;
}

pub struct ConstantNode<T> {
    pub value: T,
    pub tpe: SType,
}

impl<T> Value for ConstantNode<T> {
    fn op_code(&self) -> OpCode {
        todo!()
    }
    fn eval(&self, e: ErgoTreeEvaluator, env: DataEnv) -> EvalResult {
        todo!()
    }
    fn tpe(&self) -> &SType {
        &self.tpe
    }
}

pub struct Outputs {
    tpe: SType,
}

impl Value for Outputs {
    fn op_code(&self) -> OpCode {
        OpCode(0)
    }
    fn eval(&self, e: ErgoTreeEvaluator, env: DataEnv) -> EvalResult {
        todo!()
    }
    fn tpe(&self) -> &SType {
        &self.tpe
    }
}

pub struct ExtractRegisterAs {
    pub input: Box<dyn Value>,
    pub register_id: RegisterId,
    pub tpe: SType,
}

impl Value for ExtractRegisterAs {
    fn tpe(&self) -> &SType {
        &self.tpe
    }
    fn op_code(&self) -> OpCode {
        todo!()
    }
    fn eval(&self, e: ErgoTreeEvaluator, env: DataEnv) -> EvalResult {
        todo!()
    }
}

fn deserialize_value() -> Box<dyn Value> {
    todo!()
}

pub fn deserialize_extract_reg_as_value() -> Box<dyn Value> {
    let tpe = deserialize_type();
    let input = deserialize_value();
    Box::new(ExtractRegisterAs {
        input: input,
        register_id: RegisterId(0),
        tpe: SType::SOption(Box::new(tpe)),
    })
}

pub struct Fold {
    pub input: Box<dyn Value>,
    pub zero: Box<dyn Value>,
    pub fold_op: Box<dyn Value>,
}

fn deserialize_fold_as_value() -> Box<dyn Value> {
    todo!()
    // deserialize_fold()
}

fn deserialize_data<T>(tpe: SType) -> Box<ConstantNode<T>> {
    todo!()
}

fn deserialize_constant() -> Box<dyn Value> {
    todo!()
}

pub struct MethodCall {
    obj: Box<dyn Value>,
    method: SMethod,
    args: Vec<Box<dyn Value>>,
}

impl Value for MethodCall {
    fn tpe(&self) -> &SType {
        &self.method.tpe
    }
    fn op_code(&self) -> OpCode {
        todo!()
    }
    fn eval(&self, e: ErgoTreeEvaluator, env: DataEnv) -> EvalResult {
        todo!()
    }
}
