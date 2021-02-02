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

/// If, non-lazy - evaluate both branches
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct If {
    condition: Box<Expr>,
    true_branch: Box<Expr>,
    false_branch: Box<Expr>,
}

impl If {
    pub const OP_CODE: OpCode = OpCode::IF;

    pub fn tpe(&self) -> SType {
        self.true_branch.tpe()
    }

    pub fn op_code(&self) -> OpCode {
        Self::OP_CODE
    }
}

impl Evaluable for If {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        todo!()
    }
}

impl SigmaSerializable for If {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.condition.sigma_serialize(w)?;
        self.true_branch.sigma_serialize(w)?;
        self.false_branch.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let condition = Expr::sigma_parse(r)?.into();
        let true_branch = Expr::sigma_parse(r)?.into();
        let false_branch = Expr::sigma_parse(r)?.into();
        Ok(Self {
            condition,
            true_branch,
            false_branch,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::ast::expr::tests::ArbExprParams;
    use crate::ast::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    impl Arbitrary for If {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Expr>(ArbExprParams {
                    tpe: SType::SBoolean,
                    depth: 2,
                }),
                any::<Expr>(),
            )
                .prop_map(|(condition, true_branch)| Self {
                    condition: condition.into(),
                    true_branch: true_branch.clone().into(),
                    false_branch: true_branch.into(),
                })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<If>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        // #[test]
        // fn eval(bools in collection::vec(any::<bool>(), 0..10)) {
        //     let expr: Expr = If {input: Expr::Const(bools.clone().into()).into()}.into();
        //     let ctx = Rc::new(force_any_val::<Context>());
        //     let res = eval_out::<bool>(&expr, ctx);
        //     prop_assert_eq!(res, bools.iter().all(|b| *b));
        // }
    }
}
