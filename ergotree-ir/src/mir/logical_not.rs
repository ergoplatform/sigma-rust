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
pub struct LogicalNot {
    input: Box<Expr>,
}

impl LogicalNot {
    pub const OP_CODE: OpCode = OpCode::LOGICAL_NOT;

    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for LogicalNot {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self {
            input: Expr::sigma_parse(r)?.into(),
        })
    }
}

impl Evaluable for LogicalNot {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bool = input_v.try_extract_into::<bool>()?;
        Ok((!input_v_bool).into())
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use crate::mir::expr::arbitrary::ArbExprParams;

    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for LogicalNot {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = usize;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any_with::<Expr>(ArbExprParams {
                tpe: SType::SBoolean,
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

    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<LogicalNot>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        #[test]
        fn eval(b in any::<bool>()) {
            let expr: Expr = LogicalNot {input: Expr::Const(b.into()).into()}.into();
            let res = eval_out_wo_ctx::<bool>(&expr);
            prop_assert_eq!(res, !b);
        }
    }
}
