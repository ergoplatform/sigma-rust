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

use super::EvalError;
use super::EvalFn;
use ergotree_ir::types::stype::SType;

pub(crate) static DIGEST_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
        avl_tree_data.digest.0.iter().map(|&b| b as i8).collect(),
    ))))
};

pub(crate) static ENABLED_OPERATIONS_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Byte(avl_tree_data.tree_flags.serialize() as i8))
};

pub(crate) static KEY_LENGTH_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Int(avl_tree_data.key_length as i32))
};

pub(crate) static VALUE_LENGTH_OPT_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Opt(
        avl_tree_data
            .value_length_opt
            .map(|v| Value::Int(*v as i32))
            .map(Box::new),
    ))
};

pub(crate) static IS_INSERT_ALLOWED_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.insert_allowed()))
};

pub(crate) static IS_UPDATE_ALLOWED_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.update_allowed()))
};

pub(crate) static IS_REMOVE_ALLOWED_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.remove_allowed()))
};

pub(crate) static UPDATE_OPERATIONS_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
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

pub(crate) static UPDATE_DIGEST_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
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

pub(crate) static GET_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
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

    let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
    let mut bv = match BatchAVLVerifier::new(
        &starting_digest,
        &proof,
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
    ) {
        Ok(bv) => bv,
        // The reference impl's verifier construction never throws: a proof that
        // does not match the tree digest yields a verifier with no reconstructed
        // tree, the lookup fails, and `get_eval` raises the same "Tree proof is
        // incorrect" error as an op-level lookup failure
        // (`CErgoTreeEvaluator.get_eval`).
        Err(_) => {
            return Err(EvalError::AvlTree(format!(
                "Tree proof is incorrect {:?}",
                avl_tree_data
            )))
        }
    };

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
    |_mc, _env, _ctx, obj, args| {
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

        let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
        let mut bv = match BatchAVLVerifier::new(
            &starting_digest,
            &proof,
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
        ) {
            Ok(bv) => bv,
            // The reference impl's verifier construction never throws: lookups
            // fail on the no-tree verifier, so the first key raises the same
            // "Tree proof is incorrect" error as an op-level lookup failure —
            // and with no keys no lookup runs at all, leaving the empty
            // collection (`CErgoTreeEvaluator.getMany_eval`).
            Err(_) => {
                return if keys.is_empty() {
                    Ok(Value::Coll(CollKind::WrappedColl {
                        elem_tpe: SType::SOption(Arc::new(SType::SColl(Arc::new(SType::SByte)))),
                        items: Arc::new([]),
                    }))
                } else {
                    Err(EvalError::AvlTree(format!(
                        "Tree proof is incorrect {:?}",
                        avl_tree_data
                    )))
                }
            }
        };

        let res = keys
            .into_iter()
            .map(|key| {
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

        let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
        let mut bv = match BatchAVLVerifier::new(
            &starting_digest,
            &proof,
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
        ) {
            Ok(bv) => bv,
            // The reference impl's verifier construction never throws: every
            // insert fails on the no-tree verifier, which pre-v3 raises at the
            // first failed op and from v3 fast-breaks (issue #908), leaving the
            // None digest — so the method evaluates to None. With no entries no
            // op runs and the digest is still None at every version
            // (`CErgoTreeEvaluator.insert_eval`).
            Err(_) => {
                return if entries.is_empty() || ctx.tree_version() >= ErgoTreeVersion::V3 {
                    Ok(Value::Opt(None))
                } else {
                    Err(EvalError::AvlTree(format!(
                        "Incorrect insert for {:?}",
                        avl_tree_data
                    )))
                }
            }
        };
        for (key, value) in entries {
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
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Value::Opt(Some(Box::new(Value::AvlTree(avl_tree_data.into()))))
        } else {
            Value::Opt(None)
        })
    };

pub(crate) static REMOVE_EVAL_FN: EvalFn =
    |_mc, _env, _ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

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

        let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
        let mut bv = match BatchAVLVerifier::new(
            &starting_digest,
            &proof,
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
        ) {
            Ok(bv) => bv,
            // The reference impl's verifier construction never throws: every
            // remove fails on the no-tree verifier (op results are ignored —
            // `remove_eval` loops with `cfor`, no break), and the final digest
            // is None — so the method evaluates to None
            // (`CErgoTreeEvaluator.remove_eval`).
            Err(_) => return Ok(Value::Opt(None)),
        };
        for key in keys {
            // op results are ignored — the reference impl loops with `cfor`,
            // no break, no check (`CErgoTreeEvaluator.remove_eval`); a failed
            // op invalidates the verifier, so the digest below decides the
            // outcome
            let _ = bv.perform_one_operation(&Operation::Remove(Bytes::from(key)));
        }
        if let Some(new_digest) = bv.digest() {
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Ok(Value::Opt(Some(Box::new(Value::AvlTree(
                avl_tree_data.into(),
            )))))
        } else {
            Ok(Value::Opt(None))
        }
    };

