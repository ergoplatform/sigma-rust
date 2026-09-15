use crate::eval::Env;
use alloc::boxed::Box;
use alloc::vec::Vec;
use ergotree_ir::mir::avl_tree_data::AvlTreeData;
use ergotree_ir::mir::avl_tree_data::AvlTreeFlags;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::value::Value;
use ergotree_ir::reference::Ref;
use ergotree_ir::serialization::SigmaSerializable;

use super::Context;
use super::EvalError;
use super::Evaluable;

impl Evaluable for GlobalVars {
    fn eval<'ctx>(&self, _env: &mut Env, ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        match self {
            GlobalVars::Height => Ok((ctx.height as i32).into()),
            GlobalVars::SelfBox => Ok(Value::CBox(Ref::from(ctx.self_box))),
            GlobalVars::Outputs => Ok(ctx
                .outputs
                .iter()
                .map(Ref::Borrowed)
                .collect::<Vec<_>>()
                .into()),
            GlobalVars::Inputs => Ok(ctx
                .inputs
                .iter()
                .map(|&i| Ref::Borrowed(i))
                .collect::<Vec<_>>()
                .into()),
            GlobalVars::MinerPubKey => Ok(ctx.pre_header.miner_pk.sigma_serialize_bytes()?.into()),
            GlobalVars::LastBlockUtxoRootHash => {
                // Same derivation as the `Context.LastBlockUtxoRootHash` property eval
                // (`LAST_BLOCK_UTXO_ROOT_HASH_EVAL_FN`) — the two wire forms of the
                // property must agree.
                let digest = ctx.headers[0].state_root;
                let tree_flags = AvlTreeFlags::new(true, true, true);
                Ok(Value::AvlTree(Box::from(AvlTreeData {
                    digest,
                    tree_flags,
                    key_length: 32,
                    value_length_opt: None,
                })))
            }
            GlobalVars::GroupGenerator => Ok(ergo_chain_types::ec_point::generator().into()),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {

    use crate::eval::test_util::eval_out;
    use ergo_chain_types::EcPoint;
    use ergoscript_compiler::compiler::compile_expr;
    use ergoscript_compiler::script_env::ScriptEnv;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::types::scontext;
    use sigma_test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_height() {
        let ctx = force_any_val::<Context>();
        let expr = compile_expr("HEIGHT", ScriptEnv::new()).unwrap();
        assert_eq!(eval_out::<i32>(&expr, &ctx), ctx.height as i32);
    }

    #[test]
    fn eval_self_box() {
        let ctx = force_any_val::<Context>();
        assert_eq!(
            &*eval_out::<Ref<'_, ErgoBox>>(&GlobalVars::SelfBox.into(), &ctx),
            ctx.self_box
        );
    }

    #[test]
    fn eval_outputs() {
        let ctx = force_any_val::<Context>();

        eval_out::<Vec<Ref<'_, ErgoBox>>>(&GlobalVars::Outputs.into(), &ctx)
            .iter()
            .zip(ctx.outputs)
            .for_each(|(a, b)| assert_eq!(&**a, b));
    }

    #[test]
    fn eval_inputs() {
        let ctx = force_any_val::<Context>();

        eval_out::<Vec<Ref<'_, ErgoBox>>>(&GlobalVars::Inputs.into(), &ctx)
            .iter()
            .zip(ctx.inputs)
            .for_each(|(a, b)| assert_eq!(&**a, b));
    }

    #[test]
    fn eval_group_generator() {
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<EcPoint>(&GlobalVars::GroupGenerator.into(), &ctx),
            ergo_chain_types::ec_point::generator()
        );
    }

    #[test]
    fn eval_last_block_utxo_root_hash_op_form_blessed_bytes() {
        // `10 00 a6` — a v0 constant-segregated tree whose root is the bare
        // dedicated op code 0xa6 (`LastBlockUtxoRootHash`), the way the JVM
        // serializes the op form (`Context.op_forms.json ::
        // lastblockutxoroothash-opform`). The op form and the
        // `CONTEXT.LastBlockUtxoRootHash` PropertyCall form must evaluate to
        // the same AvlTree.
        let ctx = force_any_val::<Context>();
        let tree = ErgoTree::sigma_parse_bytes(&[0x10, 0x00, 0xa6]).unwrap();
        let expr = tree.proposition().unwrap();
        let expected = AvlTreeData {
            digest: ctx.headers[0].state_root,
            tree_flags: AvlTreeFlags::new(true, true, true),
            key_length: 32,
            value_length_opt: None,
        };
        assert_eq!(eval_out::<AvlTreeData>(&expr, &ctx), expected);

        let property_form: Expr = PropertyCall::new(
            Expr::Context,
            scontext::LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY.clone(),
        )
        .unwrap()
        .into();
        assert_eq!(
            eval_out::<AvlTreeData>(&property_form, &ctx),
            eval_out::<AvlTreeData>(&expr, &ctx)
        );
    }
}
