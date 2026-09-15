use alloc::vec::Vec;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use ergotree_ir::mir::xor_of::XorOf;

impl Evaluable for XorOf {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bools = input_v.try_extract_into::<Vec<bool>>()?;
        ctx.add_per_item_jit_cost(20, 5, 32, input_v_bools.len() as u32)?;
        Ok(input_v_bools.into_iter().fold(false, |a, b| a ^ b).into())
    }
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_util::eval_out;
    use ergotree_ir::chain::context::Context;
    use ergotree_ir::mir::expr::Expr;
    use proptest::collection;
    use proptest::prelude::*;
    use sigma_test_util::force_any_val;

    proptest! {

        #[test]
        fn eval(bools in collection::vec(any::<bool>(), 0..=10)) {
            let expr: Expr = XorOf {input: Expr::Const(bools.clone().into()).into()}.into();
            let ctx = force_any_val::<Context>();
            let res = eval_out::<bool>(&expr, &ctx);
            // eval is true when collection has odd number of "true" values
            let expected = bools.into_iter().filter(|x| *x).count() & 1 == 1;
            prop_assert_eq!(res, expected);
        }
    }
}
