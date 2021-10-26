use super::Evaluable;
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use ergotree_ir::chain::digest32::ADDigest;
use ergotree_ir::mir::avl_tree::CreateAvlTree;
use ergotree_ir::mir::avl_tree_data::{AvlTreeData, AvlTreeFlags};
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use ergotree_ir::util::AsVecU8;
use std::convert::TryFrom;

impl Evaluable for CreateAvlTree {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let flags_v = self.flags.eval(env, ctx)?.try_extract_into::<i8>()? as u8;
        let digest_v = self.digest.eval(env, ctx)?.try_extract_into::<Vec<i8>>()?;
        let key_length = self.key_length.eval(env, ctx)?.try_extract_into::<i32>()? as u32;
        let value_length_opt = match self.value_length.clone() {
            Some(expr) => Some(Box::new(
                expr.eval(env, ctx)?.try_extract_into::<i32>()? as u32
            )),
            None => None,
        };

        let tree_flags = AvlTreeFlags::parse(flags_v);
        let digest = ADDigest::try_from(digest_v.as_vec_u8()).map_err(map_eval_err)?;

        Ok(Value::AvlTree(Box::new(AvlTreeData {
            tree_flags,
            digest,
            key_length,
            value_length_opt,
        })))
    }
}

fn map_eval_err<T: std::fmt::Debug>(e: T) -> EvalError {
    EvalError::AvlTree(format!("{:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;

    use ergotree_ir::chain::digest32::ADDigest;
    use ergotree_ir::mir::{
        avl_tree_data::{AvlTreeData, AvlTreeFlags},
        expr::Expr,
    };
    use ergotree_ir::serialization::SigmaSerializable;
    use scorex_crypto_avltree::authenticated_tree_ops::AuthenticatedTreeOps;
    use scorex_crypto_avltree::batch_avl_prover::BatchAVLProver;
    use scorex_crypto_avltree::batch_node::{AVLTree, Node, NodeHeader};

    #[test]
    fn eval_create_avl_tree() {
        let prover = BatchAVLProver::new(
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
        let flags = AvlTreeFlags::new(false, false, false);
        let expr: Expr = CreateAvlTree::new(
            Expr::Const(flags.clone().into()),
            Expr::Const(initial_digest.clone().into()),
            1.into(),
            None.into(),
        )
        .unwrap()
        .into();

        let tree = eval_out_wo_ctx::<AvlTreeData>(&expr);
        assert_eq!(tree.digest, initial_digest);
        assert_eq!(tree.key_length, 1);
        assert_eq!(tree.value_length_opt, None);
        assert_eq!(tree.tree_flags, flags);
    }
}
