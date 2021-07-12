use super::sigma_byte_writer::SigmaByteWrite;
use crate::mir::constant::ConstantPlaceholder;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};

use std::io;

impl SigmaSerializable for ConstantPlaceholder {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u32(self.id)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let id = r.get_u32()?;
        if let Some(c) = r.constant_store().get(id) {
            Ok(ConstantPlaceholder {
                id,
                tpe: c.tpe.clone(),
            })
        } else {
            Err(SigmaParsingError::ConstantForPlaceholderNotFound(id))
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::constant::Constant;
    use crate::serialization::{
        constant_store::ConstantStore, sigma_byte_reader::SigmaByteReader,
        sigma_byte_writer::SigmaByteWriter,
    };
    use io::Cursor;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(c in any::<Constant>()) {
            let mut data = Vec::new();
            let mut cs = ConstantStore::empty();
            let ph = cs.put(c);
            let mut w = SigmaByteWriter::new(&mut data, Some(cs));
            ph.sigma_serialize(&mut w).expect("serialization failed");
            let cs2 =  w.constant_store.unwrap();
            let mut sr = SigmaByteReader::new(Cursor::new(&mut data[..]), cs2);
            let ph_parsed = ConstantPlaceholder::sigma_parse(&mut sr).expect("parse failed");
            prop_assert_eq![ph, ph_parsed];
        }
    }
}