pub(crate) static CONTAINS_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
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

    let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
    let mut bv = match BatchAVLVerifier::new(
        &starting_digest,
        &proof,
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
    ) {
        Ok(bv) => bv,
        // The reference impl's verifier construction never throws: the lookup
        // fails on the no-tree verifier and `contains_eval` maps the failure to
        // false (`CErgoTreeEvaluator.contains_eval`).
        Err(_) => return Ok(Value::Boolean(false)),
    };

    Ok(match bv.perform_one_operation(&Operation::Lookup(key)) {
        Ok(s) => match s {
            Some(_e) => Value::Boolean(true),
            _ => Value::Boolean(false),
        },
        Err(_) => Value::Boolean(false),
    })
};

pub(crate) static UPDATE_EVAL_FN: EvalFn =
    |_mc, _env, _ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

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

        let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
        let mut bv = match BatchAVLVerifier::new(
            &starting_digest,
            &proof,
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
        ) {
            Ok(bv) => bv,
            // The reference impl's verifier construction never throws: every
            // update fails on the no-tree verifier (`update_eval`'s forall
            // fast-breaks at the first failure), and the final digest is None —
            // so the method evaluates to None
            // (`CErgoTreeEvaluator.update_eval`).
            Err(_) => return Ok(Value::Opt(None)),
        };
        for (key, value) in entries {
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
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Value::Opt(Some(Value::AvlTree(avl_tree_data.into()).into()))
        } else {
            Value::Opt(None)
        })
    };

