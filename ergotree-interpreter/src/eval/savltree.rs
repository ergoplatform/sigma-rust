use std::convert::TryFrom;

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
use sigma_util::AsVecI8;

pub(crate) static DIGEST_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(
        avl_tree_data.digest.into(),
    ))))
};

pub(crate) static ENABLED_OPERATIONS_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Byte(avl_tree_data.tree_flags.serialize() as i8))
};

pub(crate) static KEY_LENGTH_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Int(avl_tree_data.key_length as i32))
};

pub(crate) static VALUE_LENGTH_OPT_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Opt(Box::new(
        avl_tree_data
            .value_length_opt
            .map(|v| Value::Int(*v as i32)),
    )))
};

pub(crate) static IS_INSERT_ALLOWED_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.insert_allowed()))
};

pub(crate) static IS_UPDATE_ALLOWED_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.update_allowed()))
};

pub(crate) static IS_REMOVE_ALLOWED_EVAL_FN: EvalFn = |_env, _ctx, obj, _args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    Ok(Value::Boolean(avl_tree_data.tree_flags.remove_allowed()))
};

pub(crate) static UPDATE_OPERATIONS_EVAL_FN: EvalFn = |_env, _ctx, obj, args| {
    let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    let new_operations = {
        let v = args.get(0).cloned().ok_or_else(|| {
            EvalError::AvlTree("eval is missing first arg (new_operations)".to_string())
        })?;
        v.try_extract_into::<i8>()? as u8
    };
    avl_tree_data.tree_flags = AvlTreeFlags::parse(new_operations);
    Ok(Value::AvlTree(Box::new(avl_tree_data)))
};

pub(crate) static UPDATE_DIGEST_EVAL_FN: EvalFn = |_env, _ctx, obj, args| {
    let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    let new_digest = {
        let v = args.get(0).cloned().ok_or_else(|| {
            EvalError::AvlTree("eval is missing first arg (new_digest)".to_string())
        })?;
        let bytes_vec = v.try_extract_into::<Vec<u8>>()?;
        ADDigest::try_from(bytes_vec).map_err(map_eval_err)?
    };
    avl_tree_data.digest = new_digest;
    Ok(Value::AvlTree(Box::new(avl_tree_data)))
};

pub(crate) static GET_EVAL_FN: EvalFn =
    |_env, _ctx, obj, args| {
        let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        let key = {
            let v = args.get(0).cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing first arg (entries)".to_string())
            })?;
            v.try_extract_into::<Vec<u8>>()?
        };
        let proof = {
            let v = args.get(1).cloned().ok_or_else(|| {
                EvalError::AvlTree("eval is missing second arg (proof)".to_string())
            })?;
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

        match bv.perform_one_operation(&Operation::Lookup(Bytes::from(key))) {
            Ok(opt) => match opt {
                Some(v) => Ok(Value::Opt(Box::new(Some(Value::Coll(
                    CollKind::NativeColl(NativeColl::CollByte(v.to_vec().as_vec_i8())),
                ))))),
                _ => Ok(Value::Opt(Box::new(None))),
            },
            Err(_) => Err(EvalError::AvlTree(format!(
                "Tree proof is incorrect {:?}",
                avl_tree_data
            ))),
        }
    };

