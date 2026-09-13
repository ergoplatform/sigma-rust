use alloc::vec::Vec;

use crate::mir::expr::Expr;
use crate::mir::property_call::PropertyCall;
use crate::types::smethod::MethodId;
use crate::types::smethod::SMethod;

use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::types::TypeCode;
use super::SigmaParsingError;
use super::SigmaSerializable;
use super::SigmaSerializeResult;

impl SigmaSerializable for PropertyCall {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.method.obj_type.type_code().sigma_serialize(w)?;
        self.method.method_id().sigma_serialize(w)?;
        self.obj.sigma_serialize(w)?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let type_id = TypeCode::sigma_parse(r)?;
        let method_id = MethodId::sigma_parse(r)?;
        let obj = Expr::sigma_parse(r)?;
        let method =
            SMethod::from_ids(type_id, method_id)?.specialize_for(obj.tpe(), Vec::new())?;
        // Reject a method whose required ErgoTree version exceeds the tree's, at
        // DESERIALIZE — the JVM gates this in `SMethod.fromIds`, so it fires
        // structurally over the whole tree (a dead `if` branch is still parsed,
        // not lazily skipped). `MethodCall` already does this; `PropertyCall`
        // (e.g. `Global.none[T]`) was missing it, so a v6 property in a pre-v3
        // tree was accepted.
        if r.tree_version() < method.method_raw.min_version {
            return Err(SigmaParsingError::UnknownMethodId(
                method_id,
                type_id.value(),
            ));
        }
        Ok(PropertyCall::new(obj, method)?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::ergo_tree::ErgoTreeVersion;
    use crate::mir::expr::Expr;
    use crate::mir::property_call::PropertyCall;
    use crate::serialization::roundtrip_new_feature;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::scontext;
    use crate::types::sglobal;
    use crate::types::stype::SType;
    use alloc::vec;

    #[test]
    fn ser_roundtrip_property() {
        let mc = PropertyCall::new(Expr::Context, scontext::DATA_INPUTS_PROPERTY.clone()).unwrap();
        let expr = Expr::PropertyCall(mc.into());
        assert_eq![sigma_serialize_roundtrip(&expr), expr];
    }

    #[test]
    fn versioned_roundtrip() {
        // `Global.none` is a v6 (V3) property; a pre-V3 ErgoTree must reject it
        // at parse, mirroring MethodCall's version gate. (SANTA v6 audit: the
        // dead-branch `Global.none[UnsignedBigInt]` over-acceptance,
        // V6-PROPERTY-TYPEARG-GATE-01.)
        let pc: Expr = PropertyCall::new(
            Expr::Global,
            sglobal::NONE_METHOD
                .clone()
                .specialize_for(SType::SGlobal, vec![])
                .unwrap(),
        )
        .unwrap()
        .into();
        roundtrip_new_feature(&pc, ErgoTreeVersion::V3);
    }
}
