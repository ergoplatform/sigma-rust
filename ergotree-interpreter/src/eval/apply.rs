use alloc::vec::Vec;
use ergotree_ir::mir::apply::Apply;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::eval::LambdaInvoker;

impl Evaluable for Apply {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let func_v: Value<'ctx> = self.func.eval(env, ctx)?;
        let args_v: Vec<Value> = self
            .args
            .iter()
            .map(|arg| arg.eval(env, ctx))
            .collect::<Result<_, EvalError>>()?;
        match func_v {
            Value::Lambda(fv) => {
                // The body evaluates in the lambda's CAPTURED environment
                // (extended with the argument bindings) — not in the caller's
                // env. Mirrors the JVM, where `FuncValue.eval` returns a
                // closure over the defining env; the previous bind-into-the-
                // caller's-env dance lost the outer binding of a curried
                // lambda (`add(3)(1)`) as soon as the outer `Apply` returned.
                LambdaInvoker::new(&fv).invoke(ctx, args_v)
            }
            _ => Err(EvalError::UnexpectedValue(format!(
                "expected func_v to be Value::FuncValue got: {0:?}",
                func_v
            ))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use alloc::boxed::Box;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::bin_op::RelationOp;
    use ergotree_ir::mir::block::BlockValue;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::func_value::FuncArg;
    use ergotree_ir::mir::func_value::FuncValue;
    use ergotree_ir::mir::val_def::ValDef;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::types::stype::SType;

    use crate::eval::test_util::eval_out_wo_ctx;

    use super::*;

    // SANTA HOF regression (`HOF_currying_Apply_of_Apply` vector): a lambda
    // returned from another lambda must capture its defining environment —
    // JVM `FuncValue.eval` returns a closure over the env it was created in.
    // Pre-fix, `Apply` bound arguments into the CALLER's env and removed them
    // after the body ran, so the inner lambda of
    // `add = (a: Int) => (b: Int) => a + b` lost `a` the moment the outer
    // Apply returned, and `add(3)(1)` errored instead of evaluating to 4.
    #[test]
    fn curried_lambda_captures_environment() {
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
        // { val add = {(a: Int) => {(b: Int) => a + b}}; add(3)(1) } — the
        // vector tree (v3, segregated, sized). Clear the size bit and drop
        // the size byte: the eval-tier corpus carries an Int-typed root,
        // which the sized parse path rejects.
        let bytes = {
            let b = hx("1b200204060402d801d601d9010204d90103049a72027203dada7201017300017301");
            let mut out = alloc::vec::Vec::with_capacity(b.len() - 1);
            out.push(b[0] & !0x08);
            out.extend_from_slice(&b[2..]);
            out
        };
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        let expr = tree.proposition().unwrap();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            try_eval_out_with_version::<i32>(&expr, &ctx, 3, 3).unwrap(),
            4,
            "add(3)(1) must evaluate to 4 — the returned lambda must keep its captured `a`"
        );
    }

    // The capture must hold uniformly across invocation sites: a lambda that
    // escapes its creation site and is then fed to a HOF must still see its
    // captured bindings. `{ val mk = (a: Int) => (b: Int) => a + b;
    // val f = mk(3); Coll(1, 2).map(f) }` → [4, 5]. Pre-fix, `Coll.map`
    // bound the argument into the caller's env, where `a` no longer existed.
    #[test]
    fn escaped_lambda_into_hof_keeps_captured_env() {
        use ergotree_ir::mir::apply::Apply;
        use ergotree_ir::mir::bin_op::ArithOp;
        use ergotree_ir::mir::coll_map::Map;
        use ergotree_ir::mir::collection::Collection;

        let inner_lambda: Expr = FuncValue::new(
            vec![FuncArg {
                idx: 3.into(),
                tpe: SType::SInt,
            }],
            Expr::BinOp(
                BinOp {
                    kind: ArithOp::Plus.into(),
                    left: Box::new(
                        ValUse {
                            val_id: 2.into(),
                            tpe: SType::SInt,
                        }
                        .into(),
                    ),
                    right: Box::new(
                        ValUse {
                            val_id: 3.into(),
                            tpe: SType::SInt,
                        }
                        .into(),
                    ),
                }
                .into(),
            ),
        )
        .into();
        let mk: Expr = FuncValue::new(
            vec![FuncArg {
                idx: 2.into(),
                tpe: SType::SInt,
            }],
            inner_lambda,
        )
        .into();
        let f: Expr = Apply::new(
            ValUse {
                val_id: 1.into(),
                tpe: mk.tpe(),
            }
            .into(),
            vec![Expr::Const(3i32.into())],
        )
        .unwrap()
        .into();
        let block: Expr = BlockValue {
            items: vec![
                ValDef {
                    id: 1.into(),
                    rhs: Box::new(mk),
                }
                .into(),
                ValDef {
                    id: 4.into(),
                    rhs: Box::new(f.clone()),
                }
                .into(),
            ],
            result: Box::new(
                Map::new(
                    Collection::new(
                        SType::SInt,
                        vec![Expr::Const(1i32.into()), Expr::Const(2i32.into())],
                    )
                    .unwrap()
                    .into(),
                    ValUse {
                        val_id: 4.into(),
                        tpe: f.tpe(),
                    }
                    .into(),
                )
                .unwrap()
                .into(),
            ),
        }
        .into();
        assert_eq!(
            eval_out_wo_ctx::<alloc::vec::Vec<i32>>(&block),
            alloc::vec![4, 5],
            "the escaped lambda must keep `a = 3` when invoked by Coll.map"
        );
    }

    #[test]
    fn eval_user_defined_func_call() {
        let arg = Expr::Const(1i32.into());
        let bin_op = Expr::BinOp(
            BinOp {
                kind: RelationOp::Eq.into(),
                left: Box::new(
                    ValUse {
                        val_id: 1.into(),
                        tpe: SType::SInt,
                    }
                    .into(),
                ),
                right: Box::new(
                    ValUse {
                        val_id: 2.into(),
                        tpe: SType::SInt,
                    }
                    .into(),
                ),
            }
            .into(),
        );
        let body = Expr::BlockValue(
            BlockValue {
                items: vec![ValDef {
                    id: 2.into(),
                    rhs: Box::new(Expr::Const(1i32.into())),
                }
                .into()],
                result: Box::new(bin_op),
            }
            .into(),
        );
        let apply: Expr = Apply::new(
            FuncValue::new(
                vec![FuncArg {
                    idx: 1.into(),
                    tpe: SType::SInt,
                }],
                body,
            )
            .into(),
            vec![arg],
        )
        .unwrap()
        .into();
        assert!(eval_out_wo_ctx::<bool>(&apply));
    }
}
