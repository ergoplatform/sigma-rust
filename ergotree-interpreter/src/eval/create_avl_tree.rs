use super::Evaluable;
use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use ergotree_ir::mir::create_avl_tree::CreateAvlTree;
use ergotree_ir::mir::value::Value;

impl Evaluable for CreateAvlTree {
    fn eval<'ctx>(
        &self,
        _env: &mut Env<'ctx>,
        _ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        // Matches JVM `sigma.ast.CreateAvlTree`: does not override `Value.eval`
        // (evaluating opcode 0xB6 fails with unsupported operation).
        Err(EvalError::NotFound(
            "CreateAvlTree evaluation is not supported".into(),
        ))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::try_eval_out_wo_ctx;
    use ergo_chain_types::ADDigest;
    use ergotree_ir::mir::{
        avl_tree_data::{AvlTreeData, AvlTreeFlags},
        constant::Constant,
        expr::Expr,
    };

    #[test]
    fn eval_create_avl_tree_is_unsupported() {
        let flags = AvlTreeFlags::new(false, false, false);
        let expr: Expr = CreateAvlTree::new(
            Expr::Const(flags.into()),
            Expr::Const(ADDigest::zero().into()),
            1.into(),
            Constant::from(Option::<i32>::None).into(),
        )
        .unwrap()
        .into();

        assert!(try_eval_out_wo_ctx::<AvlTreeData>(&expr).is_err());
    }
}
