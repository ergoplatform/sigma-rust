//! Mnemonic operations according to BIP32/BIP39

use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;

/// Length of mnemonic seed in bytes
const SHA512_OUTPUT_LEN: usize = 512 / 8;

/// Mnemonic seed
pub type MnemonicSeed = [u8; SHA512_OUTPUT_LEN];

/// Mnemonic type
pub struct Mnemonic();

impl Mnemonic {
    /// Number of iterations specified in BIP39 standard
    pub const PBKDF2_ITERATIONS: u32 = 2048;

    /// Convert a mnemonic phrase into a mnemonic seed
    /// mnemonic_pass is optional and is used to salt the seed
    pub fn to_seed(mnemonic_phrase: &str, mnemonic_pass: &str) -> MnemonicSeed {
        let mut seed: MnemonicSeed = [0u8; SHA512_OUTPUT_LEN];
        pbkdf2::<Hmac<Sha512>>(
            mnemonic_phrase.as_bytes(),
            format!("mnemonic{}", mnemonic_pass).as_bytes(),
            Mnemonic::PBKDF2_ITERATIONS,
            &mut seed,
        );

        seed
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_to_seed() {
        let mnemonic = "change me do not use me change me do not use me";
        let seed = Mnemonic::to_seed(mnemonic, "");
        let encoded_seed = base16::encode_lower(&seed);
        let expected = "c5b2537b52b27b903b34c423783ced17c489e4385ec6d49d6a19a7f892ecd3917db36675de36bcbe3b8dbc6f803877f4155bdf83482ca5f0fc4282a61ac842a3";

        assert_eq!(encoded_seed, expected);
    }

    #[test]
    fn test_mnemonic_to_seed_with_pass() {
        let mnemonic = "change me do not use me change me do not use me";
        let seed = Mnemonic::to_seed(mnemonic, "password123");
        let encoded_seed = base16::encode_lower(&seed);
        let expected = "dfe3088b88e2eb8588482e8c56d9cde497c4e1f63fd29b480cbb0ed0227331d51301cfc2d461acce642868ecb618a37b4fd75d48dc6189674c55fbafd807d69c";

        assert_eq!(encoded_seed, expected);
    }
}
