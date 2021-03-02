use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::sfunc::SFunc;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::CollKind;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Map {
    /// Collection
    input: Box<Expr>,
    /// Function (lambda) to apply to each element
    mapper: Box<Expr>,
    mapper_sfunc: SFunc,
}

impl Map {
    pub const OP_CODE: OpCode = OpCode::MAP;

    pub fn new(input: Expr, mapper: Expr) -> Result<Self, InvalidArgumentError> {
        let input_elem_type: SType = *match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(elem_type),
            _ => Err(InvalidArgumentError(format!(
                "Expected Map input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;
        match mapper.tpe() {
            SType::SFunc(sfunc) if sfunc.t_dom == vec![input_elem_type] => Ok(Map {
                input: input.into(),
                mapper: mapper.into(),
                mapper_sfunc: sfunc,
            }),
            _ => Err(InvalidArgumentError(format!(
                "Invalid mapper tpe: {0:?}",
                mapper.tpe()
            ))),
        }
    }

    pub fn tpe(&self) -> SType {
        SType::SColl(self.mapper_sfunc.t_range.clone())
    }

    pub fn out_elem_tpe(&self) -> SType {
        *self.mapper_sfunc.t_range.clone()
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for Map {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.mapper.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let mapper = Expr::sigma_parse(r)?;
        Ok(Map::new(input, mapper)?)
    }
}

impl Evaluable for Map {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let mapper_v = self.mapper.eval(env, ctx)?;
        let input_v_clone = input_v.clone();
        let mut mapper_call = |arg: Value| match &mapper_v {
            Value::FuncValue(func_value) => {
                let func_arg = func_value.args().first().ok_or_else(|| {
                    EvalError::NotFound(
                        "Map: evaluated mapper has empty arguments list".to_string(),
                    )
                })?;
                let env1 = env.clone().extend(func_arg.idx, arg);
                func_value.body().eval(&env1, ctx)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected mapper to be Value::FuncValue got: {0:?}",
                input_v_clone
            ))),
        };
        let mapper_input_tpe = self
            .mapper_sfunc
            .t_dom
            .first()
            .ok_or_else(|| {
                EvalError::NotFound(
                    "Map: mapper SFunc.t_dom is empty (does not have arguments)".to_string(),
                )
            })?
            .clone();
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => {
                if *coll.elem_tpe() != mapper_input_tpe {
                    return Err(EvalError::UnexpectedValue(format!(
                        "expected Map input element type to be {0:?}, got: {1:?}",
                        mapper_input_tpe,
                        coll.elem_tpe()
                    )));
                };
                Ok(coll.as_vec())
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected Map input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        normalized_input_vals
            .iter()
            .map(|item| mapper_call(item.clone()))
            .collect::<Result<Vec<Value>, EvalError>>()
            .map(|values| {
                CollKind::from_vec(self.out_elem_tpe(), values).map_err(EvalError::TryExtractFrom)
            })
            .and_then(|v| v) // flatten <Result<Result<Value, _>, _>
            .map(Value::Coll)
    }
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use proptest::prelude::*;

    impl Arbitrary for Map {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SColl(SType::SBoolean.into()),
                    depth: 1,
                }),
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SFunc(SFunc {
                        t_dom: vec![SType::SBoolean],
                        t_range: SType::SBoolean.into(),
                        tpe_params: vec![],
                    }),
                    depth: 0,
                }),
            )
                .prop_map(|(input, mapper)| Map::new(input, mapper).unwrap())
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
    use crate::mir::bin_op::ArithOp;
    use crate::mir::bin_op::BinOp;
    use crate::mir::expr::Expr;
    use crate::mir::extract_amount::ExtractAmount;
    use crate::mir::func_value::FuncArg;
    use crate::mir::func_value::FuncValue;
    use crate::mir::property_call::PropertyCall;
    use crate::mir::val_use::ValUse;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::scontext;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn eval_box_value(ctx in any::<Context>()) {
            let data_inputs: Expr = PropertyCall {
                obj: Box::new(Expr::Context),
                method: scontext::DATA_INPUTS_PROPERTY.clone(),
            }
            .into();
            let val_use: Expr = ValUse {
                val_id: 1.into(),
                tpe: SType::SBox,
            }
            .into();
            let mapper_body: Expr = BinOp {
                kind: ArithOp::Plus.into(),
                left: Box::new(Expr::Const(1i64.into())),
                right: Box::new(Expr::ExtractAmount(
                        ExtractAmount::new(val_use)
                    .unwrap(),
                )),
            }
            .into();
            let expr: Expr = Map::new(
                data_inputs,
                FuncValue::new(
                    vec![FuncArg {
                        idx: 1.into(),
                        tpe: SType::SBox,
                    }],
                    mapper_body,
                )
                .into(),
            )
            .unwrap()
            .into();
            let ctx = Rc::new(ctx);
            assert_eq!(
                eval_out::<Vec<i64>>(&expr, ctx.clone()),
                ctx.data_inputs
                    .iter()
                    .map(| b| b.get_box(&ctx.box_arena).unwrap().value() + 1).collect::<Vec<i64>>()
            );
        }

        #[test]
        fn ser_roundtrip(v in any::<Map>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
