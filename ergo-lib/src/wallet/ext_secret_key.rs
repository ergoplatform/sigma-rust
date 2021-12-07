//! Extended private key operations according to BIP-32

use super::{derivation_path::DerivationPath, ext_pub_key::ExtPubKey, mnemonic::MnemonicSeed};
use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergotree_ir::{
    serialization::{SigmaParsingError, SigmaSerializable},
    sigma_protocol::{dlog_group::EcPoint, sigma_boolean::ProveDlog},
};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;
use thiserror::Error;

/// Public key (serialized EcPoint) bytes
pub type SecretKeyBytes = [u8; 32];
/// Chain code bytes
pub type ChainCode = [u8; 32];

type HmacSha512 = Hmac<Sha512>;

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
    #[error("scalar encoding error")]
    ScalarEncodingError,
}

impl ExtSecretKey {
    const BITCOIN_SEED: &'static [u8; 12] = b"Bitcoin seed";

    /// Create a new extended secret key instance
    pub fn new(
        secret_key_bytes: SecretKeyBytes,
        chain_code: ChainCode,
        derivation_path: DerivationPath,
    ) -> Result<Self, ExtSecretKeyError> {
        let secret_key = EcPoint::sigma_parse_bytes(&secret_key_bytes)?;
        let private_input = DlogProverInput::from_bytes(&secret_key_bytes)
            .ok_or_else(|| ExtSecretKeyError::ScalarEncodingError)?;
        Ok(Self {
            secret_key,
            chain_code,
            derivation_path,
            private_input,
        })
    }

    pub fn public_image(&self) -> ProveDlog {
        self.private_input.public_image()
    }

    /// Derive a root master key from the provided mnemonic seed
    #[allow(clippy::unwrap_used)]
    pub fn derive_master(seed: MnemonicSeed) -> Result<ExtSecretKey, ExtSecretKeyError> {
        // Unwrap is safe, we are using a valid fixed length slice
        let mut mac = HmacSha512::new_from_slice(ExtSecretKey::BITCOIN_SEED).unwrap();
        mac.update(&seed);
        let hash = mac.finalize().into_bytes();
        let mut secret_key_bytes = [0; 32];
        secret_key_bytes.copy_from_slice(&hash[..32]);
        let mut chain_code = [0; 32];
        chain_code.copy_from_slice(&hash[32..]);

        ExtSecretKey::new(secret_key_bytes, chain_code, DerivationPath::master_path())
    }
}
