//! Extended private key operations according to BIP-32
use std::convert::TryInto;

use super::{
    derivation_path::{ChildIndex, ChildIndexError, DerivationPath},
    ext_pub_key::ExtPubKey,
    mnemonic::MnemonicSeed,
};
use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergotree_ir::{
    serialization::{SigmaParsingError, SigmaSerializable, SigmaSerializationError},
    sigma_protocol::{
        dlog_group::{self, EcPoint},
        sigma_boolean::ProveDlog,
    },
};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;
use thiserror::Error;

/// Public key (serialized EcPoint) bytes
pub type SecretKeyBytes = [u8; 32];
/// Chain code bytes
pub type ChainCode = [u8; 32];

type HmacSha512 = Hmac<Sha512>;

/// Result type for ExtendedSecretKey operations
pub type Result<T> = std::result::Result<T, ExtSecretKeyError>;

/// Extended secret key
/// implemented according to BIP-32
#[derive(PartialEq, Debug, Clone)]
pub struct ExtSecretKey {
    secret_key: EcPoint,
    chain_code: ChainCode,
    derivation_path: DerivationPath,
    private_input: DlogProverInput,
}

/// Extended secret key errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum ExtSecretKeyError {
    /// Parsing error
    #[error("parsing error: {0}")]
    SigmaParsingError(#[from] SigmaParsingError),
    #[error("serialization error: {0}")]
    /// Serializing error
    SigmaSerializationError(#[from] SigmaSerializationError),
    /// Error encoding bytes as SEC-1-encoded scalar
    #[error("scalar encoding error")]
    ScalarEncodingError,
    /// Derivation path child index error
    /// For example trying to use a u32 value for a private index (31 bit size)
    #[error("child index error: {0}")]
    ChildIndexError(#[from] ChildIndexError),
}

impl ExtSecretKey {
    const BITCOIN_SEED: &'static [u8; 12] = b"Bitcoin seed";

    /// Create a new extended secret key instance
    pub fn new(
        secret_key_bytes: SecretKeyBytes,
        chain_code: ChainCode,
        derivation_path: DerivationPath,
    ) -> Result<Self> {
        let secret_key = EcPoint::sigma_parse_bytes(&secret_key_bytes)?;
        let private_input = DlogProverInput::from_bytes(&secret_key_bytes)
            .ok_or(ExtSecretKeyError::ScalarEncodingError)?;
        Ok(Self {
            secret_key,
            chain_code,
            derivation_path,
            private_input,
        })
    }

    #[allow(clippy::unwrap_used)]
    fn secret_key_bytes(&self) -> SecretKeyBytes {
        // Unwraps are fine here since `self.public_key` is valid through the checking constructor
        // above.
        self.secret_key
            .sigma_serialize_bytes()
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap()
    }

    /// Public image associated with the private input
    pub fn public_image(&self) -> ProveDlog {
        self.private_input.public_image()
    }

    /// Public image bytes in SEC-1 encoded & compressed format
    pub fn public_image_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.public_image().h.sigma_serialize_bytes()?)
    }

    /// The extended public key associated with this secret key
    #[allow(clippy::unwrap_used)]
    pub fn public_key(&self) -> Result<ExtPubKey> {
        Ok(ExtPubKey::new(
            // unwrap is safe as it is used on an Infallible result type
            self.public_image_bytes()?.try_into().unwrap(),
            self.chain_code,
            self.derivation_path.clone(),
        )?)
    }

    /// Derive a child extended secret key using the provided index
    #[allow(clippy::unwrap_used)]
    pub fn derive(&self, index: ChildIndex) -> Result<ExtSecretKey> {
        // Unwrap is fine due to `ChainCode` type having fixed length of 32.
        let mut mac = HmacSha512::new_from_slice(&self.chain_code).unwrap();
        match index {
            ChildIndex::Hardened(_) => {
                mac.update(&[0u8]);
                mac.update(&self.secret_key_bytes());
            }
            ChildIndex::Normal(_) => mac.update(&self.public_image_bytes()?),
        }
        mac.update(&index.to_bits().to_be_bytes());
        let mac_bytes = mac.finalize().into_bytes();
        let mut secret_key_bytes = [0; 32];
        secret_key_bytes.copy_from_slice(&mac_bytes[..32]);
        if let Some(_) = DlogProverInput::from_bytes(&secret_key_bytes) {
            let secret_key = EcPoint::sigma_parse_bytes(&secret_key_bytes)?;
            if dlog_group::is_identity(&secret_key) {
                // point is infinity element, thus repeat with next index value (see BIP-32)
                self.derive(index.next()?)
            } else {
                let mut chain_code = [0; 32];
                chain_code.copy_from_slice(&mac_bytes[32..]);
                ExtSecretKey::new(
                    secret_key_bytes,
                    chain_code,
                    self.derivation_path.extend(index),
                )
            }
        } else {
            // not in range [0, modulus), thus repeat with next index value (BIP-32)
            self.derive(index.next()?)
        }
    }

    /// Derive a root master key from the provided mnemonic seed
    #[allow(clippy::unwrap_used)]
    pub fn derive_master(seed: MnemonicSeed) -> Result<ExtSecretKey> {
        // Unwrap is safe, we are using a valid static length slice
        let mut mac = HmacSha512::new_from_slice(ExtSecretKey::BITCOIN_SEED).unwrap();
        mac.update(&seed);
        let hash = mac.finalize().into_bytes();
        let mut secret_key_bytes = [0; 32];
        secret_key_bytes.copy_from_slice(&hash[..32]);
        let mut chain_code = [0; 32];
        chain_code.copy_from_slice(&hash[32..]);

        // TODO: can we construct the struct directly and remove Result<>
        ExtSecretKey::new(secret_key_bytes, chain_code, DerivationPath::master_path())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn bip32_test_vector0_root() {
        let secret_key =
            base16::decode(b"e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35")
                .unwrap();
        let chain_code =
            base16::decode(b"873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508")
                .unwrap();
        let ext_secret_key = ExtSecretKey::new(
            secret_key.try_into().unwrap(),
            chain_code.try_into().unwrap(),
            DerivationPath::master_path(),
        )
        .unwrap();
        let expected_bytes: SecretKeyBytes = base16::decode(b"0488ade4000000000000000000873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d50800e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35").unwrap().try_into().unwrap();

        assert_eq!(ext_secret_key.secret_key_bytes(), expected_bytes)
    }
}
