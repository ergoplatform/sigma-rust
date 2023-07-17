use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::EvalError::Misc;
use crate::eval::Evaluable;
use ergo_chain_types::EcPoint;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::serialization::SigmaSerializable;

impl Evaluable for DecodePoint {
    fn eval(&self, env: &mut Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let point_bytes = self.input.eval(env, ctx)?.try_extract_into::<Vec<u8>>()?;
        let point: EcPoint = SigmaSerializable::sigma_parse_bytes(&point_bytes).map_err(|_| {
            Misc(String::from(
                "DecodePoint: Failed to parse EC point from bytes",
            ))
        })?;
        Ok(point.into())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use crate::eval::tests::eval_out_wo_ctx;

    use super::*;

    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::serialization::SigmaSerializable;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn eval(ecp in any::<EcPoint>()) {
            let bytes = ecp.sigma_serialize_bytes().unwrap();
            let expr: Expr = DecodePoint {input: Expr::Const(bytes.into()).into()}.into();
            let res = eval_out_wo_ctx::<EcPoint>(&expr);
            prop_assert_eq!(res, ecp);
        }
    }
}
