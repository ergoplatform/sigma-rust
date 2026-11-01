use ergotree_ir::mir::tuple::Tuple;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for Tuple {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        // sigma-state restricts tuples to exactly 2 elements: `Tuple.eval`
        // rejects `items.length != 2` with "Invalid tuple" before doing any work
        // (sigma `values.scala`: "in v5.0 version we support only tuples of 2
        // elements to be equivalent with v4.x"). The check is unconditional —
        // no version gate, and sigma-state 6.0.3 evaluates every height through
        // it. sigma-rust models tuples as flat N-ary (`TupleItems =
        // BoundedVec<1,255>`), so without this guard it accepts non-pair tuples
        // the JVM rejects — a consensus accept/reject divergence (sigma-rust
        // would accept a spend the JVM rejects). Mirror the JVM: reject non-pairs.
        if self.items.len() != 2 {
            return Err(EvalError::Misc(format!(
                "Tuple: sigma-state allows only 2-element tuples, got {}",
                self.items.len()
            )));
        }
        let items_v = self
            .items
            .try_mapped_ref(|i| -> Result<Value<'ctx>, EvalError> {
                let v = i.eval(env, ctx)?;
                // Mirror the JVM Tuple eval, which `checkType`s each item
                // (values.scala:801/804): an item whose type is an unsupported tuple
                // (arity != 2) is rejected ("Unsupported tuple type", SType.scala:200).
                // Catches an arity-3 tuple constant carried as a pair item, which the
                // JVM refuses but sigma-rust would otherwise evaluate.
                crate::eval::check_value_type(&i.tpe())?;
                Ok(v)
            });
        Ok(Value::Tup(items_v?))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::{eval_out_wo_ctx, try_eval_out};
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval() {
        let e1: Expr = 1i64.into();
        let e2: Expr = GlobalVars::Height.into();
        let exprs = vec![e1, e2];
        let tuple: Expr = Tuple::new(exprs).unwrap().into();
        let res = eval_out_wo_ctx::<Value>(&tuple);
        assert!(matches!(res, Value::Tup(_)));
    }

    #[test]
    fn eval_rejects_non_pair_tuple() {
        // sigma-state rejects tuples with length != 2 ("Invalid tuple", JVM
        // consensus). A flat arity-3 tuple is constructible here (TupleItems
        // allows 1..=255), but must be rejected at eval to match the JVM — else
        // sigma-rust would accept a spend the JVM rejects.
        let e1: Expr = 1i64.into();
        let e2: Expr = 2i64.into();
        let e3: Expr = 3i64.into();
        let tuple: Expr = Tuple::new(vec![e1, e2, e3]).unwrap().into();
        let ctx = force_any_val::<Context>();
        let res = try_eval_out::<Value>(&tuple, &ctx);
        assert!(
            res.is_err(),
            "arity-3 Tuple must be rejected at eval, got {res:?}"
        );
    }

    #[test]
    fn eval_rejects_non_pair_tuple_expr_in_every_tree_version() {
        use crate::eval::test_util::try_eval_out_with_version;
        let ctx = force_any_val::<Context>();
        for items in [
            vec![1i32.into()],
            vec![1i32.into(), 2i32.into(), 3i32.into()],
        ] {
            let tuple: Expr = Tuple::new(items).unwrap().into();
            for tree_version in 0..=3u8 {
                assert!(
                    try_eval_out_with_version::<Value>(&tuple, &ctx, tree_version, 3).is_err(),
                    "non-pair Tuple evaluated under tree v{tree_version}"
                );
            }
        }
    }

    #[test]
    fn one_item_tuple_in_tree_fails_only_when_evaluated() {
        use crate::eval::reduce_to_crypto;
        use ergotree_ir::ergo_tree::ErgoTree;
        use ergotree_ir::serialization::SigmaSerializable;
        use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

        // v0 trees (header 0x00, or 0x10 with constant segregation; 0x0b is v3 sized) and
        // what sigma-state 6.0.7 does with each, activated version 3. `c1` is the constant
        // (Int)(1), bytes 60 01 04 02, and `cc` is the constant Coll[(Int)](c1).
        let cases = [
            ("00d1936001040260010402", false),   // c1 == c1
            ("100160010402d19373007300", false), // c1 == c1, segregated
            ("00d1938601040286010402", false),   // Tuple(1) == Tuple(1)
            ("00d1938c60010402010402", false),   // c1._1 == 1
            ("00d193b20c6001040102040000b20c6001040102040000", false), // cc(0) == cc(0)
            ("00d1ae0c6001040102d901016001040101", false), // cc.exists(x => true)
            ("100160010402d195010101019373007300", true), // if (true) true else c1 == c1
            ("00d193b10c60010401020402", true),  // cc.size == 1
            // Coll[(Int)]().forall(x => false): the lambda is never applied
            ("00d1af0c60010400d901016001040100", true),
            ("0b0fd1af0c60010400d901016001040100", true),
            // substConstants(<segregated c1 == c1 tree>, [], []).size > 0: sigma-state
            // re-serializes the constants and its type serializer throws
            ("00d191b1740e0c100160010402d19373007300100010000400", false),
            // ... and it throws with an out-of-range position, or with an unparseable body
            (
                "00d191b1740e0c100160010402d1937300730010010a10010e0400",
                false,
            ),
            ("00d191b1740e07100160010402ff100010000400", false),
            // ... but not for a pair constant, nor for a one-item tuple in an unsegregated body
            ("00d191b1740e0b1001580202d19373007300100010000400", true),
            ("00d191b1740e0b00d1936001040260010402100010000400", true),
            // Known stricter: cc.indexOf(c1, 0) == 0 is TrueProp in sigma-state; Rust
            // rejects the one-item tuple c1 as a method-call argument (see `expr.rs`)
            ("00d193dc0c1a0c6001040102026001040204000400", false),
        ];
        let mut ctx = force_any_val::<Context>();
        ctx.pre_header.version = 4;
        for (hex, accepted) in cases {
            let bytes = base16::decode(hex).unwrap();
            let tree = ErgoTree::sigma_parse_bytes(&bytes)
                .map_err(|e| format!("{hex}: {e:?}"))
                .unwrap();
            assert!(tree.proposition().is_ok(), "{hex} did not parse");
            assert_eq!(
                tree.sigma_serialize_bytes().unwrap(),
                bytes,
                "{hex} round-trip"
            );
            let res = reduce_to_crypto(&tree, &ctx).map(|r| r.sigma_prop);
            if accepted {
                assert_eq!(res.unwrap(), SigmaBoolean::TrivialProp(true), "{hex}");
            } else {
                assert!(res.is_err(), "{hex} evaluated: {res:?}");
            }
        }
    }

    #[test]
    fn eval_rejects_arity3_tuple_item_constant() {
        use crate::eval::test_util::try_eval_out_wo_ctx;
        use ergotree_ir::mir::constant::{Constant, Literal};
        use ergotree_ir::types::stuple::{STuple, TupleItems};
        use ergotree_ir::types::stype::SType;

        // SANTA Tuple.checkType_unsupported (inline): a pair Tuple whose item0 is
        // an arity-3 (Bool,Bool,Bool) tuple CONSTANT. The JVM `checkType`s each
        // Tuple item (values.scala:801/804) → arity != 2 → "Unsupported tuple
        // type" (SType.scala:200-202). sigma-rust models flat N-ary tuples, so the
        // value is representable; it must be rejected at eval to match the JVM.
        let triple = Constant {
            tpe: SType::STuple(STuple::triple(
                SType::SBoolean,
                SType::SBoolean,
                SType::SBoolean,
            )),
            v: Literal::Tup(
                TupleItems::try_from(vec![
                    Literal::Boolean(true),
                    Literal::Boolean(true),
                    Literal::Boolean(true),
                ])
                .unwrap(),
            ),
        };
        let tuple: Expr = Tuple::new(vec![Expr::Const(triple), 1i32.into()])
            .unwrap()
            .into();
        assert!(
            try_eval_out_wo_ctx::<Value>(&tuple).is_err(),
            "arity-3 tuple item constant must be rejected at eval (Unsupported tuple type)"
        );
    }
}
