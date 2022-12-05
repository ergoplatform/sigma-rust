//! Main "remote" type for [BlockId]()

use derive_more::Display;

use super::digest32::Digest32;

/// Block id
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash, Display)]
pub struct BlockId(pub Digest32);

impl From<BlockId> for Vec<i8> {
    fn from(value: BlockId) -> Self {
        let BlockId(digest32) = value;
        digest32.into()
    }
}
