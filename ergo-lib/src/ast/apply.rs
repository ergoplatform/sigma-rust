use std::io;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::val_def::ValId;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Apply {
    pub func: Expr,
    pub args: Vec<Expr>,
}

impl Apply {
    pub fn tpe(&self) -> SType {
        match self.func.tpe() {
            SType::SColl(_) => todo!(),
            SType::SFunc(f) => f.t_range,
            // TODO: add checked constructor (like in OptionGet)
            _ => panic!("unexpected Apply::func: {0:?}", self.func.tpe()),
        }
    }
}

impl Evaluable for Apply {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let func_v = self.func.eval(env, ctx)?;
        let args_v_res: Result<Vec<Value>, EvalError> =
            self.args.iter().map(|arg| arg.eval(env, ctx)).collect();
        let args_v = args_v_res?;
        match func_v {
            Value::FuncValue(fv) => {
                let arg_ids: Vec<ValId> = fv.args.iter().map(|a| a.idx).collect();
                let mut cur_env = env.clone();
                arg_ids.iter().zip(args_v).for_each(|(idx, arg_v)| {
                    cur_env.insert(*idx, arg_v);
                });
                fv.body.eval(&cur_env, ctx)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected func_v to be Value::FuncValue got: {0:?}",
                func_v
            ))),
        }
    }
}

impl SigmaSerializable for Apply {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.func.sigma_serialize(w)?;
        self.args.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let func = Expr::sigma_parse(r)?;
        let args = Vec::<Expr>::sigma_parse(r)?;
        Ok(Apply { func, args })
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ast::bin_op::BinOp;
    use crate::ast::bin_op::LogicOp;
    use crate::ast::block::BlockValue;
    use crate::ast::func_value::*;
    use crate::ast::val_def::ValDef;
    use crate::ast::val_use::ValUse;
    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::test_util::force_any_val;
    use crate::types::stype::SType;

    use super::*;

    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Apply {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), vec(any::<Expr>(), 1..10))
                .prop_map(|(func, args)| Self { func, args })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Apply>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }

    #[test]
    fn user_defined_func() {
        let arg = Expr::Const(Box::new(1i32.into()));
        let bin_op = Expr::BinOp(Box::new(BinOp {
            kind: LogicOp::Eq.into(),
            left: Expr::ValUse(Box::new(ValUse {
                val_id: 1.into(),
                tpe: SType::SInt,
            })),
            right: Expr::ValUse(Box::new(ValUse {
                val_id: 2.into(),
                tpe: SType::SInt,
            })),
        }));
        let body = Expr::BlockValue(Box::new(BlockValue {
            items: vec![ValDef {
                id: 2.into(),
                rhs: Expr::Const(Box::new(1i32.into())),
            }],
            result: bin_op,
        }));
        let apply: Expr = Box::new(Apply {
            func: Expr::FuncValue(Box::new(FuncValue {
                args: vec![FuncArg {
                    idx: 1.into(),
                    tpe: SType::SInt,
                }],
                body,
            })),
            args: vec![arg],
        })
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert!(eval_out::<bool>(&apply, ctx));
    }
}
