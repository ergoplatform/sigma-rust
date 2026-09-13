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

    // SANTA tx-tier regression (captured testnet tx at height 111,927): trees
    // containing deserialize nodes must charge the JVM's substitution pass —
    // `ergoTree.bytes.length * CostPerTreeByte(2)` block cost, i.e. bytes × 20
    // JitCost (Scala `Interpreter.reductionWithDeserialize`). Since V6
    // activation the charge is part of the reported cost; pre-V6 the JVM
    // checks it against the cost limit but excludes it from the result.
    #[test]
    fn deserialize_substitution_cost_charged() {
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
        let tree = ErgoTree::new(ErgoTreeHeader::v1(false), &expr).unwrap();
        let tree_len = tree.sigma_serialize_bytes().unwrap().len() as u64;

        let run = |block_version: u8| -> (u64, u64) {
            let mut ctx = force_any_val::<Context>().with_extension(&ctx_ext);
            ctx.pre_header.version = block_version;
            ctx.jit_cost.set(0);
            let res = reduce_to_crypto(&tree, &ctx).unwrap();
            assert_eq!(res.sigma_prop, true.into());
            (ctx.jit_cost_value(), res.cost)
        };

        // V6 activated (block version 4 → activated script version 3): the
        // substitution charge is included; pre-V6 it is rolled back after the
        // limit check. The subtraction isolates exactly the per-byte charge.
        let (jit_v6, cost_v6) = run(4);
        let (jit_pre, cost_pre) = run(3);
        assert_eq!(
            jit_v6 - jit_pre,
            tree_len * 20,
            "V6 reduction must charge tree bytes ({tree_len}) × 20 JitCost over pre-V6"
        );
        assert_eq!(
            cost_v6 - cost_pre,
            tree_len * 2,
            "reported block cost must include bytes × CostPerTreeByte(2) since V6"
        );

        // The limit check fires in BOTH eras (Scala's `addCostChecked` runs
        // before the era branch): a limit below the substitution charge must
        // reject even pre-V6, where the charge is excluded from the result.
        let mut ctx = force_any_val::<Context>().with_extension(&ctx_ext);
        ctx.pre_header.version = 3;
        ctx.jit_cost.set(0);
        ctx.jit_cost_limit = Some(tree_len * 20 - 1);
        assert!(
            reduce_to_crypto(&tree, &ctx).is_err(),
            "substitution cost must be limit-checked even pre-V6"
        );
    }

    // Each actually-substituted var charges the JVM's deserialization
    // complexity — `scriptBytes.length × CostPerByteDeserialized(2)` block =
    // bytes × 20 JitCost (`Interpreter.deserializeMeasured`). On this branch
    // an absent var errors (no dead-branch leniency yet), so isolate the
    // charge by varying the substituted payload size: the deserialize node
    // sits on a dead `if` branch, keeping the evaluated path identical
    // between the two runs — only the substitution charge differs.
    #[test]
    fn deserialize_substituted_var_charges_per_byte() {
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

        let run = |inner: &Expr| -> (u64, u64) {
            let bytes = inner.sigma_serialize_bytes().unwrap();
            let len = bytes.len() as u64;
            let ext = ContextExtension {
                values: [(0u8, Constant::from(bytes))].iter().cloned().collect(),
            };
            let ctx = force_any_val::<Context>().with_extension(&ext);
            let before = ctx.jit_cost_value();
            assert!(try_eval_with_deserialize::<bool>(&expr, &ctx).unwrap());
            (ctx.jit_cost_value() - before, len)
        };

        let small: Expr = false.into();
        let big: Expr = If {
            condition: Expr::Const(true.into()).into(),
            true_branch: Expr::Const(false.into()).into(),
            false_branch: Expr::Const(false.into()).into(),
        }
        .into();
        let (cost_small, len_small) = run(&small);
        let (cost_big, len_big) = run(&big);
        assert!(len_big > len_small);
        assert_eq!(
            cost_big - cost_small,
            (len_big - len_small) * 20,
            "substitution must charge the var's bytes × 20 JitCost"
        );
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
