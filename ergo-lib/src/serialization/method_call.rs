use std::io::Error;

use crate::ast::expr::Expr;
use crate::ast::method_call::MethodCall;
use crate::types::smethod::MethodId;
use crate::types::smethod::SMethod;
use crate::types::stype_companion::TypeId;

use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SerializationError;
use super::SigmaSerializable;

impl SigmaSerializable for MethodCall {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        self.method.obj_type.type_id().sigma_serialize(w)?;
        self.method.method_id().sigma_serialize(w)?;
        self.obj.sigma_serialize(w)?;
        w.put_u32(self.args.len() as u32)?;
        self.args.iter().try_for_each(|a| a.sigma_serialize(w))?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let type_id = TypeId::sigma_parse(r)?;
        let method_id = MethodId::sigma_parse(r)?;
        let obj = Expr::sigma_parse(r)?;
        let args_count = r.get_u32()?;
        let mut args = Vec::with_capacity(args_count as usize);
        for _ in 0..args_count {
            args.push(Expr::sigma_parse(r)?);
        }
        Ok(MethodCall {
            obj,
            method: SMethod::from_ids(type_id, method_id),
            args,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::constant::Constant;
    use crate::ast::expr::Expr;
    use crate::ast::global_vars::GlobalVars;
    use crate::ast::method_call::MethodCall;
    use crate::serialization::sigma_serialize_roundtrip;
    use crate::types::sbox;

    #[test]
    fn ser_roundtrip_property() {
        let mc: Expr = Box::new(MethodCall {
            obj: Box::new(GlobalVars::SelfBox).into(),
            method: sbox::GET_REG_METHOD.clone(),
            args: vec![Box::new(Constant::from(0i8)).into()],
        })
        .into();
        assert_eq![sigma_serialize_roundtrip(&mc), mc];
    }
}
