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
#[derive(PartialEq, Eq, Clone)]
pub struct STypeVar {
    /// Type variable name (e.g. "T")
    name: BoundedVec<u8, 1, 254>,
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for STypeVar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.as_vec().hash(state);
    }
}

impl std::fmt::Debug for STypeVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_string().fmt(f)
    }
}

impl STypeVar {
    /// Creates type variable from UTF8 text string of 1..255 length or returns an error
    pub fn new(name: String) -> Result<Self, InvalidArgumentError> {
        if name.as_bytes().len() < u8::MAX as usize {
            Ok(Self {
                name: name.into_bytes().try_into()?,
            })
        } else {
            Err(InvalidArgumentError(format!(
                "'{0}' exceeds max length (254 bytes)",
                name
            )))
        }
    }

    /// Returns text representation (e.g "T", etc.)
    pub fn as_string(&self) -> String {
        #[allow(clippy::unwrap_used)]
        String::from_utf8(self.name.as_vec().clone()).unwrap()
    }

    /// "T" type variable
    pub fn t() -> Self {
        #[allow(clippy::unwrap_used)]
        STypeVar::new("T".to_string()).unwrap()
    }

    /// "IV"(Input Value) type variable
    pub fn iv() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new("IV".to_string()).unwrap()
    }
    /// "OV"(Input Value) type variable
    pub fn ov() -> STypeVar {
        #[allow(clippy::unwrap_used)]
        STypeVar::new("OV".to_string()).unwrap()
    }
}

impl SigmaSerializable for STypeVar {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.name.len() as u8)?;
        w.write_all(self.name.as_slice())?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let name_len = r.get_u8()?;
        let mut bytes = vec![0; name_len as usize];
        r.read_exact(&mut bytes)?;
        Ok(STypeVar {
            name: bytes.try_into()?,
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
