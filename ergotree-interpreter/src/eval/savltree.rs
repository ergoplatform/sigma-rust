use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryFrom;
use ergotree_ir::ergo_tree::ErgoTreeVersion;

use bytes::Bytes;
use ergo_avltree_rust::authenticated_tree_ops::AuthenticatedTreeOps;
use ergo_avltree_rust::batch_avl_verifier::BatchAVLVerifier;
use ergo_avltree_rust::batch_node::AVLTree;
use ergo_avltree_rust::batch_node::Node;
use ergo_avltree_rust::batch_node::NodeHeader;
use ergo_avltree_rust::operation::KeyValue;
use ergo_avltree_rust::operation::Operation;
use ergo_chain_types::ADDigest;
use ergotree_ir::mir::avl_tree_data::AvlTreeData;
use ergotree_ir::mir::avl_tree_data::AvlTreeFlags;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::{CollKind, NativeColl, Value};
use sigma_ser::ScorexSerializable;

use super::Context;
use super::EvalError;
use super::EvalFn;
use ergotree_ir::types::stype::SType;

/// Tree height as recorded in the digest's last byte — the same source Scala's
/// `BatchAVLVerifier.rootNodeHeight` reads (`startingDigest.last & 0xff`), used
/// to scale per-operation verifier costs.
///
/// `rootNodeHeight` is assigned only AFTER reconstruction's up-front
/// `require`s pass (`keyLength > 0`, digest length); when they fail no root is
/// built and the height stays 0, so the JVM charges degenerate trees a
/// zero-height walk (`CAvlTreeVerifier.treeHeight`). `keyLength` is a signed
/// `Int` on the JVM — wire values with the high bit set are negative there.
/// (Failures *during* proof parsing — malformed proof bytes, wrong value
/// length — happen after the assignment and keep the digest-derived height.)
fn tree_height(avl_tree_data: &AvlTreeData) -> u32 {
    if (avl_tree_data.key_length as i32) <= 0 {
        return 0;
    }
    avl_tree_data.digest.0.last().copied().unwrap_or(0) as u32
}

/// Build the batch AVL verifier for `avl_tree_data` over `proof`, charging the
/// proof-size-scaled construction cost: `CreateAvlVerifier_Info` =
/// PerItemCost(110, 20, 64) per proof byte (Scala `CErgoTreeEvaluator.createVerifier`
/// — "the cost of tree reconstruction from proof is O(proof.length)").
fn create_verifier(
    ctx: &Context,
    avl_tree_data: &AvlTreeData,
    proof: &Bytes,
) -> Result<BatchAVLVerifier, EvalError> {
    ctx.add_per_item_jit_cost(110, 20, 64, proof.len() as u32)?;
    let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
    BatchAVLVerifier::new(
        &starting_digest,
        proof,
        AVLTree::new(
            |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
            avl_tree_data.key_length as usize,
            avl_tree_data
                .value_length_opt
                .as_ref()
                .map(|v| **v as usize),
        ),
        None,
        None,
    )
    .map_err(map_eval_err)
}

pub(crate) static DIGEST_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
        avl_tree_data.digest.0.iter().map(|&b| b as i8).collect(),
    ))))
};

pub(crate) static ENABLED_OPERATIONS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Byte(avl_tree_data.tree_flags.serialize() as i8))
};

pub(crate) static KEY_LENGTH_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Int(avl_tree_data.key_length as i32))
};

pub(crate) static VALUE_LENGTH_OPT_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Opt(
        avl_tree_data
            .value_length_opt
            .map(|v| Value::Int(*v as i32))
            .map(Box::new),
    ))
};

pub(crate) static IS_INSERT_ALLOWED_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.insert_allowed()))
};

pub(crate) static IS_UPDATE_ALLOWED_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.update_allowed()))
};

pub(crate) static IS_REMOVE_ALLOWED_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.remove_allowed()))
};

pub(crate) static UPDATE_OPERATIONS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    ctx.add_jit_cost(45)?;
    let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    let new_operations = {
        let v = args.first().cloned().ok_or_else(|| {
            EvalError::AvlTree("eval is missing first arg (new_operations)".to_string())
        })?;
        v.try_extract_into::<i8>()? as u8
    };
    avl_tree_data.tree_flags = AvlTreeFlags::parse(new_operations);
    Ok(Value::AvlTree(Box::new(avl_tree_data)))
};

pub(crate) static UPDATE_DIGEST_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    ctx.add_jit_cost(40)?;
    let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    let new_digest = {
        let v = args.first().cloned().ok_or_else(|| {
            EvalError::AvlTree("eval is missing first arg (new_digest)".to_string())
        })?;
        let bytes_vec = v.try_extract_into::<Vec<u8>>()?;
        ADDigest::try_from(bytes_vec).map_err(map_eval_err)?
    };
    avl_tree_data.digest = new_digest;
    Ok(Value::AvlTree(Box::new(avl_tree_data)))
};

