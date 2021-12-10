//! Extended public key operations according to BIP-32

use std::convert::TryInto;

use ergo_lib::wallet::derivation_path::ChildIndexNormal;
use ergo_lib::wallet::ext_pub_key::ChainCode;
use ergo_lib::wallet::ext_pub_key::ExtPubKey as InnerExtPubKey;
use ergo_lib::wallet::ext_pub_key::PubKeyBytes;
use wasm_bindgen::prelude::*;

use super::derivation_path::DerivationPath;
use crate::address::Address;
use crate::error_conversion::to_js;

extern crate derive_more;
use derive_more::{From, Into};

/// Extented public key implemented according to BIP-32
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct ExtPubKey(InnerExtPubKey);

#[wasm_bindgen]
impl ExtPubKey {
    /// Create ExtPubKey from public key bytes (from SEC1 compressed), chain code and derivation
    /// path
    pub fn new(
        public_key_bytes: &[u8],
        chain_code: &[u8],
        derivation_path: &DerivationPath,
    ) -> Result<ExtPubKey, JsValue> {
        let public_key_bytes: PubKeyBytes = public_key_bytes.try_into().map_err(to_js)?;
        let chain_code: ChainCode = chain_code.try_into().map_err(to_js)?;
        Ok(ExtPubKey(
            InnerExtPubKey::new(public_key_bytes, chain_code, derivation_path.clone().into())
                .map_err(to_js)?,
        ))
    }

    /// Soft derivation of the child public key with a given index
    /// index is expected to be a 31-bit value(32th bit should not be set)
    pub fn child(&self, index: u32) -> Result<ExtPubKey, JsValue> {
        let index = ChildIndexNormal::normal(index).map_err(to_js)?;
        Ok(self.0.child(index).into())
    }

    /// Derive a new extended pub key from the derivation path
    pub fn derive(&self, path: DerivationPath) -> Result<ExtPubKey, JsValue> {
        Ok(self.0.derive(path.into()).map_err(to_js)?.into())
    }

    /// Create address (P2PK) from this extended public key
    pub fn to_address(&self) -> Address {
        let address: ergo_lib::ergotree_ir::chain::address::Address = self.0.clone().into();
        address.into()
    }
}
