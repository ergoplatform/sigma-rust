//! Extended public key operations according to BIP-32

use wasm_bindgen::prelude::*;

use super::derivation_path::DerivationPath;
use crate::address::Address;

/// Extented public key
/// implemented according to BIP-32
#[wasm_bindgen]
pub struct ExtPubKey(ergo_lib::wallet::ext_pub_key::ExtPubKey);

#[wasm_bindgen]
impl ExtPubKey {
    /// Create ExtPubKey from public key bytes (from SEC1 compressed), chain code and derivation
    /// path
    pub fn new(
        public_key_bytes: &[u8],
        chain_code: &[u8],
        derivation_path: DerivationPath,
    ) -> Result<ExtPubKey, JsValue> {
        todo!()
    }

    /// Soft derivation of the child public key with a given index
    pub fn derive(&self, index: u32) -> ExtPubKey {
        todo!()
    }

    /// Create address (P2PK) from this extended public key
    pub fn to_address(&self) -> Address {
        todo!()
    }
}
