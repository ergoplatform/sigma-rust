use super::Evaluable;
use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use ergo_chain_types::ADDigest;
use ergotree_ir::mir::avl_tree_data::{AvlTreeData, AvlTreeFlags};
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::create_avl_tree::CreateAvlTree;
use ergotree_ir::mir::value::Value;
use sigma_util::AsVecU8;
use std::convert::TryFrom;

impl Evaluable for CreateAvlTree {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
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

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::tests::eval_out_wo_ctx;

    use ergo_chain_types::ADDigest;
    use ergotree_ir::mir::{
        avl_tree_data::{AvlTreeData, AvlTreeFlags},
        expr::Expr,
    };
    use ergo_avltree_rust::authenticated_tree_ops::AuthenticatedTreeOps;
    use ergo_avltree_rust::batch_avl_prover::BatchAVLProver;
    use ergo_avltree_rust::batch_node::{AVLTree, Node, NodeHeader};
    use sigma_ser::ScorexSerializable;

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
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();
        let flags = AvlTreeFlags::new(false, false, false);
        let expr: Expr = CreateAvlTree::new(
            Expr::Const(flags.clone().into()),
            Expr::Const(initial_digest.into()),
            1.into(),
            None,
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
