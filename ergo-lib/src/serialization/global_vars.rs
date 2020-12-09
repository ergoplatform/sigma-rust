#[cfg(test)]
mod tests {
    use crate::ast::expr::Expr;
    use crate::ast::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<GlobalVars>()) {
            let expr = Expr::GlobalVars(v);
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
