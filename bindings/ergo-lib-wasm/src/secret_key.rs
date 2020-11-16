//! Secret key
use std::convert::TryInto;

use ergo_lib::sigma_protocol::private_input::DlogProverInput;
use ergo_lib::wallet;
use wasm_bindgen::prelude::*;

use crate::address::Address;

extern crate derive_more;
use derive_more::{From, Into};

/// Secret key for the prover
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone, From, Into)]
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

/// SecretKey collection
#[wasm_bindgen]
pub struct SecretKeys(Vec<SecretKey>);

#[wasm_bindgen]
impl SecretKeys {
    /// Create empty SecretKeys
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        SecretKeys(vec![])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> SecretKey {
        self.0[index].clone()
    }

    /// Adds an elements to the collection
    pub fn add(&mut self, elem: &SecretKey) {
        self.0.push(elem.clone());
    }
}

impl From<&SecretKeys> for Vec<wallet::secret_key::SecretKey> {
    fn from(v: &SecretKeys) -> Self {
        v.0.clone().iter().map(|i| i.0.clone()).collect()
    }
}
impl From<Vec<wallet::secret_key::SecretKey>> for SecretKeys {
    fn from(v: Vec<wallet::secret_key::SecretKey>) -> Self {
        SecretKeys(v.into_iter().map(SecretKey::from).collect())
    }
}
