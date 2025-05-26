#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
#[cfg(test)]
mod tests {

    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisterId;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::deserialize_register::DeserializeRegister;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::soft_fork::SoftForkError;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    use crate::eval::test_util::try_eval_out;
    use crate::eval::test_util::try_eval_with_deserialize;
    use crate::eval::EvalError;
    use ergotree_ir::chain::context::Context;

    fn make_ctx_with_self_box(self_box: ErgoBox) -> Context<'static> {
        let ctx = force_any_val::<Context>();
        Context {
            height: 0u32,
            self_box: Box::leak(Box::new(self_box)),
            ..ctx
        }
    }

    #[test]
    fn eval() {
        // SInt
        let inner_expr: Expr = BinOp {
            kind: RelationOp::NEq.into(),
            left: Box::new(GlobalVars::Height.into()),
            right: Box::new(1i32.into()),
        }
        .into();
        let reg_value: Constant = inner_expr.sigma_serialize_bytes().unwrap().into();
        let b = force_any_val::<ErgoBox>()
            .with_additional_registers(vec![reg_value].try_into().unwrap());
        // expected SBoolean
        let expr: Expr = DeserializeRegister {
            reg: NonMandatoryRegisterId::R4.into(),
            tpe: SType::SBoolean,
            default: None,
        }
        .into();
        let ctx = make_ctx_with_self_box(b);
        assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).unwrap());
    }

    #[test]
    fn eval_reg_is_empty() {
        let b =
            force_any_val::<ErgoBox>().with_additional_registers(NonMandatoryRegisters::empty());
        // no default provided
        let expr: Expr = DeserializeRegister {
            reg: NonMandatoryRegisterId::R5.into(),
            tpe: SType::SBoolean,
            default: None,
        }
        .into();
        let ctx = make_ctx_with_self_box(b.clone());
        assert!(try_eval_out::<Value>(&expr, &ctx).is_err());

        // default with wrong type provided
        let expr: Expr = DeserializeRegister {
            reg: NonMandatoryRegisterId::R5.into(),
            tpe: SType::SInt,
            default: Some(Box::new(true.into())),
        }
        .into();
        let ctx = make_ctx_with_self_box(b.clone());
        assert!(matches!(
            try_eval_with_deserialize::<i32>(&expr, &ctx),
            Err(EvalError::SubstDeserializeError(
                ergotree_ir::mir::expr::SubstDeserializeError::SoftForkError(
                    SoftForkError::DeserializedScriptError
                )
            ))
        ));
        // default provided
        let expr: Expr = DeserializeRegister {
            reg: NonMandatoryRegisterId::R5.into(),
            tpe: SType::SInt,
            default: Some(Box::new(1i32.into())),
        }
        .into();
        let ctx = make_ctx_with_self_box(b);
        assert_eq!(try_eval_with_deserialize::<i32>(&expr, &ctx).unwrap(), 1i32);
    }

    #[test]
    fn eval_reg_wrong_type() {
        // SInt, expected SColl(SByte)
        let reg_value: Constant = 1i32.into();
        let b = force_any_val::<ErgoBox>()
            .with_additional_registers(vec![reg_value].try_into().unwrap());
        let expr: Expr = DeserializeRegister {
            reg: NonMandatoryRegisterId::R4.into(),
            tpe: SType::SBoolean,
            default: None,
        }
        .into();
        let ctx = make_ctx_with_self_box(b);
        assert!(matches!(
            try_eval_with_deserialize::<Value>(&expr, &ctx),
            Err(EvalError::SubstDeserializeError(_))
        ));
    }

    #[test]
    fn evaluated_expr_wrong_type() {
        // SInt
        let inner_expr: Expr = 1i32.into();
        let reg_value: Constant = inner_expr.sigma_serialize_bytes().unwrap().into();
        let b = force_any_val::<ErgoBox>()
            .with_additional_registers(vec![reg_value].try_into().unwrap());
        // expected SBoolean
        let expr: Expr = DeserializeRegister {
            reg: NonMandatoryRegisterId::R4.into(),
            tpe: SType::SBoolean,
            default: None,
        }
        .into();
        let ctx = make_ctx_with_self_box(b);
        assert!(matches!(
            try_eval_with_deserialize::<bool>(&expr, &ctx),
            Err(EvalError::SubstDeserializeError(
                ergotree_ir::mir::expr::SubstDeserializeError::SoftForkError(
                    SoftForkError::DeserializedScriptError
                )
            ))
        ));
    }
}
