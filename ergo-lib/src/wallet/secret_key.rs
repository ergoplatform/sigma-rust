//! Secret types
use crate::chain::address::Address;
use crate::sigma_protocol::private_input::{DlogProverInput, PrivateInput};

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

    /// Parse DlogSecretKey from bytes (SEC-1-encoded scalar)
    pub fn dlog_from_bytes(bytes: &[u8; DlogProverInput::SIZE_BYTES]) -> Option<SecretKey> {
        DlogProverInput::from_bytes(bytes).map(SecretKey::DlogSecretKey)
    }

    /// Address (encoded public image)
    pub fn get_address_from_public_image(&self) -> Address {
        match self {
            SecretKey::DlogSecretKey(dpi) => Address::P2PK(dpi.public_image()),
        }
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
