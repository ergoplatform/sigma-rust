//! ContextExtension type
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use sigma_ser::vlq_encode;
use std::collections::HashMap;
use std::io;

/// User-defined variables to be put into context
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ContextExtension {
    /// key-value pairs of variable id and it's value
    pub values: HashMap<u8, Vec<u8>>,
}

impl ContextExtension {
    /// Returns an empty ContextExtension
    pub fn empty() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl SigmaSerializable for ContextExtension {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, _: &mut W) -> Result<(), io::Error> {
        assert!(self.values.is_empty(), "implemented only for empty");
        Ok(())
    }
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(_: &mut R) -> Result<Self, SerializationError> {
        Ok(ContextExtension::empty())
    }
}
