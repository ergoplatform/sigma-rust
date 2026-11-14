#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
#[cfg(test)]
mod tests {

    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisterId;
    use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::deserialize_register::DeserializeRegister;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::expr::SubstDeserializeError;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::if_op::If;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    use crate::eval::reduce_to_crypto;
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
                SubstDeserializeError::ExprTpeError {
                    expected: _,
                    actual: _
                }
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
        // node is left unchanged, so it fails when evaluated
        let res = try_eval_with_deserialize::<Value>(&expr, &ctx);
        assert!(is_unexpected_expr(&res), "{res:?}");
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
                ergotree_ir::mir::expr::SubstDeserializeError::ExprTpeError {
                    expected: _,
                    actual: _
                }
            ))
        ));
    }

    fn is_unexpected_expr<T>(res: &Result<T, EvalError>) -> bool {
        let Err(EvalError::Spanned(e)) = res else {
            return false;
        };
        matches!(*e.error, EvalError::UnexpectedExpr(_))
    }

    fn unreached(dr: DeserializeRegister) -> Expr {
        If {
            condition: Box::new(true.into()),
            true_branch: Box::new(true.into()),
            false_branch: Box::new(dr.into()),
        }
        .into()
    }

    #[test]
    fn unreached_non_byte_array_reg_succeeds() {
        let regs: Vec<Constant> = vec![7i32.into(), 1i32.into()];
        let b = force_any_val::<ErgoBox>().with_additional_registers(regs.try_into().unwrap());
        let ctx = make_ctx_with_self_box(b);
        for default in [None, Some(Box::new(true.into()))] {
            let dr = DeserializeRegister {
                reg: NonMandatoryRegisterId::R5.into(),
                tpe: SType::SBoolean,
                default,
            };
            assert!(try_eval_with_deserialize::<bool>(&unreached(dr.clone()), &ctx).unwrap());
            // reached: node is left unchanged and the default is not used
            let res = try_eval_with_deserialize::<bool>(&dr.into(), &ctx);
            assert!(is_unexpected_expr(&res), "{res:?}");
        }
    }

    // ErgoTree vectors with SELF registers, results checked against sigmastate 6.0.7
    #[test]
    fn deserialize_register_tree_vectors() {
        let cases: [(&str, &[&str], bool); 9] = [
            // if (true) true else deserializeRegister[Boolean](R5), R5 = 1: Int
            // sigmastate 6.0.7: true
            ("00d19501010101d5050100", &["040e", "0402"], true),
            // if (true) true else deserializeRegister[Boolean](R5, default = true), R5 = 1: Int
            // sigmastate 6.0.7: true
            ("00d19501010101d50501010101", &["040e", "0402"], true),
            // if (true) true else deserializeRegister[Boolean](R4), R4 = bytes of 1: Int
            // sigmastate 6.0.7: error, the parsed expression's type is checked
            ("00d19501010101d5040100", &["0e020402"], false),
            // if (true) true else deserializeRegister[Boolean](R4, default = true),
            // R4 = bytes of 1: Int; sigmastate 6.0.7: error, the parsed expression's type is checked
            ("00d19501010101d50401010101", &["0e020402"], false),
            // deserializeRegister[Boolean](R5, default = 1: Int), R5 empty
            // sigmastate 6.0.7: error, the Int default is put in place and fails when evaluated
            ("00d1d50501010402", &["040e"], false),
            // deserializeRegister[Boolean](R5, default = true), R5 = 1: Int
            // sigmastate 6.0.7: error
            ("00d1d50501010101", &["040e", "0402"], false),
            // root deserializeRegister[SigmaProp](R5, default = 1: Int), R5 empty
            // sigmastate 6.0.7: error, the Int default put in place is not a SigmaProp
            ("00d50508010402", &["040e"], false),
            // Coll[Int](deserializeRegister[Int](R5, default = 1L)).size == 1, R5 empty
            // sigmastate 6.0.7: error, the Long default fails when the collection is evaluated
            ("00d193b1830104d505040105020402", &["040e"], false),
            // if (true) true else upcast[Long](deserializeRegister[Int](R5, default = true)) == 1L,
            // R5 empty; sigmastate 6.0.7: error, upcast does not accept a Boolean
            ("00d19501010101937ed50504010101050502", &[], false),
        ];
        for (tree_hex, regs, expect_true) in cases {
            let tree = ErgoTree::sigma_parse_bytes(&base16::decode(tree_hex).unwrap()).unwrap();
            let regs: Vec<Constant> = regs
                .iter()
                .map(|r| Constant::sigma_parse_bytes(&base16::decode(r).unwrap()).unwrap())
                .collect();
            let b = force_any_val::<ErgoBox>().with_additional_registers(regs.try_into().unwrap());
            let res = reduce_to_crypto(&tree, &make_ctx_with_self_box(b));
            if expect_true {
                assert_eq!(res.unwrap().sigma_prop, true.into(), "{tree_hex}");
            } else {
                assert!(res.is_err(), "{tree_hex}");
            }
        }
    }
}
