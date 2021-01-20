use crate::chain::blake2b256_hash;
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
use crate::util::AsVecU8;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::Coll;
use super::value::CollPrim;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CalcBlake2b256 {
    input: Box<Expr>,
}

impl CalcBlake2b256 {
    pub fn new(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(SType::SColl(Box::new(SType::SByte)))?;
        Ok(CalcBlake2b256 {
            input: Box::new(input),
        })
    }

    pub fn tpe(&self) -> SType {
        SType::SColl(Box::new(SType::SByte))
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::CALC_BLAKE2B256
    }
}

impl Evaluable for CalcBlake2b256 {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v.clone() {
            Value::Coll(coll) => match *coll {
                Coll::Primitive(CollPrim::CollByte(coll_byte)) => {
                    let expected_hash: Vec<u8> =
                        blake2b256_hash(coll_byte.as_vec_u8().as_slice()).0.to_vec();
                    Ok(expected_hash.into())
                }
                _ => Err(EvalError::UnexpectedValue(format!(
                    "expected CalcBlake2b256 input to be byte array, got: {0:?}",
                    input_v
                ))),
            },
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected CalcBlake2b256 input to be byte array, got: {0:?}",
                input_v
            ))),
        }
    }
}

impl SigmaSerializable for CalcBlake2b256 {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        Ok(CalcBlake2b256::new(input)?)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::expr::tests::ArbExprParams;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;
    use crate::util::AsVecU8;

    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for CalcBlake2b256 {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(Box::new(SType::SByte)),
                depth: 0,
            })
            .prop_map(|input| Self {
                input: input.into(),
            })
            .boxed()
        }
    }

    proptest! {

        #[test]
        fn eval(byte_array in any::<Vec<u8>>()) {
            let expected_hash = blake2b256_hash(byte_array.as_slice()).0.to_vec();
            let expr: Expr = CalcBlake2b256 {
                input: Box::new(Expr::Const(byte_array.into())),
            }
            .into();
            let ctx = Rc::new(force_any_val::<Context>());
            assert_eq!(eval_out::<Vec<i8>>(&expr, ctx).as_vec_u8(), expected_hash);
        }

        #[test]
        fn ser_roundtrip(v in any::<CalcBlake2b256>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