pub(crate) static GET_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    let key = {
        let v = args
            .first()
            .cloned()
            .ok_or_else(|| EvalError::AvlTree("eval is missing first arg (entries)".to_string()))?;
        v.try_extract_into::<Vec<u8>>()?
    };
    let proof = {
        let v = args
            .get(1)
            .cloned()
            .ok_or_else(|| EvalError::AvlTree("eval is missing second arg (proof)".to_string()))?;
        Bytes::from(v.try_extract_into::<Vec<u8>>()?)
    };

    let mut bv = create_verifier(ctx, &avl_tree_data, &proof)?;
    // The cost of a tree lookup is O(treeHeight): `LookupAvlTree_Info` =
    // PerItemCost(40, 10, 1) per height item (Scala `get_eval`).
    ctx.add_per_item_jit_cost(40, 10, 1, tree_height(&avl_tree_data))?;
    match bv.perform_one_operation(&Operation::Lookup(Bytes::from(key))) {
        Ok(opt) => match opt {
            Some(v) => Ok(Value::Opt(Some(Box::new(Value::Coll(
                CollKind::NativeColl(NativeColl::CollByte(v.iter().map(|&b| b as i8).collect())),
            ))))),
            _ => Ok(Value::Opt(None)),
        },
        Err(_) => Err(EvalError::AvlTree(format!(
            "Tree proof is incorrect {:?}",
            avl_tree_data
        ))),
    }
};

pub(crate) static GET_MANY_EVAL_FN: EvalFn =
    |_mc, _env, ctx, obj, args| {
        let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        let keys = {
            let v = args.first().cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing first arg (entries)".to_string())
            })?;
            v.try_extract_into::<Vec<Vec<u8>>>()?
        };
        let proof = {
            let v = args.get(1).cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing second arg (proof)".to_string())
            })?;
            Bytes::from(v.try_extract_into::<Vec<u8>>()?)
        };

        let mut bv = create_verifier(ctx, &avl_tree_data, &proof)?;
        let height = tree_height(&avl_tree_data);
        let res = keys
            .into_iter()
            .map(|key| {
                // Per key, the cost of a tree lookup is O(treeHeight):
                // `LookupAvlTree_Info` = PerItemCost(40, 10, 1) per height item
                // (Scala `getMany_eval`).
                ctx.add_per_item_jit_cost(40, 10, 1, height)?;
                if let Ok(r) = bv.perform_one_operation(&Operation::Lookup(Bytes::from(key))) {
                    if let Some(v) = r {
                        Ok(Value::Opt(Some(Box::new(Value::Coll(
                            CollKind::NativeColl(NativeColl::CollByte(
                                v.iter().map(|&b| b as i8).collect(),
                            )),
                        )))))
                    } else {
                        Ok(Value::Opt(None))
                    }
                } else {
                    Err(EvalError::AvlTree(format!(
                        "Tree proof is incorrect {:?}",
                        avl_tree_data
                    )))
                }
            })
            .collect::<Result<Arc<[_]>, _>>()?;

        Ok(Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SOption(Arc::new(SType::SColl(Arc::new(SType::SByte)))),
            items: res,
        }))
    };

pub(crate) static INSERT_EVAL_FN: EvalFn =
    |_mc, _env, ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        // Flag-check cost: `isInsertAllowed_Info` = FixedCost(15) (Scala `insert_eval`).
        ctx.add_jit_cost(15)?;
        if !avl_tree_data.tree_flags.insert_allowed() {
            return Ok(Value::Opt(None));
        }

        let entries = {
            let v = args.first().cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing first arg (entries)".to_string())
            })?;
            v.try_extract_into::<Vec<(Vec<u8>, Vec<u8>)>>()?
        };

        let proof = {
            let v = args.get(1).cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing second arg (proof)".to_string())
            })?;
            Bytes::from(v.try_extract_into::<Vec<u8>>()?)
        };

        let mut bv = create_verifier(ctx, &avl_tree_data, &proof)?;
        // When the tree is empty we still need to add the insert cost (Scala
        // `insert_eval`: `nItems = Math.max(bv.treeHeight, 1)`).
        let n_items = tree_height(&avl_tree_data).max(1);
        for (key, value) in entries {
            // The cost of a tree insert is O(treeHeight): `InsertIntoAvlTree_Info`
            // = PerItemCost(40, 10, 1) per height item.
            ctx.add_per_item_jit_cost(40, 10, 1, n_items)?;
            if bv
                .perform_one_operation(&Operation::Insert(KeyValue {
                    key: key.into(),
                    value: value.into(),
                }))
                .is_err()
            {
                if ctx.tree_version() >= ErgoTreeVersion::V3 {
                    break;
                } else {
                    return Err(EvalError::AvlTree(format!(
                        "Incorrect insert for {:?}",
                        avl_tree_data
                    )));
                }
            }
        }
        Ok(if let Some(new_digest) = bv.digest() {
            // Digest extraction after mutation: `updateDigest_Info` = FixedCost(40).
            ctx.add_jit_cost(40)?;
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Value::Opt(Some(Box::new(Value::AvlTree(avl_tree_data.into()))))
        } else {
            Value::Opt(None)
        })
    };

