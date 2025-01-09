use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;

use ergotree_ir::mir::avl_tree_data::AvlTreeData;
use ergotree_ir::mir::avl_tree_data::AvlTreeFlags;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::reference::Ref;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::types::stype::SType;

use super::EvalError;
use super::EvalFn;

pub(crate) static DATA_INPUTS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.dataInputs: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    Ok(Value::Coll(CollKind::WrappedColl {
        items: ctx.data_inputs.clone().map_or(Arc::new([]), |d| {
            d.iter().map(|&di| Ref::from(di)).map(Value::CBox).collect()
        }),
        elem_tpe: SType::SBox,
    }))
};

pub(crate) static SELF_BOX_INDEX_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.selfBoxIndex: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    let box_index = ctx
        .inputs
        .iter()
        .position(|it| *it == ctx.self_box)
        .ok_or_else(|| EvalError::NotFound("Context.selfBoxIndex: box not found".to_string()))?;
    Ok(Value::Int(box_index as i32))
};

pub(crate) static HEADERS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.headers: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    Ok(Value::Coll(CollKind::WrappedColl {
        items: Arc::new(ctx.headers.clone().map(Box::new).map(Value::Header)),
        elem_tpe: SType::SHeader,
    }))
};

pub(crate) static PRE_HEADER_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.preHeader: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    Ok(Box::from(ctx.pre_header.clone()).into())
};

pub(crate) static LAST_BLOCK_UTXO_ROOT_HASH_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.LastBlockUtxoRootHash: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    let digest = ctx.headers[0].state_root;
    let tree_flags = AvlTreeFlags::new(true, true, true);
    Ok(Value::AvlTree(Box::from(AvlTreeData {
        digest,
        tree_flags,
        key_length: 32,
        value_length_opt: None,
    })))
};

pub(crate) static MINER_PUBKEY_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.preHeader: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    Ok(ctx
        .pre_header
        .miner_pk
        .clone()
        .sigma_serialize_bytes()?
        .into())
};

pub(crate) static GET_VAR_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.getVar: expected object of Value::Context, got {:?}",
            obj
        )));
    }
    let var_id = match args.first() {
        Some(Value::Byte(id)) => *id as u8,
        _ => {
            return Err(EvalError::UnexpectedValue(
                "getVar: invalid variable id".to_string(),
            ))
        }
    };

    let expected_type = match mc.tpe().t_range.as_ref() {
        SType::SOption(inner_type) => inner_type.as_ref(),
        _ => {
            return Err(EvalError::UnexpectedValue(
                "Expected SOption type".to_string(),
            ))
        }
    };

    match ctx.extension.values.get(&var_id) {
        None => Ok(Value::Opt(Box::new(None))),
        Some(v) if v.tpe == *expected_type => Ok(Value::Opt(Box::new(Some(v.v.clone().into())))),
        Some(_) => Err(EvalError::UnexpectedValue(
            "getVar: type mismatch".to_string(),
        )),
    }
};

