#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergotree_ir::ergo_tree::{ErgoTree, ErgoTreeHeader, ErgoTreeVersion};
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::deserialize_context::DeserializeContext;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::unsignedbigint256::UnsignedBigInt;
    use num_traits::Zero;
    use sigma_test_util::force_any_val;

    use crate::eval::reduce_to_crypto;
    use crate::eval::test_util::try_eval_with_deserialize;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::chain::context_extension::ContextExtension;

    #[test]
    fn eval() {
        let expr: Expr = Expr::from(DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        });
        let inner_expr: Expr = true.into();
        let ctx_ext = ContextExtension {
            values: [(1u8, inner_expr.sigma_serialize_bytes().unwrap().into())]
                .iter()
                .cloned()
                .collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).unwrap());
    }

    // Verify that reduce_to_crypto performs deserialize substitution
    #[test]
    fn eval_reduction() {
        let expr: Expr = Expr::from(DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        });
        let inner_expr: Expr = true.into();
        let ctx_ext = ContextExtension {
            values: [(1u8, inner_expr.sigma_serialize_bytes().unwrap().into())]
                .iter()
                .cloned()
                .collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        assert_eq!(
            reduce_to_crypto(
                &ErgoTree::new(ErgoTreeHeader::v1(false), &expr).unwrap(),
                &ctx,
            )
            .unwrap()
            .sigma_prop,
            true.into()
        );
    }

    #[test]
    fn eval_id_not_found() {
        let expr: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        }
        .into();
        let extension = ContextExtension::empty();
        let ctx = force_any_val::<Context>().with_extension(&extension);
        assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).is_err());
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
        let ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).is_err());
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
        let ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        assert!(try_eval_with_deserialize::<Value>(&expr, &ctx).is_err());
    }

    #[test]
    fn eval_recursive() {
        let expr: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 1,
        }
        .into();
        let ctx_ext = ContextExtension {
            values: [(1u8, expr.sigma_serialize_bytes().unwrap().into())]
                .iter()
                .cloned()
                .collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        // Evaluating executeFromVar(1) with ctx[1] being executeFromVar(1) should fail during evaluation
        assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).is_err());
    }
    #[test]
    fn deserialize_v6_type() {
        let expr: Expr = DeserializeContext {
            tpe: SType::SUnsignedBigInt,
            id: 1,
        }
        .into();
        let inner_expr: Expr = Constant::from(UnsignedBigInt::zero()).into();
        let ctx_ext = ContextExtension {
            values: [(1u8, inner_expr.sigma_serialize_bytes().unwrap().into())]
                .iter()
                .cloned()
                .collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        ctx.tree_version.set(ErgoTreeVersion::V0);
        assert!(try_eval_with_deserialize::<Value>(&expr, &ctx).is_err());
        ctx.tree_version.set(ErgoTreeVersion::V3);
        assert_eq!(
            try_eval_with_deserialize::<UnsignedBigInt>(&expr, &ctx).unwrap(),
            UnsignedBigInt::zero()
        );
    }

    #[test]
    fn reduce_to_crypto_soft_fork_deserialize() {
        use ergotree_ir::chain::ergo_box::{ErgoBox, NonMandatoryRegisterId};
        use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
        use ergotree_ir::mir::deserialize_register::DeserializeRegister;
        use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

        // DeserializeContext with unknown opcode (0xfd) is soft-forkable only when
        // activated_script_version > MAX_SCRIPT_VERSION (Interpreter.scala#L249)
        let expr: Expr = BoolToSigmaProp {
            input: Box::new(
                DeserializeContext {
                    tpe: SType::SBoolean,
                    id: 1,
                }
                .into(),
            ),
        }
        .into();
        let tree = ErgoTree::new(ErgoTreeHeader::v1(true), &expr).unwrap();
        let ctx_ext = ContextExtension {
            values: [(1u8, vec![0xfdu8].into())].into_iter().collect(),
        };
        let mut ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        ctx.pre_header.version = u8::from(ErgoTreeVersion::MAX_SCRIPT_VERSION) + 1;
        assert!(reduce_to_crypto(&tree, &ctx).is_err());

        ctx.pre_header.version = u8::from(ErgoTreeVersion::MAX_SCRIPT_VERSION) + 2;
        assert_eq!(
            reduce_to_crypto(&tree, &ctx).unwrap().sigma_prop,
            SigmaBoolean::TrivialProp(true)
        );

        // DeserializeRegister type mismatch (ExprTpeError) is NOT soft-forkable even after soft-fork
        let reg_expr: Expr = BoolToSigmaProp {
            input: Box::new(
                DeserializeRegister {
                    reg: NonMandatoryRegisterId::R4.into(),
                    tpe: SType::SBoolean,
                    default: None,
                }
                .into(),
            ),
        }
        .into();
        let reg_tree = ErgoTree::new(ErgoTreeHeader::v1(true), &reg_expr).unwrap();
        let wrong_tpe_bytes = Expr::from(GlobalVars::Height)
            .sigma_serialize_bytes()
            .unwrap();
        let self_box = force_any_val::<ErgoBox>()
            .with_additional_registers(vec![Constant::from(wrong_tpe_bytes)].try_into().unwrap());
        let mut reg_ctx = force_any_val::<Context>();
        reg_ctx.self_box = Box::leak(Box::new(self_box));
        reg_ctx.pre_header.version = u8::from(ErgoTreeVersion::MAX_SCRIPT_VERSION) + 2;
        assert!(reduce_to_crypto(&reg_tree, &reg_ctx).is_err());
    }
}
