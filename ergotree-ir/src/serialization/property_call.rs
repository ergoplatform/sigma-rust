use hashbrown::HashMap;

use alloc::vec::Vec;

use crate::mir::expr::Expr;
use crate::mir::property_call::PropertyCall;
use crate::types::smethod::MethodId;
use crate::types::smethod::SMethod;
use crate::types::stype::SType;
use crate::types::stype_param::STypeVar;

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
        for type_arg in &self.method.method_raw.explicit_type_args {
            // Should not fail as existence of explicit type args is checked in constructor
            let tpe = &self.explicit_type_args[type_arg];
            tpe.sigma_serialize(w)?;
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let type_id = TypeCode::sigma_parse(r)?;
        let method_id = MethodId::sigma_parse(r)?;
        let obj = Expr::sigma_parse(r)?;
        let method = SMethod::from_ids(type_id, method_id)?;
        let explicit_type_args = method
            .method_raw
            .explicit_type_args
            .iter()
            .cloned()
            .map(|tpe| Ok((tpe, SType::sigma_parse(r)?)))
            .collect::<Result<HashMap<STypeVar, SType>, SigmaParsingError>>()?;
        // as in sigmastate: explicit type args fix the signature, otherwise infer from obj
        let method = if explicit_type_args.is_empty() {
            method.specialize_for(obj.tpe(), Vec::new())?
        } else {
            method.with_concrete_types(&explicit_type_args)
        };
        Ok(PropertyCall::with_type_args(
            obj,
            method,
            explicit_type_args,
        )?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::mir::expr::Expr;
    use crate::mir::property_call::PropertyCall;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::scontext;

    #[test]
    fn ser_roundtrip_property() {
        let mc = PropertyCall::new(Expr::Context, scontext::DATA_INPUTS_PROPERTY.clone()).unwrap();
        let expr = Expr::PropertyCall(mc.into());
        assert_eq![sigma_serialize_roundtrip(&expr), expr];
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod global_none_type_arg_tests {
    use crate::ergo_tree::ErgoTreeVersion;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_byte_reader::{from_bytes, SigmaByteRead};
    use crate::serialization::{SigmaParsingError, SigmaSerializable};
    use crate::types::stype::SType;

    fn parse_v3(bytes: &[u8]) -> Result<Expr, SigmaParsingError> {
        from_bytes(bytes).with_tree_version(ErgoTreeVersion::V3, Expr::sigma_parse)
    }

    // sigmastate 6.0.7: none is a PropertyCall with explicit type arg T, which
    // PropertyCallSerializer writes after obj: none[Int] = db6a0add04
    #[test]
    fn none_reads_and_writes_explicit_type_arg() {
        for (tpe, code) in [
            (SType::SInt, 0x04u8),
            (SType::SColl(SType::SByte.into()), 0x0e),
        ] {
            let scala = [0xdb, 0x6a, 0x0a, 0xdd, code];
            let expr = parse_v3(&scala).unwrap();
            assert_eq!(expr.tpe(), SType::SOption(tpe.into()));
            assert_eq!(expr.sigma_serialize_bytes().unwrap(), scala);
        }
    }

    // without the type arg the bytes are truncated for sigmastate
    #[test]
    fn none_without_type_arg_is_rejected() {
        assert!(parse_v3(&[0xdb, 0x6a, 0x0a, 0xdd]).is_err());
    }
}
