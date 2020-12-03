use std::io::Error;

use crate::ast::global_vars::GlobalVars;

use super::sigma_byte_reader::SigmaByteRead;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SerializationError;
use super::SigmaSerializable;

impl SigmaSerializable for GlobalVars {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        todo!()
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;
    use proptest::prelude::*;

    proptest! {

        #[ignore]
        fn ser_roundtrip(v in any::<GlobalVars>()) {
            let expr = Expr::GlobalVars(v);
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}
