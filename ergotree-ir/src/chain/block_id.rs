//! Main "remote" type for [BlockId]()

use super::digest::Digest32;

/// Block id
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockId(pub Digest32);
