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
use super::value::CollKind;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Filter {
    /// Collection
    input: Box<Expr>,
    /// Function (lambda) to test each element
    condition: Box<Expr>,
    elem_tpe: SType,
}

impl Filter {
    pub const OP_CODE: OpCode = OpCode::FILTER;

    pub fn new(input: Expr, condition: Expr) -> Result<Self, InvalidArgumentError> {
        let input_elem_type: SType = *match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(elem_type),
            _ => Err(InvalidArgumentError(format!(
                "Expected Map input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;
        match condition.tpe() {
            SType::SFunc(sfunc)
                if sfunc.t_dom == vec![input_elem_type.clone()]
                    && *sfunc.t_range == SType::SBoolean =>
            {
                Ok(Filter {
                    input: input.into(),
                    condition: condition.into(),
                    elem_tpe: input_elem_type,
                })
            }
            _ => Err(InvalidArgumentError(format!(
                "Invalid condition tpe: {0:?}",
                condition.tpe()
            ))),
        }
    }

    pub fn tpe(&self) -> SType {
        SType::SColl(self.elem_tpe.clone().into())
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl SigmaSerializable for Filter {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.condition.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let condition = Expr::sigma_parse(r)?;
        Ok(Filter::new(input, condition)?)
    }
}

impl Evaluable for Filter {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let condition_v = self.condition.eval(env, ctx)?;
        let input_v_clone = input_v.clone();
        let mut condition_call = |arg: Value| match &condition_v {
            Value::FuncValue(func_value) => {
                let func_arg = func_value.args().first().ok_or_else(|| {
                    EvalError::NotFound(
                        "Filter: evaluated condition has empty arguments list".to_string(),
                    )
                })?;
                let env1 = env.clone().extend(func_arg.idx, arg);
                func_value.body().eval(&env1, ctx)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected Filter::condition to be Value::FuncValue got: {0:?}",
                input_v_clone
            ))),
        };
        let normalized_input_vals: Vec<Value> = match input_v {
            Value::Coll(coll) => {
                if *coll.elem_tpe() != self.elem_tpe {
                    return Err(EvalError::UnexpectedValue(format!(
                        "expected Filter input element type to be {0:?}, got: {1:?}",
                        self.elem_tpe,
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

        let items_conditions: Vec<bool> = normalized_input_vals
            .clone()
            .into_iter()
            .map(|item| {
                condition_call(item).and_then(|res| {
                    res.try_extract_into::<bool>()
                        .map_err(EvalError::TryExtractFrom)
                })
            })
            .collect::<Result<Vec<bool>, EvalError>>()?;
        let filtered_items = normalized_input_vals
            .into_iter()
            .zip(items_conditions)
            .filter(|(_, condition)| *condition)
            .map(|(item, _)| item)
            .collect();
        Ok(Value::Coll(CollKind::from_vec(
            self.elem_tpe.clone(),
            filtered_items,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::bin_op::BinOp;
    use crate::ast::bin_op::RelationOp;
    use crate::ast::expr::tests::ArbExprParams;
    use crate::ast::expr::Expr;
    use crate::ast::extract_amount::ExtractAmount;
    use crate::ast::func_value::FuncArg;
    use crate::ast::func_value::FuncValue;
    use crate::ast::property_call::PropertyCall;
    use crate::ast::val_use::ValUse;
    use crate::chain::ergo_box::ErgoBox;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::scontext;
    use crate::types::sfunc::SFunc;

    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for Filter {
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
                .prop_map(|(input, mapper)| Filter::new(input, mapper).unwrap())
                .boxed()
        }
    }

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
            let body: Expr = BinOp {
                kind: RelationOp::LE.into(),
                left: Box::new(Expr::Const(1i64.into())),
                right: Box::new(Expr::ExtractAmount(
                        ExtractAmount::new(val_use)
                    .unwrap(),
                )),
            }
            .into();
            let expr: Expr = Filter::new(
                data_inputs,
                FuncValue::new(
                    vec![FuncArg {
                        idx: 1.into(),
                        tpe: SType::SBox,
                    }],
                    body,
                )
                .into(),
            )
            .unwrap()
            .into();
            let ctx = Rc::new(ctx);
            assert_eq!(
                eval_out::<Vec<ErgoBox>>(&expr, ctx.clone()),
                ctx.data_inputs.clone()
                    .into_iter()
                    .filter(| b| 1 <= b.value.as_i64()).collect::<Vec<ErgoBox>>()
            );
        }

        #[test]
        fn ser_roundtrip(v in any::<Filter>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
