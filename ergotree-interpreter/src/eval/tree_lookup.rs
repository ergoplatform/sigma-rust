use alloc::string::ToString;
use ergotree_ir::mir::tree_lookup::TreeLookup;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for TreeLookup {
    fn eval<'ctx>(
        &self,
        _env: &mut Env<'ctx>,
        _ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        // The reference interpreter has no eval override for the standalone
        // `TreeLookup` node (opcode `AvlTreeGet`): its `costKind` is
        // `notSupportedError`, so the default `Value.eval` raises ("Should be
        // overriden"). The ErgoScript compiler emits an `AvlTree.get` MethodCall,
        // never this node, so mainnet never reaches it — but a hand-crafted tree
        // using the opcode directly must error to match the oracle rather than
        // evaluate (it would be a consensus split otherwise).
        Err(EvalError::UnexpectedExpr(
            "TreeLookup (AvlTreeGet) node is not supported for evaluation".to_string(),
        ))
    }
}

#[allow(clippy::unwrap_used, clippy::panic, clippy::unreachable)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::try_eval_out_wo_ctx;

    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;
    use bytes::Bytes;
    use ergo_avltree_rust::authenticated_tree_ops::AuthenticatedTreeOps;
    use ergo_avltree_rust::batch_avl_prover::BatchAVLProver;
    use ergo_avltree_rust::batch_node::{AVLTree, Node, NodeHeader};
    use ergo_avltree_rust::operation::{KeyValue, Operation};
    use ergo_chain_types::ADDigest;
    use ergotree_ir::mir::avl_tree_data::{AvlTreeData, AvlTreeFlags};
    use ergotree_ir::mir::expr::Expr;
    use sigma_ser::ScorexSerializable;
    use sigma_util::AsVecI8;

    #[test]
    fn tree_lookup_eval_is_unsupported() {
        // A well-formed TreeLookup over a real tree and a valid proof: the
        // reference interpreter still refuses to evaluate the standalone node, so
        // eval must error rather than return the looked-up value.
        let mut prover = populate_tree(vec![(vec![1u8], 10u64.to_be_bytes().to_vec())]);
        let initial_digest =
            ADDigest::scorex_parse_bytes(&prover.digest().unwrap().into_iter().collect::<Vec<_>>())
                .unwrap();
        let proof = prover.generate_proof().to_vec().as_vec_i8();

        let obj = Expr::Const(
            AvlTreeData {
                digest: initial_digest,
                tree_flags: AvlTreeFlags::new(false, false, false),
                key_length: 1,
                value_length_opt: None,
            }
            .into(),
        );
        let expr: Expr = TreeLookup {
            tree: Box::new(obj),
            key: Box::new(vec![1i8].into()),
            proof: Box::new(proof.into()),
        }
        .into();

        assert!(try_eval_out_wo_ctx::<Value>(&expr).is_err());
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
