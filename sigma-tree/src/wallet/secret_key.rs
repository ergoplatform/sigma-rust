//! Secret types
use crate::sigma_protocol::{DlogProverInput, PrivateInput};

/// Types of secrets
#[derive(PartialEq, Debug, Clone)]
pub enum SecretKey {
    /// Secret exponent of a group element, i.e. secret w such as h = g^^w, where g is group generator, h is a public key.
    DlogSecretKey(DlogProverInput),
}

impl SecretKey {
    /// Generates random DlogProverInput
    pub fn random_dlog() -> SecretKey {
        SecretKey::DlogSecretKey(DlogProverInput::random())
    }
}

impl From<SecretKey> for PrivateInput {
    fn from(s: SecretKey) -> Self {
        match s {
            SecretKey::DlogSecretKey(dpi) => PrivateInput::DlogProverInput(dpi),
        }
    }
}

impl From<DlogProverInput> for SecretKey {
    fn from(pi: DlogProverInput) -> Self {
        SecretKey::DlogSecretKey(pi)
    }
}