pub(crate) static GET_MANY_EVAL_FN: EvalFn =
    |_env, _ctx, obj, args| {
        let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        let keys = {
            let v = args.get(0).cloned().ok_or_else(|| {
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

        let mut res = vec![];
        for key in keys {
            if let Ok(r) = bv.perform_one_operation(&Operation::Lookup(Bytes::from(key))) {
                if let Some(v) = r {
                    res.push(Value::Opt(Box::new(Some(Value::Coll(
                        CollKind::NativeColl(NativeColl::CollByte(v.to_vec().as_vec_i8())),
                    )))))
                } else {
                    res.push(Value::Opt(Box::new(None)))
                }
            } else {
                return Err(EvalError::AvlTree(format!(
                    "Tree proof is incorrect {:?}",
                    avl_tree_data
                )));
            }
        }

        Ok(Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SOption(Box::new(SType::SColl(Box::new(SType::SByte)))),
            items: res,
        }))
    };

pub(crate) static INSERT_EVAL_FN: EvalFn =
    |_env, _ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        if !avl_tree_data.tree_flags.insert_allowed() {
            return Err(EvalError::AvlTree("Insertions not allowed".into()));
        }

        let entries = {
            let v = args.get(0).cloned().ok_or_else(|| {
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
                .perform_one_operation(&Operation::Insert(KeyValue {
                    key: key.into(),
                    value: value.into(),
                }))
                .is_err()
            {
                return Err(EvalError::AvlTree(format!(
                    "Incorrect insert for {:?}",
                    avl_tree_data
                )));
            }
        }
        if let Some(new_digest) = bv.digest() {
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Ok(Value::Opt(Box::new(Some(Value::AvlTree(
                avl_tree_data.into(),
            )))))
        } else {
            Err(EvalError::AvlTree("Cannot update digest".into()))
        }
    };

pub(crate) static REMOVE_EVAL_FN: EvalFn =
    |_env, _ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        if !avl_tree_data.tree_flags.remove_allowed() {
            return Err(EvalError::AvlTree("Removals not allowed".into()));
        }

        let keys = {
            let v = args.get(0).cloned().ok_or_else(|| {
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
        for key in keys {
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
        if let Some(new_digest) = bv.digest() {
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Ok(Value::Opt(Box::new(Some(Value::AvlTree(
                avl_tree_data.into(),
            )))))
        } else {
            Err(EvalError::AvlTree("Cannot update digest".into()))
        }
    };

pub(crate) static CONTAINS_EVAL_FN: EvalFn = |_env, _ctx, obj, args| {
    let avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;
    let key = {
        let v = args
            .get(0)
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

    match bv.perform_one_operation(&Operation::Lookup(key)) {
        Ok(s) => match s {
            Some(_e) => Ok(Value::Boolean(true)),
            _ => Ok(Value::Boolean(false)),
        },
        Err(_) => Err(EvalError::AvlTree(format!(
            "Incorrect contains call for {:?}",
            avl_tree_data
        ))),
    }
};

pub(crate) static UPDATE_EVAL_FN: EvalFn =
    |_env, _ctx, obj, args| {
        let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

        if !avl_tree_data.tree_flags.update_allowed() {
            return Err(EvalError::AvlTree("Updates not allowed".into()));
        }

        let entries = {
            let v = args.get(0).cloned().ok_or_else(|| {
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
                .perform_one_operation(&Operation::Update(KeyValue {
                    key: key.into(),
                    value: value.into(),
                }))
                .is_err()
            {
                return Err(EvalError::AvlTree(format!(
                    "Incorrect update for {:?}",
                    avl_tree_data
                )));
            }
        }
        if let Some(new_digest) = bv.digest() {
            let digest = ADDigest::scorex_parse_bytes(&new_digest)?;
            avl_tree_data.digest = digest;
            Ok(Value::Opt(Box::new(Some(Value::AvlTree(
                avl_tree_data.into(),
            )))))
        } else {
            Err(EvalError::AvlTree("Cannot update digest".into()))
        }
    };

fn map_eval_err<T: std::fmt::Debug>(e: T) -> EvalError {
    EvalError::AvlTree(format!("{:?}", e))
}

#[allow(clippy::unwrap_used, clippy::panic, clippy::unreachable)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::convert::TryFrom;

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

    use crate::eval::tests::eval_out_wo_ctx;

    use super::*;
    use sigma_util::AsVecU8;

    #[test]
    fn eval_avl_get() {
        let mut prover = populate_tree(vec![(vec![1u8], 10u64.to_be_bytes().to_vec())]);
        let initial_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();

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
            if let Some(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b)))) = *opt {
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

        let initial_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();

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

        let search_key_1 = Literal::try_from(vec![1u8]).unwrap();
        let search_key_2 = Literal::try_from(vec![2u8]).unwrap();
        let search_key_3 = Literal::try_from(vec![3u8]).unwrap();

        let keys = Constant {
            tpe: SType::SColl(Box::new(SType::SColl(Box::new(SType::SByte)))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: vec![search_key_1, search_key_2, search_key_3],
                elem_tpe: SType::SColl(Box::new(SType::SByte)),
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
                    match *opt {
                        None => assert!(expected.is_none()),
                        Some(Value::Coll(CollKind::NativeColl(NativeColl::CollByte(b)))) => {
                            assert_eq!(b, expected.unwrap().to_vec().as_vec_i8());
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
        let initial_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();
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
        let final_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();
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
            tpe: SType::SColl(Box::new(SType::STuple(STuple::pair(
                SType::SColl(Box::new(SType::SByte)),
                SType::SColl(Box::new(SType::SByte)),
            )))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: vec![pair1, pair2, pair3],
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Box::new(SType::SByte)),
                    SType::SColl(Box::new(SType::SByte)),
                )),
            }),
        };
        let expr: Expr = MethodCall::new(
            obj,
            savltree::INSERT_METHOD.clone(),
            vec![entries.into(), proof.into()],
        )
        .unwrap()
        .into();

        let res = eval_out_wo_ctx::<Value>(&expr);
        if let Value::Opt(opt) = res {
            if let Some(Value::AvlTree(avl)) = *opt {
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
                assert_eq!(b, digest);
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
                assert_eq!(*opt, value_length_opt);
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
        let digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();

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
        let initial_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();

        let key1 = Bytes::from(vec![1u8]);
        let op1 = Operation::Remove(key1);
        prover.perform_one_operation(&op1).unwrap();
        let final_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();
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

        let key1 = Literal::try_from(vec![1u8]).unwrap();
        let keys = Constant {
            tpe: SType::SColl(Box::new(SType::SColl(Box::new(SType::SByte)))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: vec![key1],
                elem_tpe: SType::SColl(Box::new(SType::SByte)),
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
            if let Some(Value::AvlTree(avl)) = *opt {
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
        let initial_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();

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

        let final_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();
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
            tpe: SType::SColl(Box::new(SType::STuple(STuple::pair(
                SType::SColl(Box::new(SType::SByte)),
                SType::SColl(Box::new(SType::SByte)),
            )))),
            v: Literal::Coll(CollKind::WrappedColl {
                items: vec![pair1, pair2],
                elem_tpe: SType::STuple(STuple::pair(
                    SType::SColl(Box::new(SType::SByte)),
                    SType::SColl(Box::new(SType::SByte)),
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
            if let Some(Value::AvlTree(avl)) = *opt {
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
            Literal::try_from(vec![x]).unwrap(),
            Literal::try_from(y.to_be_bytes().to_vec()).unwrap(),
        ]
    }
}
