use alloc::string::ToString;

use ergotree_ir::mir::value::Value;
use ergotree_ir::mir::zk_proof::ZkProofBlock;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ZkProofBlock {
    fn eval<'ctx>(
        &self,
        _env: &mut Env<'ctx>,
        _ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        // Mirrors Scala's `ZKProofBlock` — `costKind = notSupportedError`,
        // `OpCodes.Undefined`, no serializer. The AST node exists at the typer
        // layer but is rejected by the graph-builder/serializer with
        // `GraphBuildingException`. The interpreter has no evaluation path.
        Err(EvalError::Misc(
            "ZKProof block has no interpreter eval (frontend-only — Scala throws GraphBuildingException at compile)".to_string(),
        ))
    }
}
