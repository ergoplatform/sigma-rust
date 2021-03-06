//! THRESHOLD conjunction for sigma proposition

use super::SigmaBoolean;
use super::SigmaConjectureItems;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{SerializationError, SigmaSerializable};
use std::io::Error;
// use crate::sigma_protocol::sigma_boolean::SigmaConjecture;

/// THRESHOLD conjunction for sigma proposition
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Cthreshold {
    /// Number of conjectures to be proven
    pub n: i32,
    /// Items of the proposal
    pub items: SigmaConjectureItems<SigmaBoolean>,
}

impl SigmaSerializable for Cthreshold {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        w.put_i32(self.n)?;
        self.items.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let n = r.get_i32()?;
        let items = SigmaConjectureItems::<_>::sigma_parse(r)?;
        Ok(Cthreshold { n, items })
    }
}
