use std::io::Error;

use crate::mir::expr::Expr;
use crate::mir::method_call::MethodCall;
use crate::types::smethod::MethodId;
use crate::types::smethod::SMethod;

use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::types::TypeCode;
use super::SigmaParsingError;
use super::SigmaSerializable;

impl SigmaSerializable for MethodCall {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        self.method.obj_type.type_id().sigma_serialize(w)?;
        self.method.method_id().sigma_serialize(w)?;
        self.obj.sigma_serialize(w)?;
        self.args.sigma_serialize(w)?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let type_id = TypeCode::sigma_parse(r)?;
        let method_id = MethodId::sigma_parse(r)?;
        let obj = Expr::sigma_parse(r)?;
        let args = Vec::<Expr>::sigma_parse(r)?;
        let arg_types = args.iter().map(|arg| arg.tpe()).collect();
        let method = SMethod::from_ids(type_id, method_id)?.specialize_for(obj.tpe(), arg_types)?;
        Ok(MethodCall::new(obj, method, args)?)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::mir::expr::Expr;
    use crate::mir::method_call::MethodCall;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::scoll;
    use crate::types::stype::SType;
    use crate::types::stype_param::STypeVar;

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
}
