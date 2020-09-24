//! Secret key
use sigma_tree::wallet;
use wasm_bindgen::prelude::*;

use crate::address::Address;

/// Secret key for the prover
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone)]
pub struct SecretKey(wallet::secret_key::SecretKey);

#[wasm_bindgen]
impl SecretKey {
    /// generate random key
    pub fn random_dlog() -> SecretKey {
        SecretKey(wallet::secret_key::SecretKey::random_dlog())
    }

    /// Address (encoded public image)
    pub fn get_address(&self) -> Address {
        self.0.get_address_from_public_image().into()
    }
}

impl From<SecretKey> for wallet::secret_key::SecretKey {
    fn from(s: SecretKey) -> Self {
        s.0
    }
}

impl From<wallet::secret_key::SecretKey> for SecretKey {
    fn from(sk: wallet::secret_key::SecretKey) -> Self {
        SecretKey(sk)
    }
}
