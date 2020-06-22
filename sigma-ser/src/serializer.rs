use super::vlq_encode;
use std::io;
use thiserror::Error;

/// Ways serialization might fail
#[derive(Error, Debug)]
pub enum SerializationError {
    /// Failed to parse op
    #[error("op parsing error")]
    InvalidOpCode,
    /// Lacking support for the op
    #[error("not implemented op error")]
    NotImplementedOpCode(u8),
    /// Failed to parse type
    #[error("type parsing error")]
    InvalidTypePrefix,
    /// Failed to decode VLQ
    #[error("vlq encode error")]
    VlqEncode(vlq_encode::VlqEncodingError),
    /// IO fail (EOF, etc.)
    #[error("io error")]
    Io(io::Error),
    /// Misc fail
    #[error("misc error")]
    Misc(String),
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

/// Consensus-critical serialization for Ergo
pub trait SigmaSerializable: Sized {
    /// Write `self` to the given `writer`.
    /// This function has a `sigma_` prefix to alert the reader that the
    /// serialization in use is consensus-critical serialization    
    /// Notice that the error type is [`std::io::Error`]; this indicates that
    /// serialization MUST be infallible up to errors in the underlying writer.
    /// In other words, any type implementing `SigmaSerializable`
    /// must make illegal states unrepresentable.
    fn sigma_serialize<W: vlq_encode::WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error>;

    /// Try to read `self` from the given `reader`.
    /// `sigma-` prefix to alert the reader that the serialization in use
    /// is consensus-critical
    fn sigma_parse<R: vlq_encode::ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError>;
}
