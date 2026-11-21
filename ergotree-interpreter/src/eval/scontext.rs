use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;

use ergotree_ir::mir::avl_tree_data::AvlTreeData;
use ergotree_ir::mir::avl_tree_data::AvlTreeFlags;
use ergotree_ir::mir::constant::TryExtractInto;
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
    // JVM bug compatibility: selfBoxIndex always returned -1 before JIT
    // activation (v5.0). The bug was `eq` (reference equality) instead of
    // `==` (value equality) in CostingDataContext.scala — a global impl
    // bug, not per-script semantics. Fixed in v5.x for ALL scripts.
    // Gate: activated_script_version (block level), NOT tree_version.
    // See: https://github.com/ScorexFoundation/sigmastate-interpreter/issues/603
    if ctx.activated_script_version() < ergotree_ir::ergo_tree::ErgoTreeVersion::V2 {
        return Ok(Value::Int(-1));
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

pub(crate) static GET_VAR_FROM_INPUT_EVAL_FN: EvalFn = |mc, _env, ctx, _obj, args| {
    #[allow(clippy::unreachable)] // getVarFromInput output type is always SOption[T]
    let SType::SOption(output_tpe) = &*mc.tpe().t_range
    else {
        unreachable!()
    };
    let input_idx = args[0].clone().try_extract_into::<i16>()? as usize;
    let var_id = args[1].clone().try_extract_into::<i8>()? as u8;
    Ok(
        match ctx
            .extension_provider
            .context_extension(input_idx)
            .and_then(|extension| extension.values.get(&(var_id)))
            .cloned()
        {
            Some(c) if c.tpe == **output_tpe => Value::Opt(Some(Box::new(c.v.into()))),
            _ => Value::Opt(None),
        },
    )
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use crate::eval::test_util::eval_out;
    use ergo_chain_types::{Header, PreHeader};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::mir::avl_tree_data::{AvlTreeData, AvlTreeFlags};
    use ergotree_ir::mir::constant::TryExtractFrom;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::ergo_tree::ErgoTreeVersion;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::types::scontext::{self, GET_VAR_FROM_INPUT_METHOD};
    use ergotree_ir::types::stype::LiftIntoSType;
    use ergotree_ir::types::stype_param::STypeVar;
    use sigma_test_util::force_any_val;
    use core::cell::Cell;

    fn make_ctx_inputs_includes_self_box(
        tree_version: ErgoTreeVersion,
        pre_header_version: u8,
    ) -> Context<'static> {
        let ctx = force_any_val::<Context>();
        let self_box = &*Box::leak(Box::new(force_any_val::<ErgoBox>()));
        let inputs = vec![&*Box::leak(Box::new(force_any_val::<ErgoBox>())), self_box]
            .try_into()
            .unwrap();
        let pre_header = PreHeader {
            version: pre_header_version,
            ..ctx.pre_header.clone()
        };
        Context {
            height: 0u32,
            self_box,
            inputs,
            pre_header,
            tree_version: Cell::new(tree_version),
            ..ctx
        }
    }

    #[test]
    fn eval_self_box_index_v2_tree() {
        let expr: Expr =
            PropertyCall::new(Expr::Context, scontext::SELF_BOX_INDEX_PROPERTY.clone())
                .unwrap()
                .into();
        // V2 tree in v5+ block (pre_header.version=3 → activated=V2): real index.
        let context = make_ctx_inputs_includes_self_box(ErgoTreeVersion::V2, 3);
        assert_eq!(eval_out::<i32>(&expr, &context), 1);
    }

    #[test]
    fn eval_self_box_index_v0_tree_pre_v5() {
        let expr: Expr =
            PropertyCall::new(Expr::Context, scontext::SELF_BOX_INDEX_PROPERTY.clone())
                .unwrap()
                .into();
        // V0 tree in pre-v5 block (pre_header.version=1 → activated=V0): -1.
        let context = make_ctx_inputs_includes_self_box(ErgoTreeVersion::V0, 1);
        assert_eq!(eval_out::<i32>(&expr, &context), -1);
    }

    #[test]
    fn eval_self_box_index_v1_tree_pre_v5() {
        let expr: Expr =
            PropertyCall::new(Expr::Context, scontext::SELF_BOX_INDEX_PROPERTY.clone())
                .unwrap()
                .into();
        // V1 tree in pre-v5 block (pre_header.version=1 → activated=V0): -1.
        let context = make_ctx_inputs_includes_self_box(ErgoTreeVersion::V1, 1);
        assert_eq!(eval_out::<i32>(&expr, &context), -1);
    }

    #[test]
    fn eval_self_box_index_v0_tree_v5_context() {
        let expr: Expr =
            PropertyCall::new(Expr::Context, scontext::SELF_BOX_INDEX_PROPERTY.clone())
                .unwrap()
                .into();
        // V0 tree in v5+ block (pre_header.version=3 → activated=V2): real index.
        // JVM bug #603 was a global impl bug fixed in v5.x for ALL scripts.
        let context = make_ctx_inputs_includes_self_box(ErgoTreeVersion::V0, 3);
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
    fn eval_get_var_from_input() {
        fn get_var_from_input<T: LiftIntoSType + TryExtractFrom<Value<'static>> + 'static>(
            ctx: &Context<'static>,
            input_index: i16,
            var_id: i8,
        ) -> Option<T> {
            let mc = MethodCall::with_type_args(
                Expr::Context,
                GET_VAR_FROM_INPUT_METHOD.clone(),
                vec![input_index.into(), var_id.into()],
                [(STypeVar::t(), T::stype())].into_iter().collect(),
            )
            .unwrap()
            .into();
            eval_out(&mc, ctx)
        }
        let context = crate::eval::get_var::tests::prepare_context();
        assert_eq!(get_var_from_input::<i32>(&context, 0, 3), Some(123));
        assert_eq!(get_var_from_input::<i64>(&context, 0, 3), None); // wrong type
        assert_eq!(get_var_from_input::<i32>(&context, 0, 4), None); // context extension var doesn't exist
        assert_eq!(
            get_var_from_input::<i32>(&context, context.inputs.len() as i16 + 1, 4),
            None
        ); // input out of bounds
    }
}
