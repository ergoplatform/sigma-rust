use sigma_tree::wallet;
use wasm_bindgen::prelude::*;

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
}

impl From<SecretKey> for wallet::secret_key::SecretKey {
    fn from(s: SecretKey) -> Self {
        s.0
    }
}
