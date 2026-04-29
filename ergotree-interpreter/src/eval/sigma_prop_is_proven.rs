use alloc::string::ToString;

use ergotree_ir::mir::sigma_prop_is_proven::SigmaPropIsProven;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for SigmaPropIsProven {
    fn eval<'ctx>(
        &self,
        _env: &mut Env<'ctx>,
        _ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        // Op-code 95 is reserved in the IR for byte-match parity with Scala
        // sigmastate. Scala typer rewrites `prop.isProven` to a `SigmaPropIsProven`
        // node, but its `costKind` is `notSupportedError` — the AOT graph-IR
        // rewrite removes the node before evaluation. Mirror that here: the
        // node is serializable but has no direct interpreter eval.
        Err(EvalError::Misc(
            "SigmaPropIsProven has no interpreter eval (frontend-only — Scala uses graph-IR rewrite to elide it)".to_string(),
        ))
    }
}