pub(crate) static REMOVE_EVAL_FN: EvalFn =
    |_mc, _env, ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        // Flag-check cost: `isRemoveAllowed_Info` = FixedCost(15) (Scala `remove_eval`).
        ctx.add_jit_cost(15)?;
        if !avl_tree_data.tree_flags.remove_allowed() {
            return Ok(Value::Opt(None));
        }

        let keys = {
            let v = args.first().cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing first arg (keys)".to_string())
            })?;
            v.try_extract_into::<Vec<Vec<u8>>>()?
        };

        let proof = {
            let v = args.get(1).cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing second arg (proof)".to_string())
            })?;
            Bytes::from(v.try_extract_into::<Vec<u8>>()?)
        };

        let mut bv = create_verifier(ctx, &avl_tree_data, &proof)?;
        // When the tree is empty we still need to add the remove cost (Scala
        // `remove_eval`: `nItems = Math.max(bv.treeHeight, 1)`).
        let n_items = tree_height(&avl_tree_data).max(1);
        for key in keys {
            // The cost of a tree remove is O(treeHeight): `RemoveAvlTree_Info`
            // = PerItemCost(100, 15, 1) per height item.
            ctx.add_per_item_jit_cost(100, 15, 1, n_items)?;
            if bv
                .perform_one_operation(&Operation::Remove(Bytes::from(key)))
                .is_err()
            {
                return Err(EvalError::AvlTree(format!(
                    "Incorrect remove for {:?}",
                    avl_tree_data
                )));
            }
        }
        // Digest read after the operation loop: `digest_Info` = FixedCost(15)
        // (Scala `remove_eval` charges it unconditionally, unlike insert/update).
        ctx.add_jit_cost(15)?;
        if let Some(new_digest) = bv.digest() {
            // Digest extraction after mutation: `updateDigest_Info` = FixedCost(40).
            ctx.add_jit_cost(40)?;
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Ok(Value::Opt(Some(Box::new(Value::AvlTree(
                avl_tree_data.into(),
            )))))
        } else {
            Ok(Value::Opt(None))
        }
    };

pub(crate) static CONTAINS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    let key = {
        let v = args
            .first()
            .cloned()
            .ok_or_else(|| EvalError::AvlTree("eval is missing first arg (key)".to_string()))?;
        Bytes::from(v.try_extract_into::<Vec<u8>>()?)
    };

    let proof = {
        let v = args
            .get(1)
            .cloned()
            .ok_or_else(|| EvalError::AvlTree("eval is missing second arg (proof)".to_string()))?;
        Bytes::from(v.try_extract_into::<Vec<u8>>()?)
    };

    let mut bv = create_verifier(ctx, &avl_tree_data, &proof)?;
    // The cost of a tree lookup is O(treeHeight): `LookupAvlTree_Info` =
    // PerItemCost(40, 10, 1) per height item (Scala `contains_eval`).
    ctx.add_per_item_jit_cost(40, 10, 1, tree_height(&avl_tree_data))?;
    Ok(match bv.perform_one_operation(&Operation::Lookup(key)) {
        Ok(s) => match s {
            Some(_e) => Value::Boolean(true),
            _ => Value::Boolean(false),
        },
        Err(_) => Value::Boolean(false),
    })
};

pub(crate) static UPDATE_EVAL_FN: EvalFn =
    |_mc, _env, ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        // Flag-check cost: `isUpdateAllowed_Info` = FixedCost(15) (Scala `update_eval`).
        ctx.add_jit_cost(15)?;
        if !avl_tree_data.tree_flags.update_allowed() {
            return Ok(Value::Opt(None));
        }

        let entries = {
            let v = args.first().cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing first arg (entries)".to_string())
            })?;
            v.try_extract_into::<Vec<(Vec<u8>, Vec<u8>)>>()?
        };

        let proof = {
            let v = args.get(1).cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing second arg (proof)".to_string())
            })?;
            Bytes::from(v.try_extract_into::<Vec<u8>>()?)
        };

        let mut bv = create_verifier(ctx, &avl_tree_data, &proof)?;
        // When the tree is empty we still need to add the update cost (Scala
        // `update_eval`: `nItems = Math.max(bv.treeHeight, 1)`).
        let n_items = tree_height(&avl_tree_data).max(1);
        for (key, value) in entries {
            // The cost of a tree update is O(treeHeight): `UpdateAvlTree_Info`
            // = PerItemCost(120, 20, 1) per height item.
            ctx.add_per_item_jit_cost(120, 20, 1, n_items)?;
            if bv
                .perform_one_operation(&Operation::Update(KeyValue {
                    key: key.into(),
                    value: value.into(),
                }))
                .is_err()
            {
                break;
            }
        }
        Ok(if let Some(new_digest) = bv.digest() {
            // Digest extraction after mutation: `updateDigest_Info` = FixedCost(40).
            ctx.add_jit_cost(40)?;
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Value::Opt(Some(Value::AvlTree(avl_tree_data.into()).into()))
        } else {
            Value::Opt(None)
        })
    };

pub(crate) static INSERT_OR_UPDATE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, args| {
    let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

    // Flag-check costs: `isUpdateAllowed_Info` + `isInsertAllowed_Info` =
    // FixedCost(15) each (Scala `insertOrUpdate_eval` charges both).
    ctx.add_jit_cost(15)?;
    ctx.add_jit_cost(15)?;
    if !avl_tree_data.tree_flags.insert_allowed() || !avl_tree_data.tree_flags.update_allowed() {
        return Ok(Value::Opt(None));
    }

    let entries = {
        let v = args
            .first()
            .cloned()
            .ok_or_else(|| EvalError::AvlTree("eval is missing first arg (entries)".to_string()))?;
        v.try_extract_into::<Vec<(Vec<u8>, Vec<u8>)>>()?
    };

    let proof = {
        let v = args
            .get(1)
            .cloned()
            .ok_or_else(|| EvalError::AvlTree("eval is missing second arg (proof)".to_string()))?;
        Bytes::from(v.try_extract_into::<Vec<u8>>()?)
    };

    let mut bv = create_verifier(ctx, &avl_tree_data, &proof)?;
    // When the tree is empty we still need to add the insert cost (Scala
    // `insertOrUpdate_eval`: `nItems = Math.max(bv.treeHeight, 1)`).
    let n_items = tree_height(&avl_tree_data).max(1);
    for (key, value) in entries {
        // Per entry, Scala `insertOrUpdate_eval` charges `UpdateAvlTree_Info`
        // = PerItemCost(120, 20, 1) per height item.
        ctx.add_per_item_jit_cost(120, 20, 1, n_items)?;
        if bv
            .perform_one_operation(&Operation::InsertOrUpdate(KeyValue {
                key: key.into(),
                value: value.into(),
            }))
            .is_err()
        {
            break;
        }
    }
    Ok(if let Some(new_digest) = bv.digest() {
        // Digest extraction after mutation: `updateDigest_Info` = FixedCost(40).
        ctx.add_jit_cost(40)?;
        let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
        avl_tree_data.digest = digest;
        Value::Opt(Some(Box::new(Value::AvlTree(avl_tree_data.into()))))
    } else {
        Value::Opt(None)
    })
};

