use alloc::vec::Vec;
use ergotree_ir::mir::apply::Apply;
use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;
use hashbrown::HashMap;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

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
                // The JVM rejects applying a lambda whose argument type is
                // still a type variable (a `FunDef`-bound polymorphic lambda
                // applied without instantiation): `Value.checkType(argTpe, v)`
                // inside the closure reaches `isValueOfType`'s catch-all →
                // `Unknown type T`. The error fires only on application —
                // binding such a lambda and never applying it is accepted,
                // since the closure body never runs.
                if let Some(arg) = fv.args.iter().find(|a| matches!(a.tpe, SType::STypeVar(_))) {
                    return Err(EvalError::Misc(format!(
                        "Apply: Unknown type {:?} of lambda argument {}",
                        arg.tpe, arg.idx
                    )));
                }
                let arg_ids: Vec<ValId> = fv.args.iter().map(|a| a.idx).collect();
                let mut existing_variables = HashMap::new();
                let mut new_variables = vec![];
                arg_ids.iter().zip(args_v).for_each(|(idx, arg_v)| {
                    if let Some(old_val) = env.get(*idx) {
                        existing_variables.insert(idx, old_val.clone());
                    } else {
                        new_variables.push(*idx);
                    }
                    env.insert(*idx, arg_v);
                });
                let res = fv.body.eval(env, ctx);
                new_variables.into_iter().for_each(|idx| {
                    env.remove(&idx);
                });
                existing_variables
                    .into_iter()
                    .for_each(|(idx, orig_value)| {
                        env.insert(*idx, orig_value);
                    });

                res
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

    // SANTA HOF regression (`eval/v6/authored` vectors): `FunDef` (0xd7)
    // trees deserialize, the polymorphic *construct* evaluates, and applying
    // a lambda whose argument type is still a type variable is rejected —
    // mirroring the JVM boundary (`Unknown type T` fires at application via
    // `Value.checkType` inside the closure, never at binding).
    #[test]
    fn fundef_polymorphic_apply_boundary() {
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
        // The vector trees carry arbitrary-typed roots (eval-tier corpus);
        // the sized parse path rejects non-SigmaProp roots, so clear the
        // size bit and drop the size byte to route through the non-sized
        // path — the same leniency the conformance runner applies.
        fn lenient(bytes: &[u8]) -> alloc::vec::Vec<u8> {
            let mut out = alloc::vec::Vec::with_capacity(bytes.len() - 1);
            out.push(bytes[0] & !0x08);
            out.extend_from_slice(&bytes[2..]); // size VLQ is 1 byte here
            out
        }

        // End-to-end: parse → substitute constants → eval. The reject trees
        // may fail at either layer (Rust's `Apply::new` arg-type check fires
        // at parse, where the JVM defers to eval — both reject the spend).
        let run = |hex: &str| -> Result<i32, alloc::string::String> {
            let tree = ErgoTree::sigma_parse_bytes(&lenient(&hx(hex)))
                .map_err(|e| alloc::format!("parse: {e:?}"))?;
            let expr = tree
                .proposition()
                .map_err(|e| alloc::format!("proposition: {e:?}"))?;
            let ctx = force_any_val::<Context>();
            try_eval_out_with_version::<i32>(&expr, &ctx, 3, 3)
                .map_err(|e| alloc::format!("eval: {e:?}"))
        };

        // { val id[T] = {(x: Int) => x}; id(7) } — tpeArgs=[T], concrete arg type
        // { val id[T] = {(x: T) => x}; 5 }      — type-var arg, bound but never applied
        let accepts: &[(&str, i32)] = &[
            ("1b1701040ed801d70101670154d90102047202da7201017300", 7),
            ("1b1501040ad801d70101670154d9010267015472027300", 5),
        ];
        // Applying through the type var — all reject:
        // (x: T) => x  /  (x: T) => 5  /  (x: T) => x + x, each applied as id(7)
        let rejects: &[&str] = &[
            "1b1901040ed801d70101670154d901026701547202da7201017300",
            "1b1b02040a040ed801d70101670154d901026701547300da7201017301",
            "1b1c01040ed801d70101670154d901026701549a72027202da7201017300",
        ];
        for (hex, expected) in accepts {
            assert_eq!(run(hex).unwrap(), *expected, "{hex}: value mismatch");
        }
        for hex in rejects {
            assert!(
                run(hex).is_err(),
                "{hex}: applying a type-var-typed lambda argument must reject"
            );
        }
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
                    tpe_args: vec![],
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
