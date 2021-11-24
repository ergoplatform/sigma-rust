//! Docuuemsn

use std::num::NonZeroU32;

use ring::{digest::SHA512_OUTPUT_LEN, pbkdf2};

type MnemonicSeed = [u8; SHA512_OUTPUT_LEN];

/// a
pub struct Mnemonic();

impl Mnemonic {
    /// Allowed numbers of words in mnemonics
    pub const ALLOWED_SENTENCE_SIZES: [u32; 5] = [12, 15, 18, 21, 24];

    /// f
    pub const ALLOWED_STRENGTHS: [u32; 5] = [128, 160, 192, 224, 256];

    /// f
    pub const ALLOWED_ENTROPHY_LENGTHS: [u32; 0] = [];

    /// x
    pub const BITS_GROUP_SIZE: u32 = 11;

    /// Number of iterations specified in BIP39 standard
    pub const PBKDF2_ITERATIONS: u32 = 2048;

    /// N
    pub const PBKDF2_KEY_LENGTH: u32 = 512;

    /// x
    pub fn to_seed(mnemonic_phrase: &str, mnemonic_pass: &str) -> MnemonicSeed {
        let mut seed: MnemonicSeed = [0u8; SHA512_OUTPUT_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA512,
            NonZeroU32::new(Mnemonic::PBKDF2_ITERATIONS).unwrap(),
            format!("mnemonic{}", mnemonic_pass).as_bytes(),
            mnemonic_phrase.as_bytes(),
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
        let expected: MnemonicSeed = [
            197, 178, 83, 123, 82, 178, 123, 144, 59, 52, 196, 35, 120, 60, 237, 23, 196, 137, 228,
            56, 94, 198, 212, 157, 106, 25, 167, 248, 146, 236, 211, 145, 125, 179, 102, 117, 222,
            54, 188, 190, 59, 141, 188, 111, 128, 56, 119, 244, 21, 91, 223, 131, 72, 44, 165, 240,
            252, 66, 130, 166, 26, 200, 66, 163,
        ];

        assert_eq!(seed, expected);
    }

    #[test]
    fn test_mnemonic_to_seed_with_pass() {
        let mnemonic = "change me do not use me change me do not use me";
        let seed = Mnemonic::to_seed(mnemonic, "password123");
        let expected: MnemonicSeed = [
            223, 227, 8, 139, 136, 226, 235, 133, 136, 72, 46, 140, 86, 217, 205, 228, 151, 196,
            225, 246, 63, 210, 155, 72, 12, 187, 14, 208, 34, 115, 49, 213, 19, 1, 207, 194, 212,
            97, 172, 206, 100, 40, 104, 236, 182, 24, 163, 123, 79, 215, 93, 72, 220, 97, 137, 103,
            76, 85, 251, 175, 216, 7, 214, 156,
        ];

        assert_eq!(seed, expected);
    }
}
