//! Main "remote" type for [BlockId]()

use super::digest32::Digest32;

/// Block id
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockId(pub Digest32);

impl BlockId {
    /// Returns bytes buffer of `BlockId`
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into()
    }
}
