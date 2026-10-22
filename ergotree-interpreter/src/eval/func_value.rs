use ergotree_ir::mir::func_value::FuncValue;
use ergotree_ir::mir::value::Lambda;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for FuncValue {
    fn eval<'ctx>(&self, _env: &mut Env, _ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        // The JVM supports only unary functions: FuncValue.eval
        // (values.scala:1042-1056) builds the closure iff args.length == 1 and
        // otherwise throws "Function must have 1 argument". The reject fires at
        // closure CREATION (BlockValue evaluates ValDefs eagerly), so a bound
        // non-unary lambda rejects even when never applied — but NOT at parse:
        // both serializers are arity-agnostic, and a non-unary lambda on the
        // dead branch of a lazy `if` must still accept.
        if self.args().len() != 1 {
            return Err(EvalError::Misc(format!(
                "Function must have 1 argument, but was: {self:?}"
            )));
        }
        Ok(Value::Lambda(Lambda {
            args: self.args().to_vec(),
            body: self.body().clone().into(),
        }))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::eval::test_util::try_eval_out_with_version;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::serialization::SigmaSerializable;
    use sigma_test_util::force_any_val;

    fn hx(s: &str) -> alloc::vec::Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    // SANTA FuncValue.non_unary_arity (eval/v5/authored, JVM-blessed): the
    // closure-creation arity gate, end-to-end from the vector bytes (v0 trees,
    // no size bit — the unsized parse path takes the Int roots as-is).
    #[test]
    fn non_unary_lambda_rejects_at_closure_creation() {
        let ctx = force_any_val::<Context>();
        let run = |hex: &str| -> Result<i32, alloc::string::String> {
            let tree = ErgoTree::sigma_parse_bytes(&hx(hex))
                .map_err(|e| alloc::format!("parse: {e:?}"))?;
            let expr = tree
                .proposition()
                .map_err(|e| alloc::format!("proposition: {e:?}"))?;
            try_eval_out_with_version::<i32>(&expr, &ctx, 0, 2)
                .map_err(|e| alloc::format!("eval: {e:?}"))
        };
        // { val add = (x:Int,y:Int) => x+y; add(3,4) }  — applied
        // { val add = (x:Int,y:Int) => x+y; 5 }         — bound, never applied
        //                                                  (ValDefs eval eagerly)
        // { val f = () => 5; f() }                      — the zero-arg side
        for hex in [
            "100204060408d801d601d902020403049a72027203da72010273007301",
            "1001040ad801d601d902020403049a720272037300",
            "1001040ad801d601d9007300da720100",
        ] {
            assert!(
                run(hex).is_err(),
                "{hex}: non-unary lambda must reject at closure creation"
            );
        }
        // if (true) 5 else { val add = (x:Int,y:Int) => x+y; add(3,4) } — the
        // 2-arg lambda sits on the dead branch of a lazy If and never evaluates:
        // the tree ACCEPTS (the gate is eval-time, not parse-time).
        assert_eq!(
            run("10040101040a040604089573007301d801d601d902020403049a72027203da72010273027303")
                .unwrap(),
            5,
            "lazy-if twin: a never-evaluated non-unary lambda must not reject the tree"
        );
    }
}
