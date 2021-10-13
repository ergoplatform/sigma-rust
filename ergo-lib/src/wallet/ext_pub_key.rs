use ergotree_ir::chain::address::Address;

use super::derivation_path::DerivationPath;

pub type PubKey = [u8; 64];
pub type ChainCode = [u8; 32];

pub struct ExtPubKey {
    pub public_key: PubKey,
    chain_code: ChainCode,
    pub derivation_path: DerivationPath,
}

impl ExtPubKey {
    pub fn new(public_key: PubKey, chain_code: ChainCode) -> Self {
        todo!()
    }

    pub fn soft_derive(&self, index: u32) -> Self {
        todo!()
    }
}

impl From<ExtPubKey> for Address {
    fn from(_: ExtPubKey) -> Self {
        todo!()
    }
}