pub(crate) static INSERT_OR_UPDATE_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, args| {
    let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

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

    let starting_digest = Bytes::from(avl_tree_data.digest.0.to_vec());
    let mut bv = BatchAVLVerifier::new(
        &starting_digest,
        &proof,
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
    .map_err(map_eval_err)?;
    for (key, value) in entries {
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
        chain::context::Context,
        ergo_tree::ErgoTree,
        mir::{
            avl_tree_data::{AvlTreeData, AvlTreeFlags},
            constant::{Constant, Literal},
            expr::Expr,
            method_call::MethodCall,
            value::CollKind,
        },
        serialization::SigmaSerializable,
        types::{savltree, stuple::STuple, stype::SType},
    };
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out_with_version, try_eval_out_wo_ctx};

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

    /// A structurally-valid proof generated from a DIFFERENT tree, paired with
    /// the digest of a non-empty tree: the construction-level digest mismatch
    /// of the wrong-tree-proof family. The reference impl's verifier
    /// construction never throws — every op fails on the no-tree verifier and
    /// each method maps that per its own semantics (santa-eval
    /// `AvlTree.wrong_tree_proof`).
    fn wrong_tree_proof_fixture(tree_flags: AvlTreeFlags) -> (Expr, Constant) {
        // tree A: non-empty (one committed insert), so its digest cannot match
        // the empty-tree digest the wrong proof starts from
        let mut prover_a = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        prover_a
            .perform_one_operation(&Operation::Insert(KeyValue {
                key: Bytes::from(vec![5u8]),
                value: Bytes::from(50u64.to_be_bytes().to_vec()),
            }))
            .unwrap();
        prover_a.generate_proof();
        let tree_a_digest = ADDigest::scorex_parse_bytes(&prover_a.digest().unwrap()).unwrap();

        // proof from tree B (empty start) inserting key 1
        let mut prover_b = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        prover_b
            .perform_one_operation(&Operation::Insert(KeyValue {
                key: Bytes::from(vec![1u8]),
                value: Bytes::from(10u64.to_be_bytes().to_vec()),
            }))
            .unwrap();
        let wrong_proof: Constant = prover_b
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let obj = Expr::Const(
            AvlTreeData {
                digest: tree_a_digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );
        (obj, wrong_proof)
    }

    fn pairs_coll(items: Arc<[Literal]>) -> Constant {
        Constant {
            tpe: SType::SColl(Arc::new(SType::STuple(STuple::pair(
                SType::SColl(Arc::new(SType::SByte)),
                SType::SColl(Arc::new(SType::SByte)),
            )))),
            v: Literal::Coll(CollKind::WrappedColl {
                items,
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Arc::new(SType::SByte)),
                    SType::SColl(Arc::new(SType::SByte)),
                )),
            }),
        }
    }

    fn keys_coll(items: Arc<[Literal]>) -> Constant {
        Constant {
            tpe: SType::SColl(Arc::new(SType::SColl(Arc::new(SType::SByte)))),
            v: Literal::Coll(CollKind::WrappedColl {
                items,
                elem_tpe: SType::SColl(Arc::new(SType::SByte)),
            }),
        }
    }

    #[test]
    fn eval_avl_contains_bad_proof() {
        let (obj, wrong_proof) = wrong_tree_proof_fixture(AvlTreeFlags::new(false, false, false));
        let key: Constant = vec![1u8].into();
        let expr: Expr = MethodCall::new(
            obj,
            savltree::CONTAINS_METHOD.clone(),
            vec![key.into(), wrong_proof.into()],
        )
        .unwrap()
        .into();
        // the failed lookup maps to false (`contains_eval`)
        assert!(!eval_out_wo_ctx::<bool>(&expr));
    }

    #[test]
    fn eval_avl_get_bad_proof() {
        let (obj, wrong_proof) = wrong_tree_proof_fixture(AvlTreeFlags::new(false, false, false));
        let key: Constant = vec![1u8].into();
        let expr: Expr = MethodCall::new(
            obj,
            savltree::GET_METHOD.clone(),
            vec![key.into(), wrong_proof.into()],
        )
        .unwrap()
        .into();
        // the failed lookup raises "Tree proof is incorrect" (`get_eval`)
        assert!(try_eval_out_wo_ctx::<Value<'_>>(&expr).is_err());
    }

    // --- F4 degenerate-input family (ergo_avltree_rust#14) ---
    // Additional construction-failure modes the reference verifier handles
    // gracefully but the crate used to panic on: unparseable/garbage proof bytes
    // and an out-of-range (wrapped-negative) key length. With the crate returning
    // Err instead of panicking (PR #14), they route exactly like the wrong-tree
    // digest mismatch above. Requires that crate fix; validated locally via
    // [patch.crates-io]. (The op-level wrong-value-length mode — santa-eval
    // `AvlTree.per_op_failure` — is covered crate-side in #14 and routes like
    // `eval_avl_insert_bad_proof` once the op returns Err.)

    /// A valid non-empty-tree digest paired with garbage proof bytes: the proof
    /// does not parse at all, a construction failure distinct from the wrong-tree
    /// digest mismatch. santa-eval `AvlTree.bad_proof_bytes`.
    fn bad_proof_bytes_fixture(tree_flags: AvlTreeFlags) -> (Expr, Constant) {
        let mut prover = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        prover
            .perform_one_operation(&Operation::Insert(KeyValue {
                key: Bytes::from(vec![5u8]),
                value: Bytes::from(50u64.to_be_bytes().to_vec()),
            }))
            .unwrap();
        prover.generate_proof();
        let digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let garbage_proof: Constant = vec![0u8].into();
        let obj = Expr::Const(
            AvlTreeData {
                digest,
                tree_flags,
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );
        (obj, garbage_proof)
    }

    /// An AvlTreeData whose key length is the wrapped-negative `0x8000_0000`: the
    /// reference verifier reads keyLength as a signed Int, fails its
    /// `keyLength > 0` check, and runs ops on a no-tree verifier. santa-eval
    /// `AvlTree.negative_keylength_tree`.
    fn negative_keylength_fixture(tree_flags: AvlTreeFlags) -> (Expr, Constant) {
        let mut prover = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        prover
            .perform_one_operation(&Operation::Insert(KeyValue {
                key: Bytes::from(vec![1u8]),
                value: Bytes::from(10u64.to_be_bytes().to_vec()),
            }))
            .unwrap();
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();
        let digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        let obj = Expr::Const(
            AvlTreeData {
                digest,
                tree_flags,
                key_length: 0x8000_0000,
                value_length_opt: None,
            }
            .into(),
        );
        (obj, proof)
    }

    #[test]
    #[ignore = "requires ergo_avltree_rust no-panic fix (ergoplatform/ergo_avltree_rust#14); \
                un-ignore on its release + version bump"]
    fn eval_avl_contains_garbage_proof_bytes() {
        let (obj, proof) = bad_proof_bytes_fixture(AvlTreeFlags::new(false, false, false));
        let key: Constant = vec![1u8].into();
        let expr: Expr = MethodCall::new(
            obj,
            savltree::CONTAINS_METHOD.clone(),
            vec![key.into(), proof.into()],
        )
        .unwrap()
        .into();
        assert!(!eval_out_wo_ctx::<bool>(&expr));
    }

    #[test]
    #[ignore = "requires ergo_avltree_rust no-panic fix (ergoplatform/ergo_avltree_rust#14); \
                un-ignore on its release + version bump"]
    fn eval_avl_get_garbage_proof_bytes() {
        let (obj, proof) = bad_proof_bytes_fixture(AvlTreeFlags::new(false, false, false));
        let key: Constant = vec![1u8].into();
        let expr: Expr = MethodCall::new(
            obj,
            savltree::GET_METHOD.clone(),
            vec![key.into(), proof.into()],
        )
        .unwrap()
        .into();
        assert!(try_eval_out_wo_ctx::<Value<'_>>(&expr).is_err());
    }

    #[test]
    #[ignore = "requires ergo_avltree_rust no-panic fix (ergoplatform/ergo_avltree_rust#14); \
                un-ignore on its release + version bump"]
    fn eval_avl_contains_negative_key_length() {
        let (obj, proof) = negative_keylength_fixture(AvlTreeFlags::new(false, false, false));
        let key: Constant = vec![1u8].into();
        let expr: Expr = MethodCall::new(
            obj,
            savltree::CONTAINS_METHOD.clone(),
            vec![key.into(), proof.into()],
        )
        .unwrap()
        .into();
        assert!(!eval_out_wo_ctx::<bool>(&expr));
    }

    #[test]
    fn eval_avl_get_many_bad_proof() {
        let (obj, wrong_proof) = wrong_tree_proof_fixture(AvlTreeFlags::new(false, false, false));
        let key1 = Literal::from(vec![1u8]);
        let expr: Expr = MethodCall::new(
            obj.clone(),
            savltree::GET_MANY_METHOD.clone(),
            vec![
                keys_coll(Arc::new([key1])).into(),
                wrong_proof.clone().into(),
            ],
        )
        .unwrap()
        .into();
        // the first failed lookup raises "Tree proof is incorrect" (`getMany_eval`)
        assert!(try_eval_out_wo_ctx::<Value<'_>>(&expr).is_err());

        // with no keys no lookup runs at all: the empty collection, not an error
        let expr: Expr = MethodCall::new(
            obj,
            savltree::GET_MANY_METHOD.clone(),
            vec![keys_coll(Arc::new([])).into(), wrong_proof.into()],
        )
        .unwrap()
        .into();
        let res = try_eval_out_wo_ctx::<Value<'_>>(&expr).unwrap();
        if let Value::Coll(CollKind::WrappedColl { items, .. }) = res {
            assert!(items.is_empty());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn eval_avl_insert_bad_proof() {
        let (obj, wrong_proof) = wrong_tree_proof_fixture(AvlTreeFlags::new(true, false, false));
        let pair1 = Literal::Tup(mk_pair(1u8, 10u64).into());
        let expr: Expr = MethodCall::new(
            obj.clone(),
            savltree::INSERT_METHOD.clone(),
            vec![
                pairs_coll(Arc::new([pair1])).into(),
                wrong_proof.clone().into(),
            ],
        )
        .unwrap()
        .into();
        // pre-v3 the first failed insert raises; from v3 it fast-breaks and the
        // None digest makes the method evaluate to None (issue #908)
        assert!(try_eval_out_with_version::<Value<'_>>(&expr, &force_any_val(), 0, 3).is_err());
        assert_eq!(
            try_eval_out_with_version::<Value<'_>>(&expr, &force_any_val(), 3, 3).unwrap(),
            Value::Opt(None)
        );

        // with no entries no op runs and the None digest yields None at every
        // version — no raise even pre-v3
        let expr: Expr = MethodCall::new(
            obj,
            savltree::INSERT_METHOD.clone(),
            vec![pairs_coll(Arc::new([])).into(), wrong_proof.into()],
        )
        .unwrap()
        .into();
        assert_eq!(
            try_eval_out_with_version::<Value<'_>>(&expr, &force_any_val(), 0, 3).unwrap(),
            Value::Opt(None)
        );
    }

    #[test]
    fn eval_avl_update_bad_proof() {
        let (obj, wrong_proof) = wrong_tree_proof_fixture(AvlTreeFlags::new(false, true, false));
        let pair1 = Literal::Tup(mk_pair(1u8, 10u64).into());
        let expr: Expr = MethodCall::new(
            obj,
            savltree::UPDATE_METHOD.clone(),
            vec![pairs_coll(Arc::new([pair1])).into(), wrong_proof.into()],
        )
        .unwrap()
        .into();
        // the failed update fast-breaks and the None digest yields None
        let res = eval_out_wo_ctx::<Value>(&expr);
        assert!(matches!(res, Value::Opt(None)));
    }

    #[test]
    fn eval_avl_remove_bad_proof() {
        let (obj, wrong_proof) = wrong_tree_proof_fixture(AvlTreeFlags::new(false, false, true));
        let key1 = Literal::from(vec![1u8]);
        let expr: Expr = MethodCall::new(
            obj,
            savltree::REMOVE_METHOD.clone(),
            vec![keys_coll(Arc::new([key1])).into(), wrong_proof.into()],
        )
        .unwrap()
        .into();
        // failed removes are ignored and the None digest yields None
        let res = eval_out_wo_ctx::<Value>(&expr);
        assert!(matches!(res, Value::Opt(None)));
    }

    #[test]
    fn eval_avl_remove_mismatched_op() {
        // a VALID proof (construction succeeds: the digest matches) committing
        // remove(1), but the script removes key 9 — the op fails against the
        // proof, invalidating the verifier, and the None digest yields None
        // where the reference impl never raises (`remove_eval` ignores op
        // results — cfor, no break)
        let mut prover = populate_tree(vec![(vec![1u8], 10u64.to_be_bytes().to_vec())]);
        let initial_digest = ADDigest::scorex_parse_bytes(&prover.digest().unwrap()).unwrap();
        prover
            .perform_one_operation(&Operation::Remove(Bytes::from(vec![1u8])))
            .unwrap();
        let proof: Constant = prover
            .generate_proof()
            .into_iter()
            .collect::<Vec<_>>()
            .into();

        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags: AvlTreeFlags::new(false, false, true),
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );
        let key9 = Literal::from(vec![9u8]);
        let expr: Expr = MethodCall::new(
            obj,
            savltree::REMOVE_METHOD.clone(),
            vec![keys_coll(Arc::new([key9])).into(), proof.into()],
        )
        .unwrap()
        .into();
        let res = eval_out_wo_ctx::<Value>(&expr);
        assert!(matches!(res, Value::Opt(None)));
    }

    // JVM-blessed byte vectors (santa-eval `AvlTree.wrong_tree_proof` /
    // `AvlTree.insert_wrong_tree`): a valid proof from a committed n=4 tree
    // against the n=8 digest, carried with the args as segregated constants.
    // The blessed sized header (`1a`/`1b` + size VLQ) is rewritten to the
    // non-sized `12`/`13` (size-bit cleared, size dropped) because the sized
    // parse path rejects non-SigmaProp roots — the same lenient deserialize
    // the conformance runner applies to expression-rooted corpus trees; body
    // bytes verbatim.
    #[test]
    fn eval_avl_contains_bad_proof_blessed_bytes() {
        let tree_bytes = base16::decode("120364fb2b77372d81da43ce2d72714aec79ae5fcac20a9aff426fe6afb476a6fbc02c04072001080e204466b8f1af03542b3c35de30426ec12b04fc34fa0a8c48289ec868e1e00aec550e8f0103a4950597f640451a7f628ea42ce4525890593c75ab337afd6315122093bc6474024466b8f1af03542b3c35de30426ec12b04fc34fa0a8c48289ec868e1e00aec558f23c5ee24a49084f24812ee9dbcdc9e11974b05cad682c21e88fca866bc85ce000000005a17a002ff03b44482bbfc4a39a5f1bd1a35c8699de6c3c51cc71376cea9fe4b85a283801397ff0401dc640973000273017302").unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        let expr = tree.proposition().unwrap();
        let ctx = force_any_val::<Context>();
        let res = try_eval_out_with_version::<Value>(&expr, &ctx, 2, 2).unwrap();
        assert!(matches!(res, Value::Boolean(false)));
    }

    #[test]
    fn eval_avl_update_bad_proof_blessed_bytes() {
        let tree_bytes = base16::decode("120464fb2b77372d81da43ce2d72714aec79ae5fcac20a9aff426fe6afb476a6fbc02c04072001080e200b65ce5d0a76265e6734964bf1a7620da7c4fb2659a9047a7b61869a007fb1110e08000000005a17a04d0eb10103f9859bd9b3050c599dc5b506caa4f2367f286c60a9af489e69d797ab16b9cff5020b65ce5d0a76265e6734964bf1a7620da7c4fb2659a9047a7b61869a007fb1114466b8f1af03542b3c35de30426ec12b04fc34fa0a8c48289ec868e1e00aec55000000005a17a003000335abe8a6e6c7b70addacbec9267be07ec3ac1c5a196b4438261d4a84648739c6ff03b44482bbfc4a39a5f1bd1a35c8699de6c3c51cc71376cea9fe4b85a283801397ff0403dc640d73000283013c0e0e8602730173027303").unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        let expr = tree.proposition().unwrap();
        let ctx = force_any_val::<Context>();
        let res = try_eval_out_with_version::<Value>(&expr, &ctx, 2, 2).unwrap();
        assert!(matches!(res, Value::Opt(None)));
    }

    #[test]
    fn eval_avl_remove_bad_proof_blessed_bytes() {
        let tree_bytes = base16::decode("120364fb2b77372d81da43ce2d72714aec79ae5fcac20a9aff426fe6afb476a6fbc02c04072001080e208f23c5ee24a49084f24812ee9dbcdc9e11974b05cad682c21e88fca866bc85ce0eb90103a4950597f640451a7f628ea42ce4525890593c75ab337afd6315122093bc6474024466b8f1af03542b3c35de30426ec12b04fc34fa0a8c48289ec868e1e00aec558f23c5ee24a49084f24812ee9dbcdc9e11974b05cad682c21e88fca866bc85ce000000005a17a002ff02c4e67333f14133fce4ed7bbf147642b3e7b2b8324268ea6ded1be0caa2da237f000000005a17a000039594489172346c6b22ac52210d14be7fdb02f6ec52b9e525b3a9c77114374cd900ff0402dc640e73000283010e73017302").unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        let expr = tree.proposition().unwrap();
        let ctx = force_any_val::<Context>();
        let res = try_eval_out_with_version::<Value>(&expr, &ctx, 2, 2).unwrap();
        assert!(matches!(res, Value::Opt(None)));
    }

    #[test]
    fn eval_avl_insert_bad_proof_v3_blessed_bytes() {
        let tree_bytes = base16::decode("130464fb2b77372d81da43ce2d72714aec79ae5fcac20a9aff426fe6afb476a6fbc02c04072001080e209a39c57d13039b50bfe4aa21b2c8238be31d133db1fbe4549b16d9b94338b4e20e08000000005a17a0320e8f0103fcbfd9e0c4781263bb161625674719024acef64654799a00c41cc148bdc4a891028f23c5ee24a49084f24812ee9dbcdc9e11974b05cad682c21e88fca866bc85cec4e67333f14133fce4ed7bbf147642b3e7b2b8324268ea6ded1be0caa2da237f000000005a17a000039594489172346c6b22ac52210d14be7fdb02f6ec52b9e525b3a9c77114374cd900ff0402dc640c73000283013c0e0e8602730173027303").unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&tree_bytes).unwrap();
        let expr = tree.proposition().unwrap();
        let ctx = force_any_val::<Context>();
        let res = try_eval_out_with_version::<Value>(&expr, &ctx, 3, 3).unwrap();
        assert!(matches!(res, Value::Opt(None)));
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
}
