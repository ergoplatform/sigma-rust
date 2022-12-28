//! Secret key
use std::convert::TryInto;

use ergo_lib::ergo_chain_types::EcPoint;
use ergo_lib::ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergo_lib::wallet;
use wasm_bindgen::prelude::*;

use crate::address::Address;

extern crate derive_more;
use derive_more::{From, Into};

/// Secret key for the prover
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
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

    /// Parse Diffie-Hellman tuple secret key from bytes.
    /// secret is expected as SEC-1-encoded scalar of 32 bytes,
    /// g,h,u,v are expected as 33-byte compressed points
    pub fn dht_from_bytes(
        secret: &[u8],
        g: &[u8],
        h: &[u8],
        u: &[u8],
        v: &[u8],
    ) -> Result<SecretKey, JsValue> {
        let sized_secret: &[u8; DlogProverInput::SIZE_BYTES] = secret.try_into().map_err(|_| {
            JsValue::from_str(&format!(
                "expected secret byte array of size {}, found {}",
                DlogProverInput::SIZE_BYTES,
                secret.len()
            ))
        })?;
        let sized_g: &[u8; EcPoint::GROUP_SIZE] = g.try_into().map_err(|_| {
            JsValue::from_str(&format!(
                "expected g byte array of size {}, found {}",
                EcPoint::GROUP_SIZE,
                g.len()
            ))
        })?;
        let sized_h: &[u8; EcPoint::GROUP_SIZE] = h.try_into().map_err(|_| {
            JsValue::from_str(&format!(
                "expected h byte array of size {}, found {}",
                EcPoint::GROUP_SIZE,
                h.len()
            ))
        })?;
        let sized_u: &[u8; EcPoint::GROUP_SIZE] = u.try_into().map_err(|_| {
            JsValue::from_str(&format!(
                "expected u byte array of size {}, found {}",
                EcPoint::GROUP_SIZE,
                u.len()
            ))
        })?;
        let sized_v: &[u8; EcPoint::GROUP_SIZE] = v.try_into().map_err(|_| {
            JsValue::from_str(&format!(
                "expected v byte array of size {}, found {}",
                EcPoint::GROUP_SIZE,
                v.len()
            ))
        })?;
        wallet::secret_key::SecretKey::dht_from_bytes_fields(
            sized_secret,
            sized_g,
            sized_h,
            sized_u,
            sized_v,
        )
        .map(SecretKey::from)
        .ok_or_else(|| JsValue::from_str("failed to parse Diffie-Hellman tuple"))
    }

    /// Address (encoded public image)
    pub fn get_address(&self) -> Address {
        self.0.get_address_from_public_image().into()
    }

    /// Parse secret key from bytes (expected 32 bytes for Dlog, 32(secret)+33(g)+33(h)+33(u)+33(v)=164 bytes for DHT)
    /// secret is expected as SEC-1-encoded scalar of 32 bytes,
    /// g,h,u,v are expected as 33-byte compressed points
    pub fn from_bytes(bytes: &[u8]) -> Result<SecretKey, JsValue> {
        wallet::secret_key::SecretKey::from_bytes(bytes)
            .map(SecretKey)
            .map_err(|e| JsValue::from_str(&format!("failed to parse SecretKey from bytes: {}", e)))
    }

    /// Serialized secret key (32 bytes for Dlog, 32(secret)+33(g)+33(h)+33(u)+33(v)=164 bytes for DHT)
    /// DHT format is the same as in from_bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    /// Parse secret key from JSON string (Dlog expected as base16-encoded bytes, DHT in node REST API format)
    pub fn from_json(json_str: &str) -> Result<SecretKey, JsValue> {
        serde_json::from_str(json_str)
            .map(SecretKey)
            .map_err(|e| JsValue::from_str(&format!("failed to parse SecretKey from JSON: {}", e)))
    }

    /// Encode secret key to JSON string (Dlog as base16-encoded bytes, DHT in node REST API format)
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.0)
            .map_err(|e| JsValue::from_str(&format!("failed to encode SecretKey to JSON: {}", e)))
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
