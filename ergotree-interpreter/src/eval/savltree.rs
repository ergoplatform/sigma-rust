use std::convert::TryInto;

use bytes::Bytes;
use ergotree_ir::chain::digest32::ADDigest;
use ergotree_ir::mir::avl_tree_data::AvlTreeData;
use ergotree_ir::mir::avl_tree_data::AvlTreeFlags;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::NativeColl;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;
use scorex_crypto_avltree::authenticated_tree_ops::AuthenticatedTreeOps;
use scorex_crypto_avltree::batch_avl_verifier::BatchAVLVerifier;
use scorex_crypto_avltree::batch_node::AVLTree;
use scorex_crypto_avltree::batch_node::Node;
use scorex_crypto_avltree::batch_node::NodeHeader;
use scorex_crypto_avltree::operation::KeyValue;
use scorex_crypto_avltree::operation::Operation;

use super::EvalError;
use super::EvalFn;

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
        let bytes: [u8; 33] = v
            .try_extract_into::<Vec<u8>>()?
            .as_slice()
            .try_into()
            .map_err(map_eval_err)?;
        ADDigest::from(bytes)
    };
    avl_tree_data.digest = new_digest;
    Ok(Value::AvlTree(Box::new(avl_tree_data)))
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
            let digest = ADDigest::sigma_parse_bytes(&new_digest)?;
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

#[allow(clippy::unwrap_used, clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::convert::TryFrom;

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
    use scorex_crypto_avltree::batch_avl_prover::BatchAVLProver;

    use crate::eval::tests::eval_out_wo_ctx;

    use super::*;
    #[test]
    fn eval_avl_insert() {
        // This example taken from `scorex_crypto_avltree` README
        let mut prover = BatchAVLProver::new(
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                1,
                None,
            ),
            true,
        );
        let initial_digest =
            ADDigest::sigma_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
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
            ADDigest::sigma_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
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
        fn eval_avl_properties(v in any::<AvlTreeData>(), new_ops in any::<AvlTreeFlags>(), new_digest in any::<ADDigest>()) {
            let digest: Vec<i8> = v.digest.clone().into();
            let enabled_ops = v.tree_flags.serialize() as i8;
            let key_length = v.key_length as i32;
            let value_length_opt = v.value_length_opt.clone().map(|v| Value::Int(*v as i32));
            let insert_allowed = v.tree_flags.insert_allowed();
            let update_allowed = v.tree_flags.update_allowed();
            let remove_allowed = v.tree_flags.remove_allowed();

            let obj = Expr::Const(v.clone().into());

            // Test digest method
            let expr: Expr = MethodCall::new(
                obj.clone(),
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

            // Test enabledOperations method
            let expr: Expr = MethodCall::new(
                obj.clone(),
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

            // Test keyLength method
            let expr: Expr = MethodCall::new(
                obj.clone(),
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

            // Test valueLengthOpt method
            let expr: Expr = MethodCall::new(
                obj.clone(),
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

            // Test isInsertAllowed method
            let expr: Expr = MethodCall::new(
                obj.clone(),
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

            // Test isUpdateAllowed method
            let expr: Expr = MethodCall::new(
                obj.clone(),
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

            // Test isRemoveAllowed method
            let expr: Expr = MethodCall::new(
                obj.clone(),
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

            // Test updateOperations method
            let obj = Expr::Const(v.clone().into());
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

            // Test updateDigest method
            let obj = Expr::Const(v.into());
            let expr: Expr = MethodCall::new(
                obj,
                savltree::UPDATE_DIGEST_METHOD.clone(),
                vec![Constant::from(new_digest.sigma_serialize_bytes()?).into()],
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

    fn mk_pair(x: u8, y: u64) -> [Literal; 2] {
        [
            Literal::try_from(vec![x]).unwrap(),
            Literal::try_from(y.to_be_bytes().to_vec()).unwrap(),
        ]
    }
}
