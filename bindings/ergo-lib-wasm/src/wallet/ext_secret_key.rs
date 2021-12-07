//! Extended secret key operations according to BIP-32

use std::convert::TryInto;

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
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
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
}
