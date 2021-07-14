#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use crate::mir::expr::Expr;
    use crate::mir::global_vars::GlobalVars;
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
