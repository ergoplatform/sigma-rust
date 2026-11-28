//! Soft-fork error type and related convenience functions
use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::SigmaParsingError;
use alloc::vec::Vec;
use hashbrown::HashMap;
use thiserror::Error;

use crate::{serialization::types::TypeCode, types::smethod::MethodId};

/// Validation failures preserved during parsing. Tolerating one during evaluation
/// additionally requires the corresponding chain validation override.
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
    InvalidOpCode(u8),
    /// Reader exceeded the proposition window (validation rule 1014).
    #[error("read position exceeds the position limit")]
    PositionLimitExceeded,
    /// Registers/ContextExtension contained a v6.0 type (UnsignedBigInt, Header, Option), which is not allowed
    #[error("Can't use v6 types (UnsignedBigInt, Header, Option) in ContextExtension/Registers ")]
    V6TypeError,
}

/// Classify validation failures without deciding whether the chain permits them.
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
            if let Some(e) = err.downcast_ref::<SigmaParsingError>() {
                if e.is_position_limit_exceeded() {
                    return Some(&SoftForkError::PositionLimitExceeded);
                }
            }
            cur_err = err.source()
        }
        None
    }
}

/// Execute `f`, applying the fallback only for a validation failure approved by
/// the supplied chain settings. Classification alone never enables the fallback.
pub fn try_soft_forkable<T, E>(
    f: impl FnOnce() -> Result<T, E>,
    when_soft_fork: impl FnOnce() -> T,
    settings: &SigmaValidationSettings,
    activated_version: ErgoTreeVersion,
) -> Result<T, E>
where
    E: core::error::Error + 'static,
{
    match f() {
        Ok(t) => Ok(t),
        Err(e) if settings.is_soft_fork(&e, activated_version) => Ok(when_soft_fork()),
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

/// Status of a validation rule as agreed by the chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleStatus {
    /// The rule is enforced.
    Enabled,
    /// The rule is disabled; this alone does not approve a failed validation.
    Disabled,
    /// The rule was replaced by another rule.
    Replaced(u16),
    /// Codes added to a rule; methods use (type, method) byte pairs.
    Changed(Vec<u8>),
}

/// Chain validation overrides. Without overrides no validation error is tolerated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SigmaValidationSettings {
    rules: HashMap<u16, RuleStatus>,
}

impl SigmaValidationSettings {
    /// Override one known validation rule with its chain-provided status.
    pub fn updated(mut self, rule: u16, status: RuleStatus) -> Self {
        self.rules.insert(rule, status);
        self
    }

    /// Check a validation failure against the chain's voted settings.
    pub fn is_soft_fork<E: core::error::Error + 'static>(
        &self,
        error: &E,
        activated_version: ErgoTreeVersion,
    ) -> bool {
        let Some(error) = error.to_soft_fork() else {
            return false;
        };
        let v6 = activated_version >= ErgoTreeVersion::V3;
        let rule = match error {
            SoftForkError::InvalidPrimitiveType(_) => {
                if v6 {
                    1017
                } else {
                    1007
                }
            }
            SoftForkError::InvalidTypeCode(_) => {
                if v6 {
                    1018
                } else {
                    1008
                }
            }
            SoftForkError::NotSerializable(_) => 1009,
            SoftForkError::NoMethods(_) => 1010,
            SoftForkError::UnknownMethodId(_, _) => {
                if v6 {
                    1016
                } else {
                    1011
                }
            }
            SoftForkError::InvalidRootType => 1001,
            SoftForkError::DeserializedScriptError => 1000,
            SoftForkError::InvalidOpCode(_) => 1002,
            SoftForkError::V6TypeError => 1019,
            SoftForkError::PositionLimitExceeded => 1014,
        };
        match self.rules.get(&rule) {
            Some(RuleStatus::Replaced(_)) => true,
            Some(RuleStatus::Changed(codes)) => match error {
                SoftForkError::InvalidPrimitiveType(code)
                | SoftForkError::InvalidTypeCode(code)
                | SoftForkError::InvalidOpCode(code) => codes.contains(code),
                SoftForkError::NoMethods(code) => codes.contains(&code.value()),
                SoftForkError::UnknownMethodId(method, tpe) => {
                    codes.chunks_exact(2).any(|pair| pair == [*tpe, method.0])
                }
                _ => false,
            },
            _ => false,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod validation_settings_tests {
    use super::*;

    #[test]
    fn soft_fork_requires_the_exact_voted_code() {
        let error = SoftForkError::InvalidOpCode(0xfb);
        let settings = SigmaValidationSettings::default();
        assert!(!settings.is_soft_fork(&error, ErgoTreeVersion::V3));
        let settings = settings.updated(1002, RuleStatus::Changed(vec![0xfc]));
        assert!(!settings.is_soft_fork(&error, ErgoTreeVersion::V3));
        let settings = settings.updated(1002, RuleStatus::Changed(vec![0xfb]));
        assert!(settings.is_soft_fork(&error, ErgoTreeVersion::V3));
        assert!(!settings.is_soft_fork(&SoftForkError::InvalidRootType, ErgoTreeVersion::V3));
        assert!(!settings.is_soft_fork(
            &SigmaParsingError::Misc("hard error".into()),
            ErgoTreeVersion::V3
        ));
    }

    #[test]
    fn soft_fork_method_changes_match_type_method_pairs() {
        let error = SoftForkError::UnknownMethodId(MethodId(12), 101);
        let settings = SigmaValidationSettings::default()
            .updated(1016, RuleStatus::Changed(vec![101, 11, 102, 12]));
        assert!(!settings.is_soft_fork(&error, ErgoTreeVersion::V3));
        assert!(settings
            .updated(1016, RuleStatus::Changed(vec![101, 12]))
            .is_soft_fork(&error, ErgoTreeVersion::V3));
        assert!(!SigmaValidationSettings::default()
            .updated(1011, RuleStatus::Replaced(1016))
            .is_soft_fork(&error, ErgoTreeVersion::V3));
    }

    #[test]
    fn soft_fork_position_limit_needs_replacement_in_every_error_channel() {
        let errors = [
            SigmaParsingError::PositionLimitExceeded,
            SigmaParsingError::VlqEncode(
                sigma_ser::vlq_encode::VlqEncodingError::PositionLimitExceeded,
            ),
            SigmaParsingError::ScorexParsingError(
                sigma_ser::ScorexParsingError::PositionLimitExceeded,
            ),
        ];
        for error in errors {
            assert!(error.is_soft_fork());
            assert!(!SigmaValidationSettings::default().is_soft_fork(&error, ErgoTreeVersion::V3));
            assert!(SigmaValidationSettings::default()
                .updated(1014, RuleStatus::Replaced(1021))
                .is_soft_fork(&error, ErgoTreeVersion::V3));
        }
    }
}
