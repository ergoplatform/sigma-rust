use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;
use crate::types::stype::TupleItems;

use super::expr::Expr;
use super::value::Coll::NonPrimitive;
use super::value::Coll::Primitive;
use super::value::CollPrim;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Fold {
    /// Collection
    input: Expr,
    /// Initial value for accumulator
    zero: Expr,
    /// Function (lambda)
    fold_op: Expr,
}

impl Fold {
    pub fn tpe(&self) -> SType {
        self.zero.tpe()
    }
}

impl SigmaSerializable for Fold {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.input.sigma_serialize(w)?;
        self.zero.sigma_serialize(w)?;
        self.fold_op.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let input = Expr::sigma_parse(r)?;
        let zero = Expr::sigma_parse(r)?;
        let fold_op = Expr::sigma_parse(r)?;
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
            Value::Coll(coll) => match *coll {
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
