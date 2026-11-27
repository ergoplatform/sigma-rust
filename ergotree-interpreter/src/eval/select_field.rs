use ergotree_ir::mir::select_field::SelectField;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SelectField {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        match input_v {
            // Matches JVM `sigma.ast.SelectField.eval`: only Tuple2 (`fieldIndex` 1 or 2) is supported.
            Value::Tup(items) if items.len() == 2 => items
                .get(self.field_index.zero_based_index())
                .cloned()
                .ok_or_else(|| {
                    EvalError::NotFound(format!(
                        "SelectField field index is out of bounds. Index: {0:?}, tuple: {1:?}",
                        self.field_index, items
                    ))
                }),
            Value::Tup(items) => Err(EvalError::UnexpectedValue(format!(
                "SelectField evaluation only supports 2-element tuples, got {}",
                items.len()
            ))),
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected SelectField input to be Value::Tup, got: {0:?}",
                input_v
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::try_eval_out_wo_ctx;
    use core::convert::TryInto;
    use ergotree_ir::mir::constant::{Constant, Literal};
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::types::stuple::STuple;
    use ergotree_ir::types::stype::SType;

    #[test]
    fn eval_accepts_pair_tuple_fields() {
        let pair = Constant {
            tpe: SType::STuple(STuple::pair(SType::SInt, SType::SInt)),
            v: Literal::Tup([Literal::Int(10), Literal::Int(20)].into()),
        };
        let sf1: Expr = SelectField::new(pair.clone().into(), 1u8.try_into().unwrap())
            .unwrap()
            .into();
        let sf2: Expr = SelectField::new(pair.into(), 2u8.try_into().unwrap())
            .unwrap()
            .into();
        assert_eq!(try_eval_out_wo_ctx::<i32>(&sf1).unwrap(), 10);
        assert_eq!(try_eval_out_wo_ctx::<i32>(&sf2).unwrap(), 20);
    }

    #[test]
    fn eval_rejects_non_pair_tuple_constant() {
        let triple = Constant {
            tpe: SType::STuple(STuple::triple(SType::SInt, SType::SInt, SType::SInt)),
            v: Literal::Tup(
                vec![Literal::Int(1), Literal::Int(2), Literal::Int(3)]
                    .try_into()
                    .unwrap(),
            ),
        };
        let sf1: Expr = SelectField::new(triple.clone().into(), 1u8.try_into().unwrap())
            .unwrap()
            .into();
        let sf3: Expr = SelectField::new(triple.into(), 3u8.try_into().unwrap())
            .unwrap()
            .into();
        assert!(try_eval_out_wo_ctx::<i32>(&sf1).is_err());
        assert!(try_eval_out_wo_ctx::<i32>(&sf3).is_err());
    }
}
