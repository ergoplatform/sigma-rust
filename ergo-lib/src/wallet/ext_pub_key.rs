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
use thiserror::Error;

use super::derivation_path::ChildIndex;
use super::derivation_path::ChildIndexNormal;
use super::derivation_path::DerivationPath;

/// Public key (serialized EcPoint) bytes
pub type PubKeyBytes = [u8; EcPoint::GROUP_SIZE];
/// Chain code bytes
pub type ChainCode = [u8; 32];

type HmacSha512 = Hmac<Sha512>;

/// Extended public key
/// implemented according to BIP-32
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExtPubKey {
    /// Parsed public key (EcPoint)
    pub public_key: EcPoint,
    chain_code: ChainCode,
    /// Derivation path for this extended public key
    pub derivation_path: DerivationPath,
}

/// Extended secret key errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ExtPubKeyError {
    /// Incompatible derivation paths when trying to derive a new key
    #[error("incompatible paths: {0}")]
    IncompatibleDerivation(String),
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

    #[allow(clippy::unwrap_used)]
    fn pub_key_bytes(&self) -> PubKeyBytes {
        // Unwraps are fine here since `self.public_key` is valid through the checking constructor
        // above.
        self.public_key
            .sigma_serialize_bytes()
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap()
    }

    /// Soft derivation of the child public key with a given index
    #[allow(clippy::unwrap_used)]
    pub fn child(&self, index: ChildIndexNormal) -> Self {
        // Unwrap is fine due to `ChainCode` type having fixed length of 32.
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
            let child_pub_key = *child_secret_key.public_image().h * &self.public_key;
            if dlog_group::is_identity(&child_pub_key) {
                // point is infinity element, thus repeat with next index value (see BIP-32)
                self.child(index.next())
            } else {
                let mut chain_code = [0; 32];
                chain_code.copy_from_slice(&mac_bytes[32..]);
                ExtPubKey {
                    public_key: child_pub_key,
                    chain_code,
                    derivation_path: self.derivation_path.extend(index.into()),
                }
            }
        } else {
            // not in range [0, modulus), thus repeat with next index value (BIP-32)
            self.child(index.next())
        }
    }

    /// Derive a new extended pub key based on the provided derivation path
    pub fn derive(&self, up_path: DerivationPath) -> Result<Self, ExtPubKeyError> {
        // TODO: branch visibility must also be equal
        let is_matching_path = up_path.0[..self.derivation_path.depth()]
            .iter()
            .zip(self.derivation_path.0.iter())
            .all(|(a, b)| a == b);

        if up_path.depth() >= self.derivation_path.depth() && is_matching_path {
            up_path.0[self.derivation_path.depth()..]
                .iter()
                .try_fold(self.clone(), |parent, i| match i {
                    ChildIndex::Hardened(_) => Err(ExtPubKeyError::IncompatibleDerivation(
                        format!("pub keys can't use hardened paths: {}", i),
                    )),
                    ChildIndex::Normal(i) => Ok(parent.child(i.clone())),
                })
        } else {
            Err(ExtPubKeyError::IncompatibleDerivation(format!(
                "{}, {}",
                up_path.to_string(),
                self.derivation_path.to_string()
            )))
        }
    }
}

