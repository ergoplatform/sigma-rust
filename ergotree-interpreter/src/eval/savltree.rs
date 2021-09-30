use bytes::Bytes;
use ergotree_ir::mir::avl_tree_data::ADDigest;
use ergotree_ir::mir::avl_tree_data::AvlTreeData;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::types::stuple::STuple;
use ergotree_ir::types::stype::SType;
use scorex_crypto_avltree::authenticated_tree_ops::AuthenticatedTreeOps;
use scorex_crypto_avltree::batch_avl_verifier::BatchAVLVerifier;
use scorex_crypto_avltree::batch_node::AVLTree;
use scorex_crypto_avltree::batch_node::Node;
use scorex_crypto_avltree::batch_node::NodeHeader;
use scorex_crypto_avltree::operation::KeyValue;
use scorex_crypto_avltree::operation::Operation;

use super::EvalError;
use super::EvalFn;

pub(crate) static INSERT_EVAL_FN: EvalFn = |_env, _ctx, obj, args| {
    let mut avl_tree_data = obj.try_extract_into::<AvlTreeData>()?;

    if !avl_tree_data.tree_flags.insert_allowed() {
        return Err(EvalError::Misc("AvlTree: Insertions not allowed".into()));
    }

    let entries = {
        let v = args
            .get(0)
            .cloned()
            .ok_or_else(|| EvalError::NotFound("AvlTree: eval is missing first arg".to_string()))?;
        match v {
            Value::Coll(CollKind::WrappedColl { elem_tpe, items }) => {
                if elem_tpe
                    == SType::STuple(STuple::pair(
                        SType::SColl(Box::new(SType::SByte)),
                        SType::SColl(Box::new(SType::SByte)),
                    ))
                {
                    let mut tup_items = Vec::with_capacity(items.len());
                    for i in items {
                        match i {
                            Value::Tup(tup) => {
                                if tup.len() == 2 {
                                    let first = Bytes::from(
                                        tup.first().clone().try_extract_into::<Vec<u8>>()?,
                                    );
                                    let second = Bytes::from(
                                        tup.last().clone().try_extract_into::<Vec<u8>>()?,
                                    );
                                    tup_items.push((first, second));
                                } else {
                                    return Err(EvalError::InvalidResultType);
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    tup_items
                } else {
                    return Err(EvalError::InvalidResultType);
                }
            }
            _ => return Err(EvalError::InvalidResultType),
        }
    };

    let proof = {
        let v = args.get(1).cloned().ok_or_else(|| {
            EvalError::NotFound("AvlTree: eval is missing second arg".to_string())
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
            .perform_one_operation(&Operation::Insert(KeyValue { key, value }))
            .is_err()
        {
            return Err(EvalError::Misc(format!(
                "AvlTree: Incorrect insert for {:?}",
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
        Err(EvalError::Misc("AvlTree: Cannot update digest".into()))
    }
};

fn map_eval_err<T: std::fmt::Debug>(e: T) -> EvalError {
    EvalError::Misc(format!("{:?}", e))
}

#[allow(clippy::unwrap_used)]
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
        },
        types::savltree,
    };
    use scorex_crypto_avltree::batch_avl_prover::BatchAVLProver;

    use crate::eval::tests::eval_out_wo_ctx;

    use super::*;
    #[test]
    fn eval_avl_tree() {
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

    fn mk_pair(x: u8, y: u64) -> [Literal; 2] {
        [
            Literal::try_from(vec![x]).unwrap(),
            Literal::try_from(y.to_be_bytes().to_vec()).unwrap(),
        ]
    }
}
