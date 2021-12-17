//! Extended secret key operations according to BIP-32

use std::convert::TryInto;

use ergo_lib::wallet::derivation_path::ChildIndex;
use ergo_lib::wallet::ext_pub_key::ChainCode;
use ergo_lib::wallet::ext_secret_key::ExtSecretKey as InnerExtSecretKey;
use ergo_lib::wallet::ext_secret_key::SecretKeyBytes;
use wasm_bindgen::prelude::*;

use super::derivation_path::DerivationPath;
use crate::error_conversion::to_js;

extern crate derive_more;
use derive_more::{From, Into};

/// Extented secret key implemented according to BIP-32
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone, From, Into)]
pub struct ExtSecretKey(InnerExtSecretKey);

#[wasm_bindgen]
impl ExtSecretKey {
    /// Create ExtSecretKey from secret key bytes, chain code and derivation path
    pub fn new(
        secret_key_bytes: &[u8],
        chain_code: &[u8],
        derivation_path: &DerivationPath,
    ) -> Result<ExtSecretKey, JsValue> {
        let secret_key_bytes: SecretKeyBytes = secret_key_bytes.try_into().map_err(to_js)?;
        let chain_code: ChainCode = chain_code.try_into().map_err(to_js)?;
        Ok(ExtSecretKey(
            InnerExtSecretKey::new(secret_key_bytes, chain_code, derivation_path.clone().into())
                .map_err(to_js)?,
        ))
    }

    /// Derive root extended secret key
    pub fn derive_master(seed_bytes: &[u8]) -> Result<ExtSecretKey, JsValue> {
        let seed = seed_bytes.try_into().map_err(to_js)?;
        Ok(InnerExtSecretKey::derive_master(seed)
            .map_err(to_js)?
            .into())
    }

    /// Derive a new extended secret key from the provided index
    /// The index is in the form of soft or hardened indices
    /// For example: 4 or 4' respectively
    pub fn child(&self, index: &str) -> Result<ExtSecretKey, JsValue> {
        let idx = index.parse::<ChildIndex>().map_err(to_js)?;
        Ok(self.0.child(idx).map_err(to_js)?.into())
    }

    /// Derive a new extended secret key from the derivation path
    pub fn derive(&self, path: DerivationPath) -> Result<ExtSecretKey, JsValue> {
        Ok(self.0.derive(path.into()).map_err(to_js)?.into())
    }

    /// The extended public key associated with this secret key
    pub fn public_key(&self) -> Result<crate::wallet::ext_pub_key::ExtPubKey, JsValue> {
        Ok(self.0.public_key().map_err(to_js)?.into())
    }
}
