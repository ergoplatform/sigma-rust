//! Operators in ErgoTree

use std::io::Error;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Upcast {
    input: Box<Expr>,
    tpe: SType,
}

impl Upcast {
    pub const OP_CODE: OpCode = OpCode::UPCAST;

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }

    pub fn tpe(&self) -> SType {
        self.tpe.clone()
    }
}

fn upcast_to_long(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i64).into()),
        Value::Short(v) => Ok((v as i64).into()),
        Value::Int(v) => Ok((v as i64).into()),
        Value::Long(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Long",
            in_v
        ))),
    }
}

fn upcast_to_int(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i32).into()),
        Value::Short(v) => Ok((v as i32).into()),
        Value::Int(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Int",
            in_v
        ))),
    }
}

fn upcast_to_short(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(v) => Ok((v as i16).into()),
        Value::Short(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Short",
            in_v
        ))),
    }
}

fn upcast_to_byte(in_v: Value) -> Result<Value, EvalError> {
    match in_v {
        Value::Byte(_) => Ok(in_v),
        _ => Err(EvalError::UnexpectedValue(format!(
            "Upcast: cannot upcast {0:?} to Byte",
            in_v
        ))),
    }
}

impl Evaluable for Upcast {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match self.tpe {
            SType::SBigInt => todo!("upcasting to BigInt is not yet supported"),
            SType::SLong => upcast_to_long(input_v),
            SType::SInt => upcast_to_int(input_v),
            SType::SShort => upcast_to_short(input_v),
            SType::SByte => upcast_to_byte(input_v),
            _ => Err(EvalError::UnexpectedValue(format!(
                "Upcast: expected numeric value, got {0:?}",
                input_v
            ))),
        }
    }
}

impl SigmaSerializable for Upcast {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        self.input.sigma_serialize(w)?;
        self.tpe.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?.into();
        let tpe = SType::sigma_parse(r)?;
        Ok(Upcast { input, tpe })
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for Upcast {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SInt,
                depth: 2,
            })
            .prop_map(|input| Upcast {
                input: Box::new(input),
                tpe: SType::SLong,
            })
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
pub mod proptests {

    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Upcast>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