impl From<ExtPubKey> for Address {
    fn from(epk: ExtPubKey) -> Self {
        Address::P2Pk(epk.public_key.into())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::wallet::{
        derivation_path::ChildIndexHardened, ext_secret_key::ExtSecretKey, mnemonic::Mnemonic,
    };

    use super::*;

    #[test]
    fn bip32_test_vector0() {
        // from https://en.bitcoin.it/wiki/BIP_0032_TestVectors
        // Chain m/0' from Test vector 1
        // The difference between path "m/0'" and our "m/44'/429'/0" does not matter
        // since we only testing soft derivation for children
        let derivation_path =
            DerivationPath::new(ChildIndexHardened::from_31_bit(0).unwrap(), vec![]);
        let pub_key_bytes =
            base16::decode(b"035a784662a4a20a65bf6aab9ae98a6c068a81c52e4b032c0fb5400c706cfccc56")
                .unwrap();
        let chain_code =
            base16::decode(b"47fdacbd0f1097043b78c63c20c34ef4ed9a111d980047ad16282c7ae6236141")
                .unwrap();
        let ext_pub_key = ExtPubKey::new(
            pub_key_bytes.try_into().unwrap(),
            chain_code.try_into().unwrap(),
            derivation_path,
        )
        .unwrap();

        // Chain m/0'/1
        let child = ext_pub_key.child(ChildIndexNormal::normal(1).unwrap());
        let expected_child_pub_key_bytes: PubKeyBytes =
            base16::decode(b"03501e454bf00751f24b1b489aa925215d66af2234e3891c3b21a52bedb3cd711c")
                .unwrap()
                .try_into()
                .unwrap();
        assert_eq!(child.pub_key_bytes(), expected_child_pub_key_bytes);
    }

    #[test]
    fn bip32_test_vector1() {
        // from https://en.bitcoin.it/wiki/BIP_0032_TestVectors
        // Chain m/0'/1/2' from Test vector 1
        // The difference between path "m/0'/1/2'" and our "m/44'/429'/0" does not matter
        // since we only testing soft derivation for children
        let derivation_path =
            DerivationPath::new(ChildIndexHardened::from_31_bit(0).unwrap(), vec![]);
        let pub_key_bytes =
            base16::decode(b"0357bfe1e341d01c69fe5654309956cbea516822fba8a601743a012a7896ee8dc2")
                .unwrap();
        let chain_code =
            base16::decode(b"04466b9cc8e161e966409ca52986c584f07e9dc81f735db683c3ff6ec7b1503f")
                .unwrap();
        let ext_pub_key = ExtPubKey::new(
            pub_key_bytes.try_into().unwrap(),
            chain_code.try_into().unwrap(),
            derivation_path,
        )
        .unwrap();

        // Chain m/0'/1/2'/2
        let child = ext_pub_key.child(ChildIndexNormal::normal(2).unwrap());
        let expected_child_pub_key_bytes: PubKeyBytes =
            base16::decode(b"02e8445082a72f29b75ca48748a914df60622a609cacfce8ed0e35804560741d29")
                .unwrap()
                .try_into()
                .unwrap();
        assert_eq!(child.pub_key_bytes(), expected_child_pub_key_bytes);

        // Chain m/0'/1/2'/2/1000000000
        let child2 = child.child(ChildIndexNormal::normal(1000000000).unwrap());
        let expected_child2_pub_key_bytes: PubKeyBytes =
            base16::decode(b"022a471424da5e657499d1ff51cb43c47481a03b1e77f951fe64cec9f5a48f7011")
                .unwrap()
                .try_into()
                .unwrap();
        assert_eq!(child2.pub_key_bytes(), expected_child2_pub_key_bytes);
    }

    #[test]
    fn bip32_test_vector2() {
        // from https://en.bitcoin.it/wiki/BIP_0032_TestVectors
        // Chain m from Test vector 2
        // The difference between path "m" and our "m/44'/429'/0" does not matter
        // since we only testing soft derivation for children
        let derivation_path =
            DerivationPath::new(ChildIndexHardened::from_31_bit(0).unwrap(), vec![]);
        let pub_key_bytes =
            base16::decode(b"03cbcaa9c98c877a26977d00825c956a238e8dddfbd322cce4f74b0b5bd6ace4a7")
                .unwrap();
        let chain_code =
            base16::decode(b"60499f801b896d83179a4374aeb7822aaeaceaa0db1f85ee3e904c4defbd9689")
                .unwrap();
        let ext_pub_key = ExtPubKey::new(
            pub_key_bytes.try_into().unwrap(),
            chain_code.try_into().unwrap(),
            derivation_path,
        )
        .unwrap();

        // Chain m/0
        let child = ext_pub_key.child(ChildIndexNormal::normal(0).unwrap());
        let expected_child_pub_key_bytes: PubKeyBytes =
            base16::decode(b"02fc9e5af0ac8d9b3cecfe2a888e2117ba3d089d8585886c9c826b6b22a98d12ea")
                .unwrap()
                .try_into()
                .unwrap();
        assert_eq!(child.pub_key_bytes(), expected_child_pub_key_bytes);
    }

    #[test]
    fn ergo_node_key_tree_derivation_from_seed() {
        // Tests against the following ergo node test vector:
        // https://github.com/ergoplatform/ergo/blob/c320810c498bca25a44197840c7c5a86440c5906/ergo-wallet/src/test/scala/org/ergoplatform/wallet/secrets/ExtendedPublicKeySpec.scala#L13-L30
        let seed_str = "edge talent poet tortoise trumpet dose";
        let seed = Mnemonic::to_seed(seed_str, "");
        let root_secret = ExtSecretKey::derive_master(seed).unwrap();
        let expected_root = "kTV6HY41wXZVSqdpoe1heA8pBZFEN2oq5T59ZCMpqKKJ";
        let cases: Vec<(&str, ChildIndexNormal)> = vec![
            (
                "uRg1eWWRkhghMxhcZEy2rRjfbc3MqWCJ1oVSP4dNmBAW",
                ChildIndexNormal::normal(1).unwrap(),
            ),
            (
                "xfhJ6aCQUodzhw1J4NcD7iJFvGVc3iPk3pBARCTncYcE",
                ChildIndexNormal::normal(1).unwrap(),
            ),
            (
                "2282dj5QqC7SM7G2ndp4pzaMZwT7vGgUAUZLCKhmXQFxG",
                ChildIndexNormal::normal(1).unwrap(),
            ),
        ];

        let mut ext_pub_key = root_secret.public_key().unwrap();
        let ext_pub_key_b58 = bs58::encode(ext_pub_key.pub_key_bytes()).into_string();

        assert_eq!(expected_root, ext_pub_key_b58);

        for (expected_key, idx) in cases {
            ext_pub_key = ext_pub_key.child(idx);
            let ext_pub_key_b58 = bs58::encode(ext_pub_key.pub_key_bytes()).into_string();

            assert_eq!(expected_key, ext_pub_key_b58);
        }
    }
}
