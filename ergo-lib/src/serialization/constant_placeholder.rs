use super::sigma_byte_writer::SigmaByteWrite;
use crate::ast::constant::ConstantPlaceholder;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};

use std::io;

impl SigmaSerializable for ConstantPlaceholder {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u32(self.id)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let id = r.get_u32()?;
        if let Some(c) = r.constant_store().get(id) {
            Ok(ConstantPlaceholder {
                id,
                tpe: c.tpe.clone(),
            })
        } else {
            Err(SerializationError::ConstantForPlaceholderNotFound(id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::constant::Constant;
    use crate::serialization::{
        constant_store::ConstantStore, sigma_byte_reader::SigmaByteReader,
        sigma_byte_writer::SigmaByteWriter,
    };
    use io::Cursor;
    use proptest::prelude::*;
    use sigma_ser::peekable_reader::PeekableReader;

    proptest! {

        #[test]
        fn ser_roundtrip(c in any::<Constant>()) {
            let mut data = Vec::new();
            let mut cs = ConstantStore::empty();
            let ph = cs.put(c);
            let mut w = SigmaByteWriter::new(&mut data, Some(&mut cs));
            ph.sigma_serialize(&mut w).expect("serialization failed");

            let cursor = Cursor::new(&mut data[..]);
            let pr = PeekableReader::new(cursor);
            let mut sr = SigmaByteReader::new(pr, cs);
            let ph_parsed = ConstantPlaceholder::sigma_parse(&mut sr).expect("parse failed");
            prop_assert_eq![ph, ph_parsed];
        }
    }
}
