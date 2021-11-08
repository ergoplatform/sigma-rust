use std::convert::TryInto;
use std::fmt::Formatter;
use std::hash::Hash;

use bounded_vec::BoundedVec;

use crate::mir::expr::InvalidArgumentError;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;

use super::stype::SType;

/// Type variable for generic signatures
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct STypeVar {
    /// Type variable name (e.g. "T")
    name_bytes: BoundedVec<u8, 1, 254>,
}

impl std::fmt::Debug for STypeVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_string().fmt(f)
    }
}

impl STypeVar {
    /// Creates type variable from UTF8 text string of 1..255 length or returns an error
    pub fn new_from_str(name: &'static str) -> Result<Self, InvalidArgumentError> {
        Ok(Self {
            name_bytes: name.to_string().into_bytes().try_into()?,
        })
    }

    /// Creates type variable from bytes of UTF8 text string of 1..255 length or returns an error
    pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self, InvalidArgumentError> {
        // test if its UTF8
        Ok(match String::from_utf8(bytes.clone()) {
            Ok(_) => Self {
                name_bytes: bytes.try_into()?,
            },
            Err(_) => {
                return Err(InvalidArgumentError(format!(
                    "STypeVar: cannot decode {:?} from UTF8",
                    bytes
                )))
            }
        })
    }

    /// Returns text representation (e.g "T", etc.)
    pub fn as_string(&self) -> String {
        #[allow(clippy::unwrap_used)]
        String::from_utf8(self.name_bytes.as_vec().clone()).unwrap()
    }

    /// "T" type variable
    pub fn t() -> Self {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("T").unwrap()
    }

    /// "IV"(Input Value) type variable
    pub fn iv() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("IV").unwrap()
    }
    /// "OV"(Output Value) type variable
    pub fn ov() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new_from_str("OV").unwrap()
    }
}

impl SigmaSerializable for STypeVar {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.name_bytes.len() as u8)?;
        w.write_all(self.name_bytes.as_slice())?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let name_len = r.get_u8()?;
        let mut bytes = vec![0; name_len as usize];
        r.read_exact(&mut bytes)?;
        Ok(STypeVar::new_from_bytes(bytes)?)
    }
}

/// Type parameter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeParam {
    pub(crate) ident: STypeVar,
    upper_bound: Option<SType>,
    lower_bound: Option<SType>,
}
