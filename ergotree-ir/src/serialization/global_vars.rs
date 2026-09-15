#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use crate::mir::expr::Expr;
    use crate::mir::global_vars::GlobalVars;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::serialization::SigmaSerializable;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<GlobalVars>()) {
            let expr = Expr::GlobalVars(v);
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn last_block_utxo_root_hash_is_a_bare_op_code() {
        // `LastBlockUtxoRootHash` serializes as the bare dedicated op code 0xa6
        // (JVM `OpCodes.LastBlockUtxoRootHashCode`, registered as a case-object
        // serializer in `ValueSerializer.scala`) — the op-form twin of the
        // `CONTEXT.LastBlockUtxoRootHash` PropertyCall form.
        let expr = Expr::GlobalVars(GlobalVars::LastBlockUtxoRootHash);
        assert_eq!(expr.sigma_serialize_bytes().unwrap(), vec![0xa6]);
        assert_eq!(Expr::sigma_parse_bytes(&[0xa6]).unwrap(), expr);
    }
}
