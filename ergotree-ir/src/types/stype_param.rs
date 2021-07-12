use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;

use super::stype::SType;

/// Type variable for generic signatures
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct STypeVar {
    /// Type variable name (e.g. "T")
    pub name: String,
}

impl STypeVar {
    /// "T" type variable
    pub fn t() -> Self {
        STypeVar {
            name: "T".to_string(),
        }
    }

    /// "IV"(Input Value) type variable
    pub fn iv() -> STypeVar {
        STypeVar {
            name: "IV".to_string(),
        }
    }
    /// "OV"(Input Value) type variable
    pub fn ov() -> STypeVar {
        STypeVar {
            name: "OV".to_string(),
        }
    }
}

impl SigmaSerializable for STypeVar {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        let bytes = self.name.as_bytes();
        assert!(
            bytes.len() < u8::MAX as usize,
            "STypeVar::name exceeds 255 bytes"
        );
        w.put_u8(bytes.len() as u8)?;
        w.write_all(bytes)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let name_len = r.get_u8()?;
        let mut bytes = vec![0; name_len as usize];
        r.read_exact(&mut bytes)?;
        Ok(STypeVar {
            name: String::from_utf8(bytes).map_err(|err| {
                SigmaParsingError::ValueOutOfBounds(format!(
                    "cannot parse UTF-8 STypeVar::name from bytes with error: {:?}",
                    err
                ))
            })?,
        })
    }
}

/// Type parameter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    pub(crate) ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}
