//! Main "remote" type for [TxId]()

use super::digest::Digest32;

/// Modifier id
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ModifierId(pub Digest32);
