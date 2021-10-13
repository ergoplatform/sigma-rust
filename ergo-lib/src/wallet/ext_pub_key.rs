//! Extended public key operations according to BIP-32
use std::convert::TryInto;

use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergotree_ir::chain::address::Address;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::dlog_group;
use ergotree_ir::sigma_protocol::dlog_group::EcPoint;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;

use super::derivation_path::ChildIndex;
use super::derivation_path::ChildIndexNormal;
use super::derivation_path::DerivationPath;

/// Public key (serialized EcPoint) bytes
pub type PubKeyBytes = [u8; EcPoint::GROUP_SIZE];
/// Chain code bytes
pub type ChainCode = [u8; 32];

type HmacSha512 = Hmac<Sha512>;

/// Extented public key
/// implemented according to BIP-32
pub struct ExtPubKey {
    /// Parsed public key (EcPoint)
    pub public_key: EcPoint,
    chain_code: ChainCode,
    /// Derivation path for this extended public key
    pub derivation_path: DerivationPath,
}

impl ExtPubKey {
    /// Create ExtPubKey from public key bytes (from SEC1 compressed), chain code and derivation
    /// path
    pub fn new(
        public_key_bytes: PubKeyBytes,
        chain_code: ChainCode,
        derivation_path: DerivationPath,
    ) -> Result<Self, SigmaParsingError> {
        let public_key = EcPoint::sigma_parse_bytes(&public_key_bytes)?;
        Ok(Self {
            public_key,
            chain_code,
            derivation_path,
        })
    }

    fn pub_key_bytes(&self) -> PubKeyBytes {
        self.public_key
            .sigma_serialize_bytes()
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap()
    }

    /// Soft derivation of the child public key with a given index
    pub fn derive(&self, index: ChildIndexNormal) -> Self {
        let mut mac = HmacSha512::new_from_slice(&self.chain_code).unwrap();
        mac.update(&self.pub_key_bytes());
        mac.update(
            ChildIndex::Normal(index.clone())
                .to_bits()
                .to_be_bytes()
                .as_ref(),
        );
        let mac_bytes = mac.finalize().into_bytes();
        let mut secret_key_bytes = [0; 32];
        secret_key_bytes.copy_from_slice(&mac_bytes[..32]);
        if let Some(child_secret_key) = DlogProverInput::from_bytes(&secret_key_bytes) {
            let mut chain_code = [0; 32];
            chain_code.copy_from_slice(&mac_bytes[32..]);
            let child_pub_key = *child_secret_key.public_image().h * &self.public_key;
            if dlog_group::is_identity(&child_pub_key) {
                // point is infinity element, thus repeat with next index value (see BIP-32)
                self.derive(index.next())
            } else {
                ExtPubKey {
                    public_key: child_pub_key,
                    chain_code,
                    derivation_path: self.derivation_path.extend(index),
                }
            }
        } else {
            // not in range [0, modulus), thus repeat with next index value (BIP-32)
            self.derive(index.next())
        }
    }
}

impl From<ExtPubKey> for Address {
    fn from(epk: ExtPubKey) -> Self {
        Address::P2Pk(epk.public_key.into())
    }
}
