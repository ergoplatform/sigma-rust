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
use super::expr::InvalidArgumentError;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ByIndex {
    input: Box<Expr>,
    index: Box<Expr>,
    default: Option<Box<Expr>>,
}

impl ByIndex {
    pub const OP_CODE: OpCode = OpCode::BY_INDEX;

    pub fn new(
        input: Expr,
        index: Expr,
        default: Option<Box<Expr>>,
    ) -> Result<Self, InvalidArgumentError> {
        let input_elem_type: SType = *match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(elem_type),
            _ => Err(InvalidArgumentError(format!(
                "Expected Map input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;
        if index.post_eval_tpe() != SType::SInt {
            return Err(InvalidArgumentError(format!(
                "ByIndex: expected index type to be SInt, got {0:?}",
                index
            )));
        }
        if !default
            .clone()
            .map(|expr| expr.post_eval_tpe() == input_elem_type)
            .unwrap_or(true)
        {
            return Err(InvalidArgumentError(format!(
                "ByIndex: expected default type to be {0:?}, got {1:?}",
                input_elem_type, default
            )));
        }
        Ok(Self {
            input: input.into(),
            index: index.into(),
            default,
        })
    }

    pub fn tpe(&self) -> SType {
        match self.input.post_eval_tpe() {
            SType::SColl(elem_tpe) => *elem_tpe,
            _ => panic!("collection is expected"),
        }
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for ByIndex {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.index.sigma_serialize(w)?;
        self.default.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let index = Expr::sigma_parse(r)?;
        let default = Option::<Box<Expr>>::sigma_parse(r)?;
        Ok(Self::new(input, index, default)?)
    }
}

impl Evaluable for ByIndex {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let index_v = self.index.eval(env, ctx)?;
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => Ok(coll.as_vec()),
            _ => Err(EvalError::UnexpectedValue(format!(
                "ByIndex: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        match self.default.clone() {
            Some(default) => {
                let default_v = default.eval(env, ctx)?;
                Ok(normalized_input_vals
                    .get(index_v.try_extract_into::<i32>()? as usize)
                    .cloned()
                    .unwrap_or(default_v))
            }
            None => normalized_input_vals
                .get(index_v.clone().try_extract_into::<i32>()? as usize)
                .cloned()
                .ok_or_else(|| {
                    EvalError::Misc(format!(
                        "ByIndex: index {0:?} out of bounds for collection size {1:?}",
                        index_v,
                        normalized_input_vals.len()
                    ))
                }),
        }
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;
    use proptest::result::Probability;

    impl Arbitrary for ByIndex {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ArbExprParams;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(args.tpe.clone().into()),
                    depth: args.depth,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SInt,
                    depth: 0,
                }),
                any_with::<Option<Box<Expr>>>((
                    Probability::default(),
                    ArbExprParams {
                        tpe: args.tpe,
                        depth: 0,
                    },
                )),
            )
                .prop_map(|(input, index, default)| Self {
                    input: input.into(),
                    index: index.into(),
                    default,
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {

    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::eval::tests::eval_out_wo_ctx;
    use crate::ir_ergo_box::IrBoxId;
    use crate::mir::expr::Expr;
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn ser_roundtrip(v in any::<ByIndex>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }

    #[test]
    fn eval() {
        let expr: Expr = ByIndex::new(GlobalVars::Outputs.into(), Expr::Const(0i32.into()), None)
            .unwrap()
            .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<IrBoxId>(&expr, ctx.clone()),
            ctx.outputs.get(0).unwrap().clone()
        );
    }

    #[test]
    fn eval_with_default() {
        let expr: Expr = ByIndex::new(
            Expr::Const(vec![1i64, 2i64].into()),
            Expr::Const(3i32.into()),
            Some(Box::new(Expr::Const(5i64.into()))),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<i64>(&expr), 5);
    }
}
