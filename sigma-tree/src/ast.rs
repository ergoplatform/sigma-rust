#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::{
    data::{self, ConstantKind, RegisterId},
    types::*,
};
use core::fmt;
use data::DataSerializer;
use io::{Read, Write};
use serializer::SerializationError;
use sigma_ser::{
    serializer::{self, SigmaSerializable},
    vlq_encode,
};
use std::{collections::HashMap, io, marker::PhantomData, rc::Rc};
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use Expr::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct OpCode(u8);

// pub struct Expr {
//     pub tpe: SType,
//     pub kind: ExprKind,
// }

pub enum Expr {
    Constant {
        tpe: SType,
        v: ConstantKind,
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

// TODO: extract
pub struct ConstantSerializer {}

impl ConstantSerializer {
    fn sigma_serialize<W: WriteSigmaVlqExt>(expr: &Expr, mut w: W) -> Result<(), io::Error> {
        match expr {
            Constant { tpe, v } => {
                tpe.sigma_serialize(&mut w)?;
                DataSerializer::sigma_serialize(v, tpe, w)
            }
            _ => panic!("constant expected"),
        }
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Expr, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
        let tpe = SType::sigma_parse(&mut r)?;
        let v = DataSerializer::sigma_parse(&tpe, &mut r)?;
        Ok(Constant { tpe, v })
    }
}

// TODO: extract to op_codes module and set correct value
const LAST_CONSTANT_CODE: u8 = 0;

// TODO: extract
impl SigmaSerializable for Expr {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, mut w: W) -> Result<(), io::Error> {
        match self {
            c @ Constant { .. } => ConstantSerializer::sigma_serialize(self, w),
            expr => {
                let op_code = self.op_code();
                w.put_u8(op_code.0)?;
                ExprSerializers::sigma_serialize(self, w)
            }
        }
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let first_byte = r.peek_u8()?;
        if first_byte <= LAST_CONSTANT_CODE {
            ConstantSerializer::sigma_parse(&mut r)
        } else {
            let op_code = r.get_u8()?;
            ExprSerializers::sigma_parse(&OpCode(op_code), r)
        }
    }
}

pub struct FoldSerializer {}

// TODO: extract
impl FoldSerializer {
    // TODO: proper op code
    const OP_CODE: OpCode = OpCode(0);

    fn sigma_serialize<W: WriteSigmaVlqExt>(expr: &Expr, mut w: W) -> Result<(), io::Error> {
        match expr {
            CollM(CollMethods::Fold {
                input,
                zero,
                fold_op,
            }) => {
                input.sigma_serialize(&mut w)?;
                zero.sigma_serialize(&mut w)?;
                fold_op.sigma_serialize(&mut w)?;
                Ok(())
            }
            e => panic!("expected Fold"),
        }
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Expr, SerializationError> {
        let input = Expr::sigma_parse(&mut r)?;
        let zero = Expr::sigma_parse(&mut r)?;
        let fold_op = Expr::sigma_parse(&mut r)?;
        Ok(CollM(CollMethods::Fold {
            input: Box::new(input),
            zero: Box::new(zero),
            fold_op: Box::new(fold_op),
        }))
    }
}

// TODO: extract
pub struct ExprSerializers {}

impl ExprSerializers {
    pub fn sigma_serialize<W: WriteSigmaVlqExt>(expr: &Expr, w: W) -> Result<(), io::Error> {
        match expr {
            CollM(cm) => match cm {
                fold @ CollMethods::Fold { .. } => FoldSerializer::sigma_serialize(expr, w),
            },
            _ => panic!(format!("don't know how to serialize {}", expr)),
        }
    }

    pub fn sigma_parse<R: ReadSigmaVlqExt>(
        op_code: &OpCode,
        r: R,
    ) -> Result<Expr, SerializationError> {
        match op_code {
            &FoldSerializer::OP_CODE => FoldSerializer::sigma_parse(r),
            o => Err(SerializationError::InvalidOpCode),
        }
    }
}
