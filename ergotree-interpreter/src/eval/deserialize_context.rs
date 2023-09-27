use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::deserialize_context::DeserializeContext;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::types::stype::SType;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for DeserializeContext {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        match ctx.ctx.extension.values.get(&self.id) {
            Some(c) => {
                let expected_tpe = SType::SColl(SType::SByte.into());
                if c.tpe != expected_tpe {
                    Err(EvalError::UnexpectedExpr(format!(
                        "DeserializeContext: expected extension value {} with id {} to have type {:?} got {:?}",
                        c, self.id, expected_tpe, c.tpe
                    )))
                } else {
                    let bytes = c.v.clone().try_extract_into::<Vec<u8>>()?;
                    let expr = Expr::sigma_parse_bytes(bytes.as_slice())?;
                    if expr.tpe() != self.tpe {
                        return Err(EvalError::UnexpectedExpr(format!("DeserializeContext: expected deserialized expr from extension value {} with id {} to have type {:?}, got {:?}", c, self.id, self.tpe, expr.tpe())));
                    }
                    expr.eval(env, ctx)
                }
            }
            None => Err(EvalError::NotFound(format!(
                "DeserializeContext: no value with id {} in context extension map {}",
                self.id, ctx.ctx.extension
            ))),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use sigma_test_util::force_any_val;

    use crate::eval::context::Context;
    use crate::eval::tests::try_eval_out;
    use crate::sigma_protocol::prover::ContextExtension;

    use super::*;

    #[test]
    fn eval() {
        let expr: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        }
        .into();
        let inner_expr: Expr = true.into();
        let ctx_ext = ContextExtension {
            values: [(1u8, inner_expr.sigma_serialize_bytes().unwrap().into())]
                .iter()
                .cloned()
                .collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(ctx_ext);
        assert!(try_eval_out::<bool>(&expr, Rc::new(ctx)).unwrap());
    }

    #[test]
    fn eval_id_not_found() {
        let expr: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        }
        .into();
        let ctx = force_any_val::<Context>().with_extension(ContextExtension::empty());
        assert!(try_eval_out::<bool>(&expr, Rc::new(ctx)).is_err());
    }

    #[test]
    fn eval_context_extension_wrong_type() {
        let expr: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        }
        .into();
        // should be byte array
        let ctx_ext_val: Constant = 1i32.into();
        let ctx_ext = ContextExtension {
            values: [(1u8, ctx_ext_val)].iter().cloned().collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(ctx_ext);
        assert!(try_eval_out::<bool>(&expr, Rc::new(ctx)).is_err());
    }

    #[test]
    fn evaluated_expr_wrong_type() {
        let expr: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        }
        .into();
        // should be SBoolean
        let inner_expr: Expr = GlobalVars::Height.into();
        let ctx_ext = ContextExtension {
            values: [(1u8, inner_expr.sigma_serialize_bytes().unwrap().into())]
                .iter()
                .cloned()
                .collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(ctx_ext);
        assert!(try_eval_out::<Value>(&expr, Rc::new(ctx)).is_err());
    }
}