pub(crate) static GET_VAR_FROM_INPUTS_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    if obj != Value::Context {
        return Err(EvalError::UnexpectedValue(format!(
            "Context.getVarFromInputs: expected object of Value::Context, got {:?}",
            obj
        )));
    }

    let input_index = match args.first() {
        Some(Value::Short(idx)) if *idx >= 0 => *idx as usize,
        _ => {
            return Err(EvalError::UnexpectedValue(
                "getVarFromInput: invalid input index".to_string(),
            ))
        }
    };
    let var_id = match args.get(1) {
        Some(Value::Byte(id)) => *id as u8,
        _ => {
            return Err(EvalError::UnexpectedValue(
                "getVarFromInput: invalid variable id".to_string(),
            ))
        }
    };

    let expected_type = match mc.tpe().t_range.as_ref() {
        SType::SOption(inner_type) => inner_type.as_ref(),
        _ => {
            return Err(EvalError::UnexpectedValue(
                "Expected SOption type".to_string(),
            ))
        }
    };

    if ctx.inputs.get(input_index).is_none() {
        return Err(EvalError::NotFound(format!(
            "getVarFromInput: input not found at index {}",
            input_index
        )));
    }

    match ctx.extension.values.get(&var_id) {
        None => Ok(Value::Opt(Box::new(None))),
        Some(v) if v.tpe == *expected_type => Ok(Value::Opt(Box::new(Some(v.v.clone().into())))),
        Some(_v) => Ok(Value::Opt(Box::new(None))),
    }
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use crate::eval::tests::eval_out;
    use ergo_chain_types::{Header, PreHeader};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::mir::avl_tree_data::{AvlTreeData, AvlTreeFlags};
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::types::scontext;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;
    use proptest::arbitrary::any;
    use proptest::prelude::ProptestConfig;
    use proptest::proptest;
    use sigma_test_util::force_any_val;

    fn make_ctx_inputs_includes_self_box() -> Context<'static> {
        let ctx = force_any_val::<Context>();
        let self_box = &*Box::leak(Box::new(force_any_val::<ErgoBox>()));
        let inputs = vec![&*Box::leak(Box::new(force_any_val::<ErgoBox>())), self_box]
            .try_into()
            .unwrap();
        Context {
            height: 0u32,
            self_box,
            inputs,
            ..ctx
        }
    }

    fn prepare_getvar_from_input_context<T>(
        input_idx: i16,
        var_idx: u8,
        var_val: T,
    ) -> Context<'static>
    where
        T: Into<Constant> + Clone,
    {
        let mut ctx = force_any_val::<Context>();
        ctx.extension.values.clear();
        ctx.extension.values.insert(var_idx, var_val.into());
        let input_box = &*Box::leak(Box::new(force_any_val::<ErgoBox>()));
        let other_box = &*Box::leak(Box::new(force_any_val::<ErgoBox>()));
        let mut inputs = Vec::new();
        if input_idx > 0 {
            for _ in 0..input_idx {
                inputs.push(other_box);
            }
        }
        inputs.push(input_box);
        ctx.inputs = inputs.try_into().unwrap();
        ctx
    }

    pub fn create_method_call(stype: SType, input_idx: i16, var_idx: u8) -> Expr {
        let type_args = std::iter::once((STypeVar::t(), stype)).collect();
        MethodCall::with_type_args(
            Expr::Context,
            scontext::GET_VAR_FROM_INPUT_METHOD
                .clone()
                .with_concrete_types(&type_args),
            vec![input_idx.into(), (var_idx as i8).into()],
            type_args,
        )
        .unwrap()
        .into()
    }

    fn prepare_getvar_context<T>(var_idx: u8, var_val: T) -> Context<'static>
    where
        T: Into<Constant> + Clone,
    {
        let mut ctx = force_any_val::<Context>();
        ctx.extension.values.clear();
        ctx.extension.values.insert(var_idx, var_val.into());
        ctx
    }

    pub fn create_getvar_method_call(stype: SType, var_idx: u8) -> Expr {
        let type_args = std::iter::once((STypeVar::t(), stype)).collect();
        MethodCall::with_type_args(
            Expr::Context,
            scontext::GET_VAR_METHOD
                .clone()
                .with_concrete_types(&type_args),
            vec![(var_idx as i8).into()],
            type_args,
        )
        .unwrap()
        .into()
    }

    #[test]
    fn eval_self_box_index() {
        let expr: Expr =
            PropertyCall::new(Expr::Context, scontext::SELF_BOX_INDEX_PROPERTY.clone())
                .unwrap()
                .into();
        let context = make_ctx_inputs_includes_self_box();
        assert_eq!(eval_out::<i32>(&expr, &context), 1);
    }

    #[test]
    fn eval_headers() {
        let expr: Expr = PropertyCall::new(Expr::Context, scontext::HEADERS_PROPERTY.clone())
            .expect("internal error: `headers` method has parameters length != 1")
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<[Header; 10]>(&expr, &ctx), ctx.headers);
    }

    #[test]
    fn eval_preheader() {
        let expr: Expr = PropertyCall::new(Expr::Context, scontext::PRE_HEADER_PROPERTY.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<PreHeader>(&expr, &ctx), ctx.pre_header);
    }

    #[test]
    fn eval_miner_pubkey() {
        let expr: Expr = PropertyCall::new(Expr::Context, scontext::MINER_PUBKEY_PROPERTY.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<u8>>(&expr, &ctx),
            ctx.pre_header.miner_pk.sigma_serialize_bytes().unwrap()
        );
    }

    #[test]
    fn eval_last_block_utxo_root_hash() {
        let expr: Expr = PropertyCall::new(
            Expr::Context,
            scontext::LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY.clone(),
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        let digest = ctx.headers[0].state_root;
        let tree_flags = AvlTreeFlags::new(true, true, true);
        let avl_tree_data = AvlTreeData {
            digest,
            tree_flags,
            key_length: 32,
            value_length_opt: None,
        };
        assert_eq!(eval_out::<AvlTreeData>(&expr, &ctx), avl_tree_data);
    }

    #[test]
    fn eval_getvar_from_input_type_mismatch() {
        let ctx = prepare_getvar_from_input_context(0i16, 1u8, 123i32);
        let result = eval_out::<Option<Value>>(&create_method_call(SType::SByte, 0, 1), &ctx);
        assert_eq!(result, None);
    }

    #[test]
    #[should_panic(expected = "getVar: type mismatch")]
    fn eval_getvar_type_mismatch() {
        let ctx = prepare_getvar_context(1u8, 123i32);
        eval_out::<Option<Value>>(&create_getvar_method_call(SType::SByte, 1), &ctx);
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

         #[test]
    fn eval_get_var_from_input_int(
       input_idx in 0..10i16,
       var_idx in any::<u8>(),
       var_val in any::<i32>()
    ) {
       let result = eval_out::<Option<Value>>(
           &create_method_call(SType::SInt, input_idx, var_idx),
           &prepare_getvar_from_input_context(input_idx, var_idx, var_val)
       );
      assert_eq!(result, Some(Value::Int(var_val)));
    }

    #[test]
    fn eval_get_var_from_input_long(
       input_idx in 0..10i16,
       var_idx in any::<u8>(),
       var_val in any::<i64>()
    ) {
       let result = eval_out::<Option<Value>>(
           &create_method_call(SType::SLong, input_idx, var_idx),
           &prepare_getvar_from_input_context(input_idx, var_idx, var_val)
       );
       assert_eq!(result, Some(Value::Long(var_val)));
    }

    #[test]
    fn eval_get_var_from_input_byte(
       input_idx in 0..10i16,
       var_idx in any::<u8>(),
       var_val in any::<i8>()
    ) {
       let result = eval_out::<Option<Value>>(
           &create_method_call(SType::SByte, input_idx, var_idx),
           &prepare_getvar_from_input_context(input_idx, var_idx, var_val)
       );
       assert_eq!(result, Some(Value::Byte(var_val)));
    }

        #[test]
    fn eval_get_var_int(
        var_idx in any::<u8>(),
        var_val in any::<i32>()
    ) {
        let result = eval_out::<Option<Value>>(
            &create_getvar_method_call(SType::SInt, var_idx),
            &prepare_getvar_context(var_idx, var_val)
        );
       assert_eq!(result, Some(Value::Int(var_val)));
    }

    #[test]
    fn eval_get_var_long(
        var_idx in any::<u8>(),
        var_val in any::<i64>()
    ) {
        let result = eval_out::<Option<Value>>(
            &create_getvar_method_call(SType::SLong, var_idx),
            &prepare_getvar_context(var_idx, var_val)
        );
        assert_eq!(result, Some(Value::Long(var_val)));
    }

    #[test]
    fn eval_get_var_byte(
        var_idx in any::<u8>(),
        var_val in any::<i8>()
    ) {
        let result = eval_out::<Option<Value>>(
            &create_getvar_method_call(SType::SByte, var_idx),
            &prepare_getvar_context(var_idx, var_val)
        );
        assert_eq!(result, Some(Value::Byte(var_val)));
    }

        }
}
