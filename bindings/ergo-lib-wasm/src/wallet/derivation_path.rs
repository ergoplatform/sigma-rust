//! Derivation path according to
//! BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
//! and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>

use derive_more::FromStr;
use ergo_lib::wallet::derivation_path::ChildIndexError;
use ergo_lib::wallet::derivation_path::ChildIndexHardened;
use ergo_lib::wallet::derivation_path::ChildIndexNormal;
use ergo_lib::wallet::derivation_path::DerivationPath as InnerDerivationPath;
use wasm_bindgen::prelude::*;

use crate::error_conversion::to_js;

extern crate derive_more;
use derive_more::{From, Into};

/// According to
/// BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
/// and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into, FromStr)]
pub struct DerivationPath(InnerDerivationPath);

#[wasm_bindgen]
impl DerivationPath {
    /// Create derivation path for a given account index (hardened) and address indices
    /// `m / 44' / 429' / acc' / 0 / address[0] / address[1] / ...`
    /// or `m / 44' / 429' / acc' / 0` if address indices are empty
    /// change is always zero according to EIP-3
    /// acc is expected as a 31-bit value (32th bit should not be set)
    pub fn new(acc: u32, address_indices: &[u32]) -> Result<DerivationPath, JsValue> {
        let acc = ChildIndexHardened::from_31_bit(acc).map_err(to_js)?;
        let address_indices = address_indices
            .iter()
            .map(|i| ChildIndexNormal::normal(*i))
            .collect::<Result<Vec<ChildIndexNormal>, ChildIndexError>>()
            .map_err(to_js)?;
        Ok(DerivationPath(InnerDerivationPath::new(
            acc,
            address_indices,
        )))
    }

    /// For 0x21 Sign Transaction command of Ergo Ledger App Protocol
    /// P2PK Sign (0x0D) instruction
    /// Sign calculated TX hash with private key for provided BIP44 path.
    /// Data:
    ///
    /// Field
    /// Size (B)
    /// Description
    ///
    /// BIP32 path length
    /// 1
    /// Value: 0x02-0x0A (2-10). Number of path components
    ///
    /// First derivation index
    /// 4
    /// Big-endian. Value: 44’
    ///
    /// Second derivation index
    /// 4
    /// Big-endian. Value: 429’ (Ergo coin id)
    ///
    /// Optional Third index
    /// 4
    /// Big-endian. Any valid bip44 hardened value.
    /// ...
    /// Optional Last index
    /// 4
    /// Big-endian. Any valid bip44 value.
    ///
    pub fn ledger_bytes(&self) -> Vec<u8> {
        self.0.ledger_bytes()
    }
}
