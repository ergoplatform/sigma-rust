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

use super::constant::TryExtractInto;
use super::expr::Expr;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Or {
    input: Box<Expr>,
}

impl Or {
    pub const OP_CODE: OpCode = OpCode::OR;

    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl Evaluable for Or {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bools = input_v.try_extract_into::<Vec<bool>>()?;
        Ok(input_v_bools.iter().any(|b| *b).into())
    }
}

impl SigmaSerializable for Or {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self {
            input: Expr::sigma_parse(r)?.into(),
        })
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for Or {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SColl(SType::SBoolean.into()),
                depth: args,
            })
            .prop_map(|input| Self {
                input: input.into(),
            })
            .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;

    use super::*;

    use proptest::collection;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any_with::<Or>(2)) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        #[test]
        fn eval(bools in collection::vec(any::<bool>(), 0..10)) {
            let expr: Expr = Or {input: Expr::Const(bools.clone().into()).into()}.into();
            let ctx = Rc::new(force_any_val::<Context>());
            let res = eval_out::<bool>(&expr, ctx);
            prop_assert_eq!(res, bools.iter().any(|b| *b));
        }
    }
}
