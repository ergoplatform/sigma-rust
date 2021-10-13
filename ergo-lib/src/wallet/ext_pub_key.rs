use ergotree_ir::chain::address::Address;

use super::derivation_path::ChildIndexNormal;
use super::derivation_path::DerivationPath;

pub type PubKey = [u8; 65];
pub type ChainCode = [u8; 32];

pub struct ExtPubKey {
    pub public_key: PubKey,
    chain_code: ChainCode,
    pub derivation_path: DerivationPath,
}

impl ExtPubKey {
    pub fn new(public_key: PubKey, chain_code: ChainCode, derivation_path: DerivationPath) -> Self {
        todo!()
    }

    pub fn derive(&self, index: ChildIndexNormal) -> Self {
        todo!()
    }
}

impl From<ExtPubKey> for Address {
    fn from(_: ExtPubKey) -> Self {
        todo!()
    }
}
