use crate::eval::EvalError;

use alloc::boxed::Box;
use alloc::string::ToString;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::RegisterId;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use ergotree_ir::reference::Ref;
use ergotree_ir::types::stype::SType;

use super::EvalFn;

pub(crate) static VALUE_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    Ok(Value::Long(
        obj.try_extract_into::<Ref<'_, ErgoBox>>()?.value.as_i64(),
    ))
};

pub(crate) static GET_REG_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    if ctx.tree_version() < ErgoTreeVersion::V3 {
        return Err(EvalError::ScriptVersionError {
            required_version: ErgoTreeVersion::V3,
            activated_version: ctx.tree_version(),
        });
    }
    let reg_idx = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("register index is missing".to_string()))?
        .try_extract_into::<i32>()?;
    // Mirror JVM CBox.getReg: an out-of-range index (negative or >= maxRegisters)
    // yields None rather than an error; only a present register of the wrong type
    // errors below.
    let reg_id = i8::try_from(reg_idx)
        .ok()
        .and_then(|id| RegisterId::try_from(id).ok());
    let reg_val_opt = match reg_id {
        Some(reg_id) => obj
            .try_extract_into::<Ref<'_, ErgoBox>>()?
            .get_register(reg_id)
            .map_err(|e| {
                EvalError::NotFound(format!(
                    "Error getting the register id {reg_id} with error {e:?}"
                ))
            })?,
        None => None,
    };
    // Return type of getReg[T] is always Option[T]
    #[allow(clippy::unreachable)]
    let SType::SOption(expected_type) = &*mc.tpe().t_range
    else {
        unreachable!()
    };
    match reg_val_opt {
        Some(constant) if constant.tpe == **expected_type => {
            Ok(Value::Opt(Some(Box::new(constant.v.into()))))
        }
        Some(constant) => Err(EvalError::UnexpectedValue(format!(
            "Expected register {reg_idx} to be of type {}, got {}",
            expected_type, constant.tpe
        ))),
        None => Ok(Value::Opt(None)),
    }
};

