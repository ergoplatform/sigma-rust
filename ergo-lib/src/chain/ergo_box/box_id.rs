//! Box id type
use std::convert::TryFrom;
use std::convert::TryInto;

use ergotree_ir::chain::digest::Digest32;
use ergotree_ir::ir_ergo_box::IrBoxId;

use ergotree_ir::serialization::SigmaSerializeResult;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use crate::chain::{Base16DecodedBytes, Base16EncodedBytes, Digest32Error, DigestRef};

use derive_more::From;
use derive_more::Into;
use ergotree_ir::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SigmaParsingError,
    SigmaSerializable,
};
#[cfg(test)]
use proptest_derive::Arbitrary;

/// newtype for box ids
#[derive(PartialEq, Eq, Hash, Debug, Clone, From, Into)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(test, derive(Arbitrary))]
pub struct BoxId(#[serde(with = "DigestDef")] Digest32);

impl BoxId {
    /// Size in bytes
    pub const SIZE: usize = Digest32::SIZE;

    /// All zeros
    pub fn zero() -> BoxId {
        BoxId(Digest32::zero())
    }
}

impl AsRef<[u8]> for BoxId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

// todo-sab Don't forget to clean-up after moving chain types to IR crate
#[cfg(feature = "json")]
impl From<BoxId> for String {
    fn from(v: BoxId) -> Self {
        let bytes: Base16EncodedBytes = v.0.into();
        bytes.into()
    }
}

impl From<&IrBoxId> for BoxId {
    fn from(irb: &IrBoxId) -> Self {
        let u8bytes: Vec<u8> = irb.0.iter().map(|b| *b as u8).collect();
        let arr: [u8; Digest32::SIZE] = u8bytes.as_slice().try_into().unwrap();
        BoxId(arr.into())
    }
}

impl From<BoxId> for IrBoxId {
    fn from(id: BoxId) -> Self {
        let i8bytes: Vec<i8> = id.0 .0.iter().map(|b| *b as i8).collect();
        IrBoxId::new(i8bytes.try_into().unwrap())
    }
}

// todo-sab Don't forget to clean-up after moving chain types to IR crate
impl TryFrom<String> for BoxId {
    type Error = Digest32Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = Base16DecodedBytes::try_from(value)?;
        Ok(Digest32::try_from(bytes)?.into())
    }
}

impl SigmaSerializable for BoxId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.0.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        Ok(Self(Digest32::sigma_parse(r)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<BoxId>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
