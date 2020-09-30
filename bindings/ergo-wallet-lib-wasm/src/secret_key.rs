//! Secret key
use std::convert::TryInto;

use sigma_tree::sigma_protocol::DlogProverInput;
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

    /// Parse dlog secret key from bytes (SEC-1-encoded scalar)
    pub fn dlog_from_bytes(bytes: &[u8]) -> Result<SecretKey, JsValue> {
        let sized_bytes: &[u8; DlogProverInput::SIZE_BYTES] = bytes.try_into().map_err(|_| {
            JsValue::from_str(&format!(
                "expected byte array of size {}, found {}",
                DlogProverInput::SIZE_BYTES,
                bytes.len()
            ))
        })?;
        wallet::secret_key::SecretKey::dlog_from_bytes(sized_bytes)
            .map(SecretKey)
            .ok_or_else(|| JsValue::from_str("failed to parse scalar"))
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
