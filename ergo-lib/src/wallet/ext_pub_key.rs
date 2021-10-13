use ergotree_ir::chain::address::Address;

use super::derivation_path::DerivationPath;

pub struct ExtPubKey {
    pub public_key: [u8; 64],
    chain_code: [u8; 32],
    pub derivation_path: DerivationPath,
}

impl ExtPubKey {
    pub fn soft_derive(&self, index: u32) -> Self {
        todo!()
    }
}

impl From<ExtPubKey> for Address {
    fn from(_: ExtPubKey) -> Self {
        todo!()
    }
}
