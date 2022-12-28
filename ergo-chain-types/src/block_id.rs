//! Main "remote" type for [BlockId]()

use derive_more::Display;

use crate::DigestNError;

use super::digest32::Digest32;

/// Block id
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash, Display)]
pub struct BlockId(pub Digest32);

impl From<BlockId> for Vec<i8> {
    fn from(value: BlockId) -> Self {
        let BlockId(digest32) = value;
        digest32.into()
    }
}

impl TryFrom<String> for BlockId {
    type Error = DigestNError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}
