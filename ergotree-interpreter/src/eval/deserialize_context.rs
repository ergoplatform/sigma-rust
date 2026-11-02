#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergotree_ir::ergo_tree::{ErgoTree, ErgoTreeHeader, ErgoTreeVersion};
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::deserialize_context::DeserializeContext;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::if_op::If;
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

    // Regression for testnet block 111,927: a `DeserializeContext` over an
    // absent context var sitting on a dead `if` branch must NOT sink reduction.
    // Substitution walks the whole tree but, mirroring the JVM
    // `Interpreter.substDeserialize` `else None`, leaves the absent-var node in
    // place; the live branch then reduces normally. A leftover node on the
    // *live* path still errors at eval (see `eval_id_not_found`).
    #[test]
    fn eval_absent_var_on_dead_branch() {
        let deser: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 0,
        }
        .into();
        // if (true) true else deserializeContext(0)
        let expr: Expr = If {
            condition: Expr::Const(true.into()).into(),
            true_branch: Expr::Const(true.into()).into(),
            false_branch: deser.into(),
        }
        .into();
        let extension = ContextExtension::empty();
        let ctx = force_any_val::<Context>().with_extension(&extension);
        assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).unwrap());
    }

    // Parity with the JVM `Interpreter.substDeserialize` inner `case _ => None`:
    // a context var that is present but not a `Coll[Byte]` is not substituted,
    // so a dead-branch deserialize over a wrong-typed var does not sink
    // reduction either. (A wrong-typed var on the *live* path still errors at
    // eval — see `eval_context_extension_wrong_type`.)
    #[test]
    fn eval_wrong_type_var_on_dead_branch() {
        let deser: Expr = DeserializeContext {
            tpe: SType::SBoolean,
            id: 0,
        }
        .into();
        // if (true) true else deserializeContext(0)
        let expr: Expr = If {
            condition: Expr::Const(true.into()).into(),
            true_branch: Expr::Const(true.into()).into(),
            false_branch: deser.into(),
        }
        .into();
        // var 0 present but an Int, not a Coll[Byte]
        let ctx_ext_val: Constant = 1i32.into();
        let ctx_ext = ContextExtension {
            values: [(0u8, ctx_ext_val)].iter().cloned().collect(),
        };
        let ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).unwrap());
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
}
