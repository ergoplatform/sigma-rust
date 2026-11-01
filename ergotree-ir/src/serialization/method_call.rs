use hashbrown::HashMap;

use alloc::vec::Vec;

use crate::ergo_tree::ErgoTreeVersion;
use crate::mir::expr::Expr;
use crate::mir::method_call::MethodCall;
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

impl SigmaSerializable for MethodCall {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.method.obj_type.type_code().sigma_serialize(w)?;
        self.method.method_id().sigma_serialize(w)?;
        self.obj.sigma_serialize(w)?;
        self.args.sigma_serialize(w)?;
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
        let args = Vec::<Expr>::sigma_parse(r)?;
        // sigmastate asserts non-empty args from v3 trees on (MethodCallSerializer.parse);
        // zero-arg methods use PropertyCall
        if args.is_empty() && r.tree_version() >= ErgoTreeVersion::V3 {
            return Err(SigmaParsingError::MethodCallWithoutArgs);
        }
        let arg_types = args.iter().map(|arg| arg.tpe()).collect();
        let method = SMethod::from_ids(type_id, method_id)?.specialize_for(obj.tpe(), arg_types)?;
        if r.tree_version() < method.method_raw.min_version {
            return Err(SigmaParsingError::UnknownMethodId(
                method_id,
                type_id.value(),
            ));
        }
        let explicit_type_args = method
            .method_raw
            .explicit_type_args
            .iter()
            .cloned()
            .zip(core::iter::from_fn(|| Some(SType::sigma_parse(r))))
            .map(|(tpe, res)| -> Result<(STypeVar, SType), SigmaParsingError> { Ok((tpe, res?)) })
            .collect::<Result<HashMap<STypeVar, SType>, _>>()?;
        Ok(MethodCall::with_type_args(
            obj,
            method,
            args,
            explicit_type_args,
        )?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use alloc::vec;

    use crate::ergo_tree::ErgoTreeVersion;
    use crate::mir::coll_by_index::ByIndex;
    use crate::mir::constant::Constant;
    use crate::mir::expr::Expr;
    use crate::mir::global_vars::GlobalVars;
    use crate::mir::method_call::MethodCall;
    use crate::mir::property_call::PropertyCall;
    use crate::serialization::sigma_byte_reader::{from_bytes, SigmaByteRead};
    use crate::serialization::{
        roundtrip_new_feature, sigma_serialize_roundtrip, SigmaParsingError, SigmaSerializable,
    };
    use crate::types::stype::SType;
    use crate::types::stype_param::STypeVar;
    use crate::types::{sbox, scoll, scontext, sglobal, sheader};

    #[test]
    fn ser_roundtrip() {
        let mc: Expr = MethodCall::new(
            vec![1i64, 2i64].into(),
            scoll::INDEX_OF_METHOD
                .clone()
                .with_concrete_types(&[(STypeVar::t(), SType::SLong)].iter().cloned().collect()),
            vec![2i64.into(), 0i32.into()],
        )
        .unwrap()
        .into();
        assert_eq![sigma_serialize_roundtrip(&mc), mc];
    }
    // test that methodcalls that are added in later versions via soft-fork can't be parsed with older version
    #[test]
    fn versioned_roundtrip() {
        let mc: Expr = MethodCall::new(
            Expr::Global,
            sglobal::SERIALIZE_METHOD
                .clone()
                .specialize_for(SType::SGlobal, vec![SType::SInt])
                .unwrap(),
            vec![Constant::from(1i32).into()],
        )
        .unwrap()
        .into();
        roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
    }

    fn parse_with_version(
        bytes: &[u8],
        version: ErgoTreeVersion,
    ) -> Result<Expr, SigmaParsingError> {
        from_bytes(bytes).with_tree_version(version, Expr::sigma_parse)
    }

    #[test]
    fn zero_arg_method_call_rejected_in_v3_tree() {
        // MethodCall(Coll(1, 2).reverse) with an explicit zero argument count
        let bytes = [0xDC, 12, 30, 0x10, 0x02, 0x02, 0x04, 0x00];
        assert_eq!(
            parse_with_version(&bytes, ErgoTreeVersion::V3),
            Err(SigmaParsingError::MethodCallWithoutArgs)
        );
    }

    #[test]
    fn zero_arg_check_pow_rejected_in_v3_tree() {
        let headers: Expr = PropertyCall::new(Expr::Context, scontext::HEADERS_PROPERTY.clone())
            .unwrap()
            .into();
        let header: Expr = ByIndex::new(headers, 0i32.into(), None).unwrap().into();
        let mc: Expr = MethodCall::new(header.clone(), sheader::CHECK_POW_METHOD.clone(), vec![])
            .unwrap()
            .into();
        let bytes = mc.sigma_serialize_bytes().unwrap();
        assert_eq!(
            parse_with_version(&bytes, ErgoTreeVersion::V3),
            Err(SigmaParsingError::MethodCallWithoutArgs)
        );
        // the property form is how a zero-argument method is serialized
        let pc: Expr = PropertyCall::new(header, sheader::CHECK_POW_METHOD.clone())
            .unwrap()
            .into();
        let bytes = pc.sigma_serialize_bytes().unwrap();
        assert_eq!(parse_with_version(&bytes, ErgoTreeVersion::V3), Ok(pc));
    }

    #[test]
    fn zero_arg_method_call_parses_before_v3() {
        let mc: Expr = MethodCall::new(
            GlobalVars::SelfBox.into(),
            sbox::VALUE_METHOD.clone(),
            vec![],
        )
        .unwrap()
        .into();
        let bytes = mc.sigma_serialize_bytes().unwrap();
        for version in [
            ErgoTreeVersion::V0,
            ErgoTreeVersion::V1,
            ErgoTreeVersion::V2,
        ] {
            assert_eq!(parse_with_version(&bytes, version), Ok(mc.clone()));
        }
    }
}
