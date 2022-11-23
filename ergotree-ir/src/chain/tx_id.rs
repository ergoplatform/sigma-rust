//! Transaction id type

use derive_more::Display;

use ergo_chain_types::Digest32;
use sigma_ser::ScorexSerializable;

use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;

/// Transaction id (ModifierId in sigmastate)
#[derive(PartialEq, Eq, Hash, Debug, Clone, Display)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct TxId(pub Digest32);

impl TxId {
    /// All zeros
    pub fn zero() -> TxId {
        TxId(Digest32::zero())
    }
}

impl SigmaSerializable for TxId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.0.scorex_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        Ok(Self(Digest32::scorex_parse(r)?))
    }
}

impl From<TxId> for String {
    fn from(v: TxId) -> Self {
        v.0.into()
    }
}

impl AsRef<[u8]> for TxId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
