use super::sigma_byte_writer::SigmaByteWrite;
use crate::ast::ConstantPlaceholder;
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::serialization::sigma_serialize_roundtrip;
//     use proptest::prelude::*;

//     proptest! {

//         #[test]
//         fn ser_roundtrip(v in any::<ConstantPlaceholder>()) {
//             prop_assert_eq![sigma_serialize_roundtrip(&v), v];
//         }
//     }
// }
