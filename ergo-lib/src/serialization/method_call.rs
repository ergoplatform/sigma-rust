use std::io::Error;

use crate::ast::method_call::MethodCall;

use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SerializationError;
use super::SigmaSerializable;

impl SigmaSerializable for MethodCall {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        w.put_u8(self.method.obj_type.type_id().0)?;
        w.put_u8(self.method.method_id().0)?;
        self.obj.sigma_serialize(w)?;
        if !self.args.is_empty() {
            self.args.iter().try_for_each(|a| a.sigma_serialize(w))?;
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        todo!()
        // let type_id = r.get_u8()?;
        // let method_id = r.get_u8()?;
        // let obj = Expr::sigma_parse(r)?;
    }
}

#[cfg(test)]
mod tests {
    // use crate::ast::expr::Expr;
    // use crate::ast::method_call::MethodCall;
    // use crate::serialization::sigma_serialize_roundtrip;
    // use crate::types::scontext;

    // #[test]
    // fn ser_roundtrip_property() {
    //     let mc = MethodCall {
    //         tpe: scontext::DATA_INPUTS_METHOD.tpe().clone(),
    //         obj: Box::new(Expr::Context),
    //         method: scontext::DATA_INPUTS_METHOD.clone(),
    //         args: vec![],
    //     };
    //     let expr = Expr::MethodCall(mc);
    //     assert_eq![sigma_serialize_roundtrip(&expr), expr];
    // }
}
