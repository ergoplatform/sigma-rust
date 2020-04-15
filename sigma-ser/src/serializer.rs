use super::vlq_encode;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("vlq encode error")]
    VlqEncode(vlq_encode::VlqEncodingError),
    #[error("io error")]
    Io(io::Error),
}

impl From<vlq_encode::VlqEncodingError> for SerializationError {
    fn from(error: vlq_encode::VlqEncodingError) -> Self {
        SerializationError::VlqEncode(error)
    }
}

impl From<io::Error> for SerializationError {
    fn from(error: io::Error) -> Self {
        SerializationError::Io(error)
    }
}

pub trait SigmaSerializable: Sized {
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: W) -> Result<(), io::Error>;
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: R) -> Result<Self, SerializationError>;
}
