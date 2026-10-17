//! Soft-fork error type and related convenience functions
use thiserror::Error;

use crate::{serialization::types::TypeCode, types::smethod::MethodId};

/// Represents soft-forkable conditions that can be tolerated when parsing.
#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum SoftForkError {
    /// Primitive type doesn't exist
    #[error("Primitive type with code {0} doesn't exist")]
    InvalidPrimitiveType(u8),
    /// Invalid Type Code
    #[error("type parsing error, invalid type code: {0}({0:#04X})")]
    InvalidTypeCode(u8),
    /// Type can not be serialized
    #[error("Type is not serializable: {0}")]
    NotSerializable(&'static str),
    /// Type has no methods
    #[error("Type with code {0:?} has no methods container")]
    NoMethods(TypeCode),
    /// Unknown method ID for given type code
    #[error("No method id {0:?} found in type companion with type id {1:?} ")]
    UnknownMethodId(MethodId, u8),
    /// ErgoTree root error. ErgoTree root type should be SigmaProp
    #[error("Expected ErgoTree root type to be SigmaProp")]
    InvalidRootType,
    /// Deserialized Script has invalid type
    #[error("Deserialized script is of invalid type")]
    DeserializedScriptError,
    /// OpCode doesn't exist or can't be parsed
    #[error("Invalid opcode")]
    InvalidOpCode(String),
    /// Registers/ContextExtension contained a v6.0 type (UnsignedBigInt, Header, Option), which is not allowed
    #[error("Can't use v6 types (UnsignedBigInt, Header, Option) in ContextExtension/Registers ")]
    V6TypeError,
}

/// Convenience trait for checking if an error's source is a [`SoftForkError`]
pub trait IsSoftForkable {
    /// Returns true if error's source is a [`SoftForkError`]
    fn is_soft_fork(&self) -> bool;
    /// Attempt to convert an error to a [`SoftForkError`]
    fn to_soft_fork(&self) -> Option<&SoftForkError>;
}

impl<E> IsSoftForkable for E
where
    E: core::error::Error + 'static,
{
    fn is_soft_fork(&self) -> bool {
        self.to_soft_fork().is_some()
    }

    fn to_soft_fork(&self) -> Option<&SoftForkError> {
        let mut cur_err: Option<&dyn core::error::Error> = Some(self);
        while let Some(err) = cur_err {
            if err.is::<SoftForkError>() {
                return err.downcast_ref();
            }
            cur_err = err.source()
        }
        None
    }
}

/// Executes `f` and returns its output. If `f` raises an error caused by a [`SoftForkError`], then `when_soft_fork` will be executed, otherwise the error will be returned as-is
pub fn try_soft_forkable<T, E>(
    f: impl FnOnce() -> Result<T, E>,
    when_soft_fork: impl FnOnce() -> T,
) -> Result<T, E>
where
    E: core::error::Error + 'static,
{
    match f() {
        Ok(t) => Ok(t),
        Err(e) if e.is_soft_fork() => Ok(when_soft_fork()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {

    use crate::{serialization::SigmaParsingError, soft_fork::IsSoftForkable};

    use super::SoftForkError;

    #[derive(thiserror::Error, Debug)]
    #[error("")]
    struct Error3(#[from] SigmaParsingError);

    #[test]
    fn check_soft_fork_cause() {
        let e0 = SoftForkError::V6TypeError;
        assert!(e0.is_soft_fork());
        let e1 = SigmaParsingError::SoftForkError(SoftForkError::V6TypeError);
        assert!(e1.is_soft_fork());
        let e2 = SigmaParsingError::Misc("".into());
        assert!(!e2.is_soft_fork());
        assert!(Error3(e1).is_soft_fork());
        assert!(!Error3(e2).is_soft_fork());
    }
}
