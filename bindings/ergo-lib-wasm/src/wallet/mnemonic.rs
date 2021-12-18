//! Mnemonic operations according to BIP32/BIP39

extern crate derive_more;
use derive_more::From;
use ergo_lib::wallet::mnemonic::Mnemonic as InnerMnemonic;
use wasm_bindgen::prelude::wasm_bindgen;

/// Mnemonic
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone, From)]
pub struct Mnemonic {}

#[wasm_bindgen]
impl Mnemonic {
    /// Convert a mnemonic phrase into a mnemonic seed
    /// mnemonic_pass is optional and is used to salt the seed
    pub fn to_seed(mnemonic_phrase: &str, mnemonic_pass: &str) -> Vec<u8> {
        InnerMnemonic::to_seed(mnemonic_phrase, mnemonic_pass).into()
    }
}
