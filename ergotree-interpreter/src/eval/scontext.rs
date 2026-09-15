use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;

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
        items: ctx
            .headers
            .iter()
            .map(|h| Value::Header(Box::new(h.clone())))
            .collect(),
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
    // The root is a standalone context input, as in the JVM (`CContext` returns
    // `lastBlockUtxoRootHash` directly, never deriving it from `headers(0)`).
    Ok(Value::AvlTree(Box::from(ctx.last_block_utxo_root.clone())))
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
    use crate::eval::test_util::{eval_out, try_eval_out_with_version};
    use ergo_chain_types::{Header, PreHeader};
    use ergotree_ir::chain::context::{Context, ContextHeaders};
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::avl_tree_data::{AvlTreeData, AvlTreeFlags};
    use ergotree_ir::mir::constant::TryExtractFrom;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::value::Value;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::types::scontext::{self, GET_VAR_FROM_INPUT_METHOD};
    use ergotree_ir::types::stype::LiftIntoSType;
    use ergotree_ir::types::stype_param::STypeVar;
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
        assert_eq!(eval_out::<Vec<Header>>(&expr, &ctx), *ctx.headers.as_vec());
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
        let mut ctx = force_any_val::<Context>();
        // Pin the field to a digest distinct from headers[0].state_root: the eval
        // must return the standalone context input (JVM `CContext` semantics),
        // not a value derived from the headers.
        ctx.last_block_utxo_root.digest = [7u8; 33].into();
        assert_ne!(
            ctx.last_block_utxo_root.digest,
            ctx.headers.first().unwrap().state_root
        );
        assert_eq!(
            eval_out::<AvlTreeData>(&expr, &ctx),
            ctx.last_block_utxo_root
        );
    }

    /// Canonical synthetic eval context essentials for the blessed vectors below:
    /// EMPTY headers — the honest value for a contextless eval, which the JVM
    /// expresses (`headers.isEmpty` is legal, `ErgoLikeContext.scala:85`) — and
    /// the dummy root (all-zero 33-byte digest, all operations allowed).
    fn empty_headers_ctx() -> Context<'static> {
        let mut ctx = force_any_val::<Context>();
        ctx.headers = ContextHeaders::from_vec(vec![]).unwrap();
        ctx.last_block_utxo_root = AvlTreeData {
            digest: [0u8; 33].into(),
            tree_flags: AvlTreeFlags::new(true, true, true),
            key_length: 32,
            value_length_opt: None,
        };
        ctx
    }

    /// JVM-blessed byte vectors (santa-eval `Context.properties`, eval/v5/authored):
    /// closed v2 trees. The blessed sized header (`1a` + size VLQ) is rewritten to
    /// the non-sized `12` (size bit cleared, size byte dropped) because the sized
    /// parse path rejects non-SigmaProp roots — the same lenient deserialize the
    /// conformance runner applies to expression-rooted corpus trees; body verbatim.
    fn eval_blessed_context_tree<T: TryExtractFrom<Value<'static>> + 'static>(
        tree_hex: &str,
        ctx: &Context<'static>,
    ) -> T {
        let tree_bytes = base16::decode(tree_hex).unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        let expr = tree.proposition().unwrap();
        try_eval_out_with_version::<T>(&expr, ctx, 2, 2).unwrap()
    }

    #[test]
    fn eval_headers_empty_context_blessed_bytes() {
        // `{ CONTEXT.headers }` (`CONTEXT.headers#dummy`): the JVM yields the
        // context's actual — here empty — header collection.
        let ctx = empty_headers_ctx();
        assert_eq!(
            eval_blessed_context_tree::<Vec<Header>>("1200db6502fe", &ctx),
            Vec::<Header>::new()
        );
    }

    #[test]
    fn eval_last_block_utxo_root_hash_empty_context_blessed_bytes() {
        // `{ CONTEXT.LastBlockUtxoRootHash }` (`CONTEXT.LastBlockUtxoRootHash#dummy`):
        // with no headers the standalone field is the only source of the root —
        // the JVM returns it; a `headers(0)`-derived value cannot express this.
        let ctx = empty_headers_ctx();
        assert_eq!(
            eval_blessed_context_tree::<AvlTreeData>("1200db6509fe", &ctx),
            ctx.last_block_utxo_root
        );
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