fn map_eval_err<T: core::fmt::Debug>(e: T) -> EvalError {
    EvalError::AvlTree(format!("{:?}", e))
}

#[allow(clippy::unwrap_used, clippy::panic, clippy::unreachable)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use alloc::sync::Arc;

    use ergo_avltree_rust::batch_avl_prover::BatchAVLProver;
    use ergotree_ir::{
        mir::{
            avl_tree_data::{AvlTreeData, AvlTreeFlags},
            constant::{Constant, Literal},
            expr::Expr,
            method_call::MethodCall,
            value::CollKind,
        },
        types::{savltree, stuple::STuple, stype::SType},
    };
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out_with_version};

    use super::*;
    use sigma_util::{AsVecI8, AsVecU8};

    #[test]
    fn eval_avl_get() {
        let mut prover = populate_tree(vec![(vec![1u8], 10u64.to_be_bytes().to_vec())]);
        let initial_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();

        let key1 = Bytes::from(vec![1u8]);
        let key2 = Bytes::from(vec![2u8]);
        let op1 = Operation::Lookup(key1);
        let op2 = Operation::Lookup(key2);
        let lookup_found = prover.perform_one_operation(&op1).unwrap();
        let lookup_not_found = prover.perform_one_operation(&op2).unwrap();
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let tree_flags = AvlTreeFlags::new(false, false, false);
        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );

        let search_key_found = vec![1i8];
        let search_key_not_found = vec![2i8];
        let expr_found: Expr = MethodCall::new(
            obj.clone(),
            savltree::GET_METHOD.clone(),
            vec![search_key_found.into(), proof.clone().into()],
        )
        .unwrap()
        .into();
        let expr_not_found: Expr = MethodCall::new(
            obj,
            savltree::GET_METHOD.clone(),
            vec![search_key_not_found.into(), proof.into()],
        )
        .unwrap()
        .into();

        let res_found = eval_out_wo_ctx::<Value>(&expr_found);
        let res_not_found = eval_out_wo_ctx::<Value>(&expr_not_found);

        if let Value::Opt(opt) = res_found {
            if let Some(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b)))) = opt.as_deref()
            {
                assert!(lookup_found.unwrap().eq(&b.as_vec_u8()));
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }

        if let Value::Opt(opt) = res_not_found {
            assert!(lookup_not_found.is_none() && opt.is_none())
        } else {
            unreachable!();
        }
    }

    #[test]
    fn eval_avl_get_many() {
        let mut prover = populate_tree(vec![
            (vec![1u8], 10u64.to_be_bytes().to_vec()),
            (vec![2u8], 20u64.to_be_bytes().to_vec()),
        ]);

        let initial_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();

        let key1 = Bytes::from(vec![1u8]);
        let key2 = Bytes::from(vec![2u8]);
        let key3 = Bytes::from(vec![3u8]);
        let op1 = Operation::Lookup(key1);
        let op2 = Operation::Lookup(key2);
        let op3 = Operation::Lookup(key3);
        let lookups = vec![
            prover.perform_one_operation(&op1).unwrap(),
            prover.perform_one_operation(&op2).unwrap(),
            prover.perform_one_operation(&op3).unwrap(),
        ];

        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let tree_flags = AvlTreeFlags::new(false, false, false);
        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );

        let search_key_1 = Literal::from(vec![1u8]);
        let search_key_2 = Literal::from(vec![2u8]);
        let search_key_3 = Literal::from(vec![3u8]);

        let keys = Constant {
            tpe: SType::SColl(Arc::new(SType::SColl(Arc::new(SType::SByte)))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([search_key_1, search_key_2, search_key_3]),
                elem_tpe: SType::SColl(Arc::new(SType::SByte)),
            }),
        };

        let expr: Expr = MethodCall::new(
            obj,
            savltree::GET_MANY_METHOD.clone(),
            vec![keys.into(), proof.into()],
        )
        .unwrap()
        .into();

        let res = eval_out_wo_ctx::<Value>(&expr);

        if let Value::Coll(CollKind::WrappedColl { items, .. }) = res {
            for (item, expected) in items.iter().zip(lookups) {
                if let Value::Opt(opt) = item.clone() {
                    match opt.as_deref() {
                        None => assert!(expected.is_none()),
                        Some(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b)))) => {
                            assert_eq!(&b[..], &expected.unwrap().to_vec().as_vec_i8()[..]);
                        }
                        Some(_) => unreachable!(),
                    }
                } else {
                    unreachable!();
                }
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn eval_avl_insert() {
        // This example taken from `ergo_avltree_rust` README
        let mut prover = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        let initial_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let key1 = Bytes::from(vec![1u8]);
        let key2 = Bytes::from(vec![2u8; 1]);
        let key3 = Bytes::from(vec![3u8; 1]);
        let op1 = Operation::Insert(KeyValue {
            key: key1,
            value: Bytes::from(10u64.to_be_bytes().to_vec()),
        });
        let op2 = Operation::Insert(KeyValue {
            key: key2,
            value: Bytes::from(20u64.to_be_bytes().to_vec()),
        });
        let op3 = Operation::Insert(KeyValue {
            key: key3,
            value: Bytes::from(30u64.to_be_bytes().to_vec()),
        });
        prover.perform_one_operation(&op1).unwrap();
        prover.perform_one_operation(&op2).unwrap();
        prover.perform_one_operation(&op3).unwrap();
        let final_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let tree_flags = AvlTreeFlags::new(true, false, false);
        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );
        let pair1 = Literal::Tup(mk_pair(1u8, 10u64).into());
        let pair2 = Literal::Tup(mk_pair(2u8, 20u64).into());
        let pair3 = Literal::Tup(mk_pair(3u8, 30u64).into());
        let entries = Constant {
            tpe: SType::SColl(Arc::new(SType::STuple(STuple::pair(
                SType::SColl(Arc::new(SType::SByte)),
                SType::SColl(Arc::new(SType::SByte)),
            )))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([pair1.clone(), pair2.clone(), pair3.clone()]),
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Arc::new(SType::SByte)),
                    SType::SColl(Arc::new(SType::SByte)),
                )),
            }),
        };
        let expr: Expr = MethodCall::new(
            obj.clone(),
            savltree::INSERT_METHOD.clone(),
            vec![entries.clone().into(), proof.clone().into()],
        )
        .unwrap()
        .into();

        let res = eval_out_wo_ctx::<Value>(&expr);
        if let Value::Opt(opt) = res {
            if let Some(Value::AvlTree(avl)) = opt.as_deref() {
                assert_eq!(avl.digest, final_digest);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }

        // perform invalid insertion (duplicate entries). Before v6.0 this would return an error. After 6.0 this will return None
        let duplicate_entries = Constant {
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([pair1, pair2, pair3.clone(), pair3]),
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Arc::new(SType::SByte)),
                    SType::SColl(Arc::new(SType::SByte)),
                )),
            }),
            ..entries
        };
        let expr: Expr = MethodCall::new(
            obj,
            savltree::INSERT_METHOD.clone(),
            vec![duplicate_entries.into(), proof.into()],
        )
        .unwrap()
        .into();
        assert!(try_eval_out_with_version::<Value<'_>>(&expr, &force_any_val(), 0, 3).is_err());
        assert_eq!(
            try_eval_out_with_version::<Value<'_>>(&expr, &force_any_val(), 3, 3).unwrap(),
            Value::Opt(None)
        );
    }

    #[test]
    fn eval_avl_insert_or_update() {
        let mut prover = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        let initial_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let key1 = Bytes::from(vec![1u8]);
        let op = Operation::InsertOrUpdate(KeyValue {
            key: key1,
            value: Bytes::from(10u64.to_be_bytes().to_vec()),
        });
        prover.perform_one_operation(&op).unwrap();
        prover.perform_one_operation(&op).unwrap();
        let final_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let tree_flags = AvlTreeFlags::new(true, true, false);
        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );
        let pair1 = Literal::Tup(mk_pair(1u8, 10u64).into());
        let entries = Constant {
            tpe: SType::SColl(Arc::new(SType::STuple(STuple::pair(
                SType::SColl(Arc::new(SType::SByte)),
                SType::SColl(Arc::new(SType::SByte)),
            )))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([pair1.clone(), pair1.clone()]),
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Arc::new(SType::SByte)),
                    SType::SColl(Arc::new(SType::SByte)),
                )),
            }),
        };
        let expr: Expr = MethodCall::new(
            obj.clone(),
            savltree::INSERT_OR_UPDATE_METHOD.clone(),
            vec![entries.clone().into(), proof.clone().into()],
        )
        .unwrap()
        .into();

        let res = eval_out_wo_ctx::<Value>(&expr);
        if let Value::Opt(opt) = res {
            if let Some(Value::AvlTree(avl)) = opt.as_deref() {
                assert_eq!(avl.digest, final_digest);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }
    proptest! {
        #[test]
        fn eval_avl_digest(v in any::<AvlTreeData>()) {
            let digest: Vec<i8> = v.digest.into();
            let obj = Expr::Const(v.into());

            let expr: Expr = MethodCall::new(
                obj,
                savltree::DIGEST_METHOD.clone(),
                vec![],
            )
            .unwrap()
            .into();

            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b))) = res {
                assert_eq!(&b[..], digest.as_slice());
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_enabled_operations(v in any::<AvlTreeData>()) {
            let enabled_ops = v.tree_flags.serialize() as i8;
            let obj = Expr::Const(v.into());

            let expr: Expr = MethodCall::new(
                obj,
                savltree::ENABLED_OPERATIONS_METHOD.clone(),
                vec![],
            )
            .unwrap()
            .into();

            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::Byte(b) = res {
                assert_eq!(b, enabled_ops);
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_key_length(v in any::<AvlTreeData>()) {
            let key_length = v.key_length as i32;
            let obj = Expr::Const(v.into());

            let expr: Expr = MethodCall::new(
                obj,
                savltree::KEY_LENGTH_METHOD.clone(),
                vec![],
            )
            .unwrap()
            .into();

            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::Int(i) = res {
                assert_eq!(key_length, i);
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_value_length_opt(v in any::<AvlTreeData>()) {
            let value_length_opt = v.value_length_opt.clone().map(|v| Value::Int(*v as i32));
            let obj = Expr::Const(v.into());

            let expr: Expr = MethodCall::new(
                obj,
                savltree::VALUE_LENGTH_OPT_METHOD.clone(),
                vec![],
            )
            .unwrap()
            .into();

            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::Opt(opt) = res {
                assert_eq!(opt.as_deref().cloned(), value_length_opt);
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_insert_allowed(v in any::<AvlTreeData>()) {
            let insert_allowed = v.tree_flags.insert_allowed();
            let obj = Expr::Const(v.into());

            let expr: Expr = MethodCall::new(
                obj,
                savltree::IS_INSERT_ALLOWED_METHOD.clone(),
                vec![],
            )
            .unwrap()
            .into();
            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::Boolean(i) = res {
                assert_eq!(insert_allowed, i);
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_update_allowed(v in any::<AvlTreeData>()) {
            let update_allowed = v.tree_flags.update_allowed();
            let obj = Expr::Const(v.into());

            let expr: Expr = MethodCall::new(
                obj,
                savltree::IS_UPDATE_ALLOWED_METHOD.clone(),
                vec![],
            )
            .unwrap()
            .into();
            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::Boolean(i) = res {
                assert_eq!(update_allowed, i);
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_remove_allowed(v in any::<AvlTreeData>()) {
            let remove_allowed = v.tree_flags.remove_allowed();
            let obj = Expr::Const(v.into());

            let expr: Expr = MethodCall::new(
                obj,
                savltree::IS_REMOVE_ALLOWED_METHOD.clone(),
                vec![],
            )
            .unwrap()
            .into();
            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::Boolean(i) = res {
                assert_eq!(remove_allowed, i);
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_update_operations(v in any::<AvlTreeData>(), new_ops in any::<AvlTreeFlags>()) {
            // Test updateOperations method
            let obj = Expr::Const(v.into());
            let expr: Expr = MethodCall::new(
                obj,
                savltree::UPDATE_OPERATIONS_METHOD.clone(),
                vec![Constant::from(new_ops.serialize() as i8).into()],
            )
            .unwrap()
            .into();
            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::AvlTree(a) = res {
                assert_eq!(a.tree_flags, new_ops);
            } else {
                unreachable!();
            }
        }

        #[test]
        fn eval_avl_update_digest(v in any::<AvlTreeData>(), new_digest in any::<ADDigest>()) {
            let obj = Expr::Const(v.into());
            let expr: Expr = MethodCall::new(
                obj,
                savltree::UPDATE_DIGEST_METHOD.clone(),
                vec![Constant::from(new_digest.scorex_serialize_bytes()?).into()],
            )
            .unwrap()
            .into();
            let res = eval_out_wo_ctx::<Value>(&expr);
            if let Value::AvlTree(a) = res {
                assert_eq!(a.digest, new_digest);
            } else {
                unreachable!();
            }
        }
    }

    #[test]
    fn eval_avl_contains() {
        let mut prover = populate_tree(vec![
            (vec![1u8], 10u64.to_be_bytes().to_vec()),
            (vec![2u8], 20u64.to_be_bytes().to_vec()),
            (vec![3u8], 30u64.to_be_bytes().to_vec()),
        ]);
        let digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();

        let op = Operation::Lookup(Bytes::from(vec![2u8]));
        prover.perform_one_operation(&op).unwrap();

        let key = Constant::from(vec![2u8]);
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();
        let tree_flags = AvlTreeFlags::new(false, false, false);
        let obj = Expr::Const(
            AvlTreeData {
                digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );
        let expr: Expr = MethodCall::new(
            obj,
            savltree::CONTAINS_METHOD.clone(),
            vec![key.into(), proof.into()],
        )
        .unwrap()
        .into();

        assert!(eval_out_wo_ctx::<bool>(&expr));
    }

    #[test]
    fn eval_avl_remove() {
        let mut prover = populate_tree(vec![(vec![1u8], 10u64.to_be_bytes().to_vec())]);
        let initial_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();

        let key1 = Bytes::from(vec![1u8]);
        let op1 = Operation::Remove(key1);
        prover.perform_one_operation(&op1).unwrap();
        let final_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let tree_flags = AvlTreeFlags::new(false, false, true);
        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );

        let key1 = Literal::from(vec![1u8]);
        let keys = Constant {
            tpe: SType::SColl(Arc::new(SType::SColl(Arc::new(SType::SByte)))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([key1]),
                elem_tpe: SType::SColl(Arc::new(SType::SByte)),
            }),
        };
        let expr: Expr = MethodCall::new(
            obj,
            savltree::REMOVE_METHOD.clone(),
            vec![keys.into(), proof.into()],
        )
        .unwrap()
        .into();

        let res = eval_out_wo_ctx::<Value>(&expr);
        if let Value::Opt(opt) = res {
            if let Some(Value::AvlTree(avl)) = opt.as_deref() {
                assert_eq!(avl.digest, final_digest);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    #[test]
    fn eval_avl_update() {
        let mut prover = populate_tree(vec![
            (vec![1u8], 10u64.to_be_bytes().to_vec()),
            (vec![2u8], 20u64.to_be_bytes().to_vec()),
            (vec![3u8], 30u64.to_be_bytes().to_vec()),
        ]);
        let initial_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();

        let op1 = Operation::Update(KeyValue {
            key: Bytes::from(vec![2u8]),
            value: Bytes::from(40u64.to_be_bytes().to_vec()),
        });
        let op2 = Operation::Update(KeyValue {
            key: Bytes::from(vec![3u8]),
            value: Bytes::from(50u64.to_be_bytes().to_vec()),
        });
        prover.perform_one_operation(&op1).unwrap();
        prover.perform_one_operation(&op2).unwrap();

        let final_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let tree_flags = AvlTreeFlags::new(false, true, false);
        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );

        let pair1 = Literal::Tup(mk_pair(2u8, 40u64).into());
        let pair2 = Literal::Tup(mk_pair(3u8, 50u64).into());
        let entries = Constant {
            tpe: SType::SColl(Arc::new(SType::STuple(STuple::pair(
                SType::SColl(Arc::new(SType::SByte)),
                SType::SColl(Arc::new(SType::SByte)),
            )))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([pair1, pair2]),
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Arc::new(SType::SByte)),
                    SType::SColl(Arc::new(SType::SByte)),
                )),
            }),
        };
        let expr: Expr = MethodCall::new(
            obj,
            savltree::UPDATE_METHOD.clone(),
            vec![entries.into(), proof.into()],
        )
        .unwrap()
        .into();

        let res = eval_out_wo_ctx::<Value>(&expr);
        if let Value::Opt(opt) = res {
            if let Some(Value::AvlTree(avl)) = opt.as_deref() {
                assert_eq!(avl.digest, final_digest);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }

    fn populate_tree(entries: Vec<(Vec<u8>, Vec<u8>)>) -> BatchAVLProver {
        let mut prover = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );

        for (key, value) in entries {
            let op = Operation::Insert(KeyValue {
                key: Bytes::from(key),
                value: Bytes::from(value),
            });
            prover.perform_one_operation(&op).unwrap();
        }

        prover.generate_proof();
        prover
    }

    fn mk_pair(x: u8, y: u64) -> [Literal; 2] {
        [
            Literal::from(vec![x]),
            Literal::from(y.to_be_bytes().to_vec()),
        ]
    }

    /// SANTA tx-tier regression (captured testnet txs at heights 2666 / 28474):
    /// the verifier-backed AvlTree ops must charge the JVM's verifier costs
    /// (Scala `CErgoTreeEvaluator.createVerifier` + per-op height-scaled costs).
    /// Pre-fix all seven ops charged nothing, under-counting the aggregate tx
    /// cost of any AVL-using script — the consensus-risky direction near
    /// MaxBlockCost.
    ///
    /// Expectations are formula-derived (no magic totals), mirroring Scala:
    ///   createVerifier = 110 + 20 * chunks(proof.len, 64)
    ///   lookup         = 40 + 10 * h          (contains/get; per key in getMany)
    ///   insert         = 40 + 10 * max(h, 1)  per entry
    ///   remove         = 100 + 15 * max(h, 1) per key, then digest read (15)
    ///   updateDigest   = 40 on successful mutation
    /// where h = the digest's height byte (what `BatchAVLVerifier.rootNodeHeight`
    /// reads) and chunks(n, sz) = (n - 1)/sz + 1.
    #[test]
    fn avl_verifier_op_costs_match_jvm() {
        use crate::eval::test_util::try_eval_out;
        use ergotree_ir::chain::context::Context;

        let chunks = |n: u64, sz: u64| (n - 1) / sz + 1;
        // Expr overhead outside the AvlTree method eval itself:
        // Const(tree) 5 + MethodCall 4 + Const(arg1) 5 + Const(arg2) 5.
        const EXPR_OVERHEAD: u64 = 19;

        let cost_of = |expr: &Expr| -> u64 {
            let ctx = force_any_val::<Context>();
            let before = ctx.jit_cost_value();
            let _ = try_eval_out::<Value>(expr, &ctx).unwrap();
            ctx.jit_cost_value() - before
        };

        let keys_const = |keys: &[Vec<u8>]| Constant {
            tpe: SType::SColl(Arc::new(SType::SColl(Arc::new(SType::SByte)))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: keys.iter().map(|k| Literal::from(k.clone())).collect(),
                elem_tpe: SType::SColl(Arc::new(SType::SByte)),
            }),
        };
        let tree_const = |digest: &ADDigest, flags: AvlTreeFlags| {
            Expr::Const(
                AvlTreeData {
                    digest: *digest,
                    tree_flags: flags,
                    key_length: 1,
                    value_length_opt: None,
                }
                .into(),
            )
        };

        // --- get: createVerifier + one height-scaled lookup ---
        let mut prover = populate_tree(vec![(vec![1u8], 10u64.to_be_bytes().to_vec())]);
        let digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let h = u64::from(*digest.0.last().unwrap());
        prover
            .perform_one_operation(&Operation::Lookup(Bytes::from(vec![1u8])))
            .unwrap();
        let proof_bytes: Vec<u8> = prover.generate_proof().into_iter().collect();
        let plen = proof_bytes.len() as u64;
        let expr: Expr = MethodCall::new(
            tree_const(&digest, AvlTreeFlags::new(false, false, false)),
            savltree::GET_METHOD.clone(),
            vec![
                Constant::from(vec![1u8]).into(),
                Constant::from(proof_bytes).into(),
            ],
        )
        .unwrap()
        .into();
        assert_eq!(
            cost_of(&expr),
            EXPR_OVERHEAD + (110 + 20 * chunks(plen, 64)) + (40 + 10 * h),
            "get = expr overhead + createVerifier(proof) + lookup(h)"
        );

        // --- getMany: createVerifier + per-key height-scaled lookups ---
        let mut prover = populate_tree(vec![
            (vec![1u8], 10u64.to_be_bytes().to_vec()),
            (vec![2u8], 20u64.to_be_bytes().to_vec()),
        ]);
        let digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let h = u64::from(*digest.0.last().unwrap());
        for k in [vec![1u8], vec![2u8], vec![3u8]] {
            prover
                .perform_one_operation(&Operation::Lookup(Bytes::from(k)))
                .unwrap();
        }
        let proof_bytes: Vec<u8> = prover.generate_proof().into_iter().collect();
        let plen = proof_bytes.len() as u64;
        let expr: Expr = MethodCall::new(
            tree_const(&digest, AvlTreeFlags::new(false, false, false)),
            savltree::GET_MANY_METHOD.clone(),
            vec![
                keys_const(&[vec![1u8], vec![2u8], vec![3u8]]).into(),
                Constant::from(proof_bytes).into(),
            ],
        )
        .unwrap()
        .into();
        assert_eq!(
            cost_of(&expr),
            EXPR_OVERHEAD + (110 + 20 * chunks(plen, 64)) + 3 * (40 + 10 * h),
            "getMany = expr overhead + createVerifier(proof) + 3 lookups"
        );

        // --- remove: flag check + createVerifier + per-key remove + digest
        //     read + updateDigest (remove is the one mutation that also
        //     charges the unconditional digest read, mirroring Scala) ---
        let mut prover = populate_tree(vec![(vec![1u8], 10u64.to_be_bytes().to_vec())]);
        let digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let h = u64::from(*digest.0.last().unwrap());
        prover
            .perform_one_operation(&Operation::Remove(Bytes::from(vec![1u8])))
            .unwrap();
        let proof_bytes: Vec<u8> = prover.generate_proof().into_iter().collect();
        let plen = proof_bytes.len() as u64;
        let expr: Expr = MethodCall::new(
            tree_const(&digest, AvlTreeFlags::new(false, false, true)),
            savltree::REMOVE_METHOD.clone(),
            vec![
                keys_const(&[vec![1u8]]).into(),
                Constant::from(proof_bytes).into(),
            ],
        )
        .unwrap()
        .into();
        assert_eq!(
            cost_of(&expr),
            EXPR_OVERHEAD + 15 + (110 + 20 * chunks(plen, 64)) + (100 + 15 * h.max(1)) + 15 + 40,
            "remove = expr overhead + isRemoveAllowed + createVerifier(proof) \
             + remove(max(h,1)) + digest + updateDigest"
        );

        // --- insert on an empty tree: max(h, 1) keeps per-entry cost nonzero ---
        let prover = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        let digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let h = u64::from(*digest.0.last().unwrap());
        assert_eq!(h, 0, "fresh tree digest height byte");
        let mut prover = prover;
        prover
            .perform_one_operation(&Operation::Insert(KeyValue {
                key: Bytes::from(vec![1u8]),
                value: Bytes::from(10u64.to_be_bytes().to_vec()),
            }))
            .unwrap();
        let proof_bytes: Vec<u8> = prover.generate_proof().into_iter().collect();
        let plen = proof_bytes.len() as u64;
        let entries = Constant {
            tpe: SType::SColl(Arc::new(SType::STuple(STuple::pair(
                SType::SColl(Arc::new(SType::SByte)),
                SType::SColl(Arc::new(SType::SByte)),
            )))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: Arc::new([Literal::Tup(mk_pair(1u8, 10u64).into())]),
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Arc::new(SType::SByte)),
                    SType::SColl(Arc::new(SType::SByte)),
                )),
            }),
        };
        let insert_expr = |flags: AvlTreeFlags, proof: Vec<u8>| -> Expr {
            MethodCall::new(
                tree_const(&digest, flags),
                savltree::INSERT_METHOD.clone(),
                vec![entries.clone().into(), Constant::from(proof).into()],
            )
            .unwrap()
            .into()
        };
        assert_eq!(
            cost_of(&insert_expr(
                AvlTreeFlags::new(true, false, false),
                proof_bytes.clone()
            )),
            EXPR_OVERHEAD + 15 + (110 + 20 * chunks(plen, 64)) + (40 + 10 * h.max(1)) + 40,
            "insert = expr overhead + isInsertAllowed + createVerifier(proof) \
             + insert(max(h,1)) + updateDigest"
        );

        // --- disallowed mutation: only the flag check is charged ---
        assert_eq!(
            cost_of(&insert_expr(
                AvlTreeFlags::new(false, false, false),
                proof_bytes
            )),
            EXPR_OVERHEAD + 15,
            "insert on a non-insert-allowed tree charges only the flag check"
        );
    }

    /// JVM parity: `BatchAVLVerifier.rootNodeHeight` is assigned from the
    /// digest's trailing byte only after the `keyLength > 0` require; a
    /// non-positive keyLength (signed `Int` on the JVM) fails reconstruction
    /// before the assignment, so the op-cost height is 0
    /// (`CAvlTreeVerifier.treeHeight`).
    #[test]
    fn tree_height_zero_when_reconstruction_cannot_start() {
        let mut digest_bytes = [0u8; 33];
        digest_bytes[32] = 4;
        let tree = |key_length: u32| AvlTreeData {
            digest: ADDigest::try_from(digest_bytes.to_vec()).unwrap(),
            tree_flags: AvlTreeFlags::new(false, false, false),
            key_length,
            value_length_opt: None,
        };
        // a valid-shaped tree keeps its digest-derived height
        assert_eq!(tree_height(&tree(32)), 4);
        // keyLength 0 or negative-on-the-JVM (high bit set) => height 0
        assert_eq!(tree_height(&tree(0)), 0);
        assert_eq!(tree_height(&tree(u32::MAX)), 0); // JVM Int -1
        assert_eq!(tree_height(&tree(0x8000_0000)), 0); // JVM Int.MinValue
    }
}
