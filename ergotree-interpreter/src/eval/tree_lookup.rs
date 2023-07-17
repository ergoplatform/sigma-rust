use bytes::Bytes;
use ergotree_ir::mir::tree_lookup::TreeLookup;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::mir::avl_tree_data::AvlTreeData;
use ergotree_ir::mir::constant::TryExtractInto;
use scorex_crypto_avltree::batch_avl_verifier::BatchAVLVerifier;
use scorex_crypto_avltree::batch_node::{AVLTree, Node, NodeHeader};
use scorex_crypto_avltree::operation::Operation;
use sigma_util::AsVecU8;

impl Evaluable for TreeLookup {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let normalized_tree_val = self
            .tree
            .eval(env, ctx)?
            .try_extract_into::<AvlTreeData>()?;
        let normalized_key_val = self.key.eval(env, ctx)?.try_extract_into::<Vec<i8>>()?;
        let normalized_proof_val = self.proof.eval(env, ctx)?.try_extract_into::<Vec<i8>>()?;

        let starting_digest = Bytes::from(normalized_tree_val.digest.0.to_vec());
        let proof = Bytes::from(normalized_proof_val.as_vec_u8());

        let mut bv = BatchAVLVerifier::new(
            &starting_digest,
            &proof,
            AVLTree::new(
                |digest| Node::LabelOnly(NodeHeader::new(Some(*digest), None)),
                normalized_tree_val.key_length as usize,
                normalized_tree_val
                    .value_length_opt
                    .as_ref()
                    .map(|v| **v as usize),
            ),
            None,
            None,
        )
        .map_err(map_eval_err)?;

        match bv.perform_one_operation(&Operation::Lookup(Bytes::from(
            normalized_key_val.as_vec_u8(),
        ))) {
            Ok(opt) => match opt {
                Some(v) => Ok(Value::Opt(Box::new(Some(v.to_vec().into())))),
                _ => Ok(Value::Opt(Box::new(None))),
            },
            Err(_) => Err(EvalError::AvlTree(format!(
                "Tree proof is incorrect {:?}",
                normalized_tree_val
            ))),
        }
    }
}

fn map_eval_err<T: std::fmt::Debug>(e: T) -> EvalError {
    EvalError::AvlTree(format!("{:?}", e))
}

#[allow(clippy::unwrap_used, clippy::panic, clippy::unreachable)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;

    use ergo_chain_types::ADDigest;
    use ergotree_ir::mir::{
        avl_tree_data::{AvlTreeData, AvlTreeFlags},
        expr::Expr,
        value::{CollKind, NativeColl},
    };
    use scorex_crypto_avltree::authenticated_tree_ops::AuthenticatedTreeOps;
    use scorex_crypto_avltree::batch_avl_prover::BatchAVLProver;
    use scorex_crypto_avltree::operation::KeyValue;
    use sigma_ser::ScorexSerializable;
    use sigma_util::AsVecI8;

    #[test]
    fn eval_tree_lookup() {
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
        let proof = prover.generate_proof().to_vec().as_vec_i8();

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

        let expr_found = TreeLookup {
            tree: Box::new(obj.clone()),
            key: Box::new(search_key_found.into()),
            proof: Box::new(proof.clone().into()),
        }
        .into();
        let expr_not_found = TreeLookup {
            tree: Box::new(obj),
            key: Box::new(search_key_not_found.into()),
            proof: Box::new(proof.into()),
        }
        .into();

        let res_found: Value = eval_out_wo_ctx(&expr_found);
        let res_not_found: Value = eval_out_wo_ctx(&expr_not_found);

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
}
