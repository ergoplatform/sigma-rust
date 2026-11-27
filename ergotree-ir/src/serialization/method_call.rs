use hashbrown::HashMap;

use alloc::vec::Vec;

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
        if r.tree_version() >= crate::ergo_tree::ErgoTreeVersion::V3 && args.is_empty() {
            return Err(crate::mir::expr::InvalidArgumentError(
                "MethodCall: args must be non-empty in ErgoTree v3+".into(),
            )
            .into());
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
    use crate::mir::constant::Constant;
    use crate::mir::expr::Expr;
    use crate::mir::method_call::MethodCall;
    use crate::serialization::{roundtrip_new_feature, sigma_serialize_roundtrip};
    use crate::types::stype::SType;
    use crate::types::stype_param::STypeVar;
    use crate::types::{scoll, sglobal};

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

    #[test]
    fn v3_rejects_method_call_with_empty_args() {
        use crate::serialization::sigma_byte_reader::SigmaByteRead;
        use crate::serialization::{sigma_byte_reader, SigmaSerializable};
        use crate::types::sbox;

        // Wire bytes for MethodCall with 0 args on GlobalVars::SelfBox (box.value)
        let mut bytes = Vec::new();
        {
            let mut w =
                crate::serialization::sigma_byte_writer::SigmaByteWriter::new(&mut bytes, None);
            sbox::TYPE_CODE.sigma_serialize(&mut w).unwrap();
            sbox::VALUE_METHOD_ID.sigma_serialize(&mut w).unwrap();
            Expr::GlobalVars(crate::mir::global_vars::GlobalVars::SelfBox)
                .sigma_serialize(&mut w)
                .unwrap();
            Vec::<Expr>::new().sigma_serialize(&mut w).unwrap();
        }
        assert!(sigma_byte_reader::from_bytes(&bytes)
            .with_tree_version(ErgoTreeVersion::V0, MethodCall::sigma_parse)
            .is_ok());
        assert!(sigma_byte_reader::from_bytes(&bytes)
            .with_tree_version(ErgoTreeVersion::V3, MethodCall::sigma_parse)
            .is_err());
    }
}
