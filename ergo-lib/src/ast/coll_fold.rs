use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stuple::STuple;
use crate::types::stuple::TupleItems;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::Coll::NonPrimitive;
use super::value::Coll::Primitive;
use super::value::CollPrim;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Fold {
    /// Collection
    input: Box<Expr>,
    /// Initial value for accumulator
    zero: Box<Expr>,
    /// Function (lambda)
    fold_op: Box<Expr>,
}

impl Fold {
    pub fn new(input: Expr, zero: Expr, fold_op: Expr) -> Result<Self, InvalidArgumentError> {
        let input_elem_type: SType = *match input.post_eval_tpe() {
            SType::SColl(elem_type) => Ok(elem_type),
            _ => Err(InvalidArgumentError(format!(
                "Expected Fold input to be SColl, got {0:?}",
                input.tpe()
            ))),
        }?;
        match fold_op.tpe() {
            SType::SFunc(sfunc)
                if sfunc.t_dom == vec![STuple::pair(zero.tpe(), input_elem_type).into()] =>
            {
                Ok(Fold {
                    input: input.into(),
                    zero: zero.into(),
                    fold_op: fold_op.into(),
                })
            }
            _ => Err(InvalidArgumentError(format!(
                "Invalid fold_op tpe: {0:?}",
                fold_op.tpe()
            ))),
        }
    }

    pub fn tpe(&self) -> SType {
        self.zero.tpe()
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::FOLD
    }
}

impl SigmaSerializable for Fold {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.zero.sigma_serialize(w)?;
        self.fold_op.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?.into();
        let zero = Expr::sigma_parse(r)?.into();
        let fold_op = Expr::sigma_parse(r)?.into();
        Ok(Fold {
            input,
            zero,
            fold_op,
        })
    }
}

impl Evaluable for Fold {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let zero_v = self.zero.eval(env, ctx)?;
        let fold_op_v = self.fold_op.eval(env, ctx)?;
        let input_v_clone = input_v.clone();
        let mut fold_op_call = |arg: Value| match &fold_op_v {
            Value::FuncValue(func_value) => {
                let func_arg = func_value
                    .args()
                    .first()
                    .ok_or_else(|| EvalError::NotFound("empty argument for fold op".to_string()))?;
                let env1 = env.clone().extend(func_arg.idx, arg);
                func_value.body().eval(&env1, ctx)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected fold_op to be Value::FuncValue got: {0:?}",
                input_v_clone
            ))),
        };
        match input_v {
            Value::Coll(coll) => match coll {
                Primitive(CollPrim::CollByte(coll_byte)) => {
                    coll_byte.iter().try_fold(zero_v, |acc, byte| {
                        let tup_arg = Value::Tup(TupleItems::pair(acc, Value::Byte(*byte)));
                        fold_op_call(tup_arg)
                    })
                }
                NonPrimitive { elem_tpe: _, v } => v.iter().try_fold(zero_v, |acc, item| {
                    let tup_arg = Value::Tup(TupleItems::pair(acc, item.clone()));
                    fold_op_call(tup_arg)
                }),
            },
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected Fold input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::rc::Rc;

    use crate::ast::bin_op::ArithOp;
    use crate::ast::bin_op::BinOp;
    use crate::ast::expr::Expr;
    use crate::ast::extract_amount::ExtractAmount;
    use crate::ast::func_value::FuncArg;
    use crate::ast::func_value::FuncValue;
    use crate::ast::property_call::PropertyCall;
    use crate::ast::select_field::SelectField;
    use crate::ast::val_use::ValUse;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::scontext;
    use crate::types::stuple::STuple;

    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for Fold {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), any::<Expr>(), any::<Expr>())
                .prop_map(|(input, zero, fold_op)| Self {
                    input: input.into(),
                    zero: zero.into(),
                    fold_op: fold_op.into(),
                })
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
            let tuple: Expr = ValUse {
                val_id: 1.into(),
                tpe: SType::STuple(STuple {
                    items: TupleItems::pair(SType::SLong, SType::SBox),
                }),
            }
            .into();
            let fold_op_body: Expr = BinOp {
                kind: ArithOp::Plus.into(),
                left: Box::new(Expr::SelectField(
                    SelectField::new(tuple.clone(), 1.try_into().unwrap()).unwrap(),
                )),
                right: Box::new(Expr::ExtractAmount(
                    ExtractAmount::new(Expr::SelectField(
                        SelectField::new(tuple, 2.try_into().unwrap()).unwrap(),
                    ))
                    .unwrap(),
                )),
            }
            .into();
            let expr: Expr = Fold::new(
                data_inputs,
                Expr::Const(0i64.into()),
                FuncValue::new(
                    vec![FuncArg {
                        idx: 1.into(),
                        tpe: SType::STuple(STuple {
                            items: TupleItems::pair(SType::SLong, SType::SBox),
                        }),
                    }],
                    fold_op_body,
                )
                .into(),
            )
            .unwrap()
            .into();
            let ctx = Rc::new(ctx);
            assert_eq!(
                eval_out::<i64>(&expr, ctx.clone()),
                ctx.data_inputs
                    .iter()
                    .fold(0i64, |acc, b| acc + b.value.as_i64())
            );
        }

        #[test]
        fn ser_roundtrip(v in any::<Fold>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
