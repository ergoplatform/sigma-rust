#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::{
    data::{self, ConstantKind, RegisterId},
    types::*,
};
use serializer::SerializationError;
use sigma_ser::{
    serializer::{self, SigmaSerializable},
    vlq_encode,
};
use std::{io, marker::PhantomData};
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use Expr::*;

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

fn sigma_parse_constant<R: ReadSigmaVlqExt>(mut r: R) -> Result<Expr, SerializationError> {
    // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/DataSerializer.scala#L84-L84
    let tpe = SType::sigma_parse(&mut r)?;
    let v = data::sigma_parse_data(&tpe, &mut r)?;
    Ok(Constant { tpe, v })
}

// TODO: extract to op_codes module and set correct value
const LAST_CONSTANT_CODE: u8 = 0;

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error> {
        todo!()
    }
    fn sigma_parse<R: ReadSigmaVlqExt>(mut r: R) -> Result<Self, SerializationError> {
        let first_byte = r.peek_u8()?;
        if first_byte <= LAST_CONSTANT_CODE {
            sigma_parse_constant(&mut r)
        } else {
            let op_code = r.get_u8()?;
            // TODO: get a serializer for this op_code and run it
            todo!()
        }
    }
}