pub(crate) static TOKENS_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let res: Value = obj
        .try_extract_into::<Ref<'_, ErgoBox>>()?
        .tokens_raw()
        .into();
    Ok(res)
};

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergotree_ir::chain::context_extension::ContextExtension;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::ergo_tree::{ErgoTree, ErgoTreeVersion};
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::types::sbox;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;
    use sigma_test_util::force_any_val;

    use crate::eval::test_util::{eval_out, try_eval_out_with_version};
    use crate::eval::EvalError;
    use ergotree_ir::chain::context::Context;

    // The vector trees carry arbitrary-typed roots (eval-tier corpus); the sized
    // parse path rejects non-SigmaProp roots, so clear the size bit and drop the
    // size byte to route through the non-sized path — the same leniency the
    // conformance runner applies.
    fn parse_tree_lenient(hex: &str) -> Expr {
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let mut lenient = Vec::with_capacity(bytes.len() - 1);
        lenient.push(bytes[0] & !0x08);
        lenient.extend_from_slice(&bytes[2..]); // size VLQ is 1 byte here
        let tree = ErgoTree::sigma_parse_bytes(&lenient).unwrap();
        tree.proposition().unwrap()
    }

    // SELF with exactly R4 = Long(7) and ContextExtension var 1 = `idx` — the
    // setup of the blessed `Box.getReg_dynamic_index` vectors.
    fn ctx_with_r4_long7_and_var1(idx: i32) -> Context<'static> {
        let b = force_any_val::<ErgoBox>()
            .with_additional_registers(vec![Constant::from(7i64)].try_into().unwrap());
        let mut ext = ContextExtension::empty();
        ext.values.insert(1u8, Constant::from(idx));
        Context {
            self_box: Box::leak(Box::new(b)),
            extension: Box::leak(Box::new(ext)),
            ..force_any_val::<Context>()
        }
    }

    #[test]
    fn eval_box_value() {
        let expr: Expr = PropertyCall::new(GlobalVars::SelfBox.into(), sbox::VALUE_METHOD.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<i64>(&expr, &ctx), ctx.self_box.value.as_i64());
    }

    #[test]
    fn eval_box_tokens() {
        let expr: Expr = PropertyCall::new(GlobalVars::SelfBox.into(), sbox::TOKENS_METHOD.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<(Vec<i8>, i64)>>(&expr, &ctx),
            ctx.self_box.tokens_raw()
        );
    }

    #[test]
    fn eval_reg_out() {
        let type_args = std::iter::once((STypeVar::t(), SType::SLong)).collect();
        let expr: Expr = MethodCall::with_type_args(
            GlobalVars::SelfBox.into(),
            sbox::GET_REG_METHOD.clone().with_concrete_types(&type_args),
            vec![Constant::from(0i32).into()],
            type_args,
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        (0..ErgoTreeVersion::V3.into()).for_each(|version| {
            assert!(try_eval_out_with_version::<i64>(&expr, &ctx, version, version).is_err())
        });
        (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
            |version| {
                assert_eq!(
                    try_eval_out_with_version::<Option<i64>>(&expr, &ctx, version, version)
                        .unwrap()
                        .unwrap(),
                    ctx.self_box.value.as_i64()
                )
            },
        );
    }

    // Attempt to extract SigmaProp from register of type SLong
    #[test]
    fn eval_reg_out_wrong_type() {
        let type_args = std::iter::once((STypeVar::t(), SType::SSigmaProp)).collect();
        let expr: Expr = MethodCall::with_type_args(
            GlobalVars::SelfBox.into(),
            sbox::GET_REG_METHOD.clone().with_concrete_types(&type_args),
            vec![Constant::from(0i32).into()],
            type_args,
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        (0..ErgoTreeVersion::V3.into()).for_each(|version| {
            // Pre-V3 the method id is not in the method table at all (min_version),
            // so the versioned roundtrip fails at deserialization — mirroring JVM
            // v5Methods, which has no getReg (id 19) entry.
            let res = try_eval_out_with_version::<Option<i64>>(&expr, &ctx, version, version);
            match res {
                Err(EvalError::SigmaParsingError(_)) => {}
                _ => panic!("Expected method-id parsing rejection, got {:?}", res),
            }
        });
        (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
            |version| {
                assert!(
                    try_eval_out_with_version::<Option<i64>>(&expr, &ctx, version, version)
                        .is_err()
                )
            },
        );
    }

    // End-to-end over the blessed `Box.getReg_dynamic_index` vector trees
    // (sigma-state 6.0.3): `{ SELF.getReg[Long](getVar[Int](1).get) }` with
    // SELF carrying only R4 = Long(7). JVM CBox.getReg yields None for an
    // absent or out-of-range index; only a present register of the wrong
    // type errors.
    #[test]
    fn eval_reg_dynamic_index_absent_or_out_of_range_is_none() {
        const GET_REG_LONG: &str = "1b0b00dc6313a701e4e3010405";
        let run = |idx: i32| -> Option<i64> {
            let expr = parse_tree_lenient(GET_REG_LONG);
            let ctx = ctx_with_r4_long7_and_var1(idx);
            try_eval_out_with_version::<Option<i64>>(&expr, &ctx, 3, 3).unwrap()
        };
        assert_eq!(run(4), Some(7), "present register of matching type");
        assert_eq!(run(5), None, "absent register R5");
        assert_eq!(run(10), None, "index beyond R9");
        assert_eq!(run(-1), None, "negative index");
        assert_eq!(run(1_000_000), None, "index beyond i8 range");

        // The wrong-type boundary stays an error: getReg[Int] over the Long R4.
        const GET_REG_INT: &str = "1b0b00dc6313a701e4e3010404";
        let expr = parse_tree_lenient(GET_REG_INT);
        let ctx = ctx_with_r4_long7_and_var1(4);
        assert!(
            try_eval_out_with_version::<Option<i32>>(&expr, &ctx, 3, 3).is_err(),
            "present register of the wrong type must error"
        );
    }

    // getRegV5 (method id 7) deserializes but has no eval — mirroring the JVM,
    // where getRegMethodV5's reflective lookup fails. A live occurrence errors;
    // a dead-branch occurrence leaves the script evaluable. Trees are the
    // blessed `Box.getReg_adversarial` vectors.
    #[test]
    fn eval_getregv5_parses_but_does_not_eval() {
        // { SELF.getRegV5(getVar[Int](1).get) } — live, must error at eval.
        let expr = parse_tree_lenient("1b0a00dc6307a701e4e30104");
        let ctx = ctx_with_r4_long7_and_var1(4);
        assert!(try_eval_out_with_version::<Value>(&expr, &ctx, 3, 3).is_err());

        // { if (true) true else SELF.getRegV5(getVar[Int](1).get).isDefined }
        // — dead branch, the tree must parse and evaluate to true.
        let expr = parse_tree_lenient("1b1402010101019573007301e6dc6307a701e4e30104");
        let ctx = ctx_with_r4_long7_and_var1(4);
        assert!(try_eval_out_with_version::<bool>(&expr, &ctx, 3, 3).unwrap());
    }
}
