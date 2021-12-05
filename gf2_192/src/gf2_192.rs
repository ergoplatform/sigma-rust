/// using irreducible polynomial x^192+x^7+x^2+x+1
/// We need only the last word
static IRRED_PENTANOMIAL: i64 = (1i64 << 7) | (1i64 << 2) | (1i64 << 1) | 1i64;

/// IRRED_PENTANOMIAL times 0, 1, x, x+1, x^2, x^2+1, x^2+x, x^2+x+1, x^3, x^3+1, x^3+x, x^3+x+1, x^3+x^2, x^3+x^2+1, x^3+x^2+x, x^3+x^2x+1,
/// Need only the last word, because the leading two words are 0
static IRRED_MULS: [i64; 16] = [
    0i64,
    IRRED_PENTANOMIAL,
    IRRED_PENTANOMIAL << 1,
    (IRRED_PENTANOMIAL << 1) ^ IRRED_PENTANOMIAL,
    IRRED_PENTANOMIAL << 2,
    (IRRED_PENTANOMIAL << 2) ^ IRRED_PENTANOMIAL,
    (IRRED_PENTANOMIAL << 2) ^ (IRRED_PENTANOMIAL << 1),
    (IRRED_PENTANOMIAL << 2) ^ (IRRED_PENTANOMIAL << 1) ^ IRRED_PENTANOMIAL,
    IRRED_PENTANOMIAL << 3,
    (IRRED_PENTANOMIAL << 3) ^ IRRED_PENTANOMIAL,
    (IRRED_PENTANOMIAL << 3) ^ (IRRED_PENTANOMIAL << 1),
    (IRRED_PENTANOMIAL << 3) ^ (IRRED_PENTANOMIAL << 1) ^ IRRED_PENTANOMIAL,
    (IRRED_PENTANOMIAL << 3) ^ (IRRED_PENTANOMIAL << 2),
    (IRRED_PENTANOMIAL << 3) ^ (IRRED_PENTANOMIAL << 2) ^ IRRED_PENTANOMIAL,
    (IRRED_PENTANOMIAL << 3) ^ (IRRED_PENTANOMIAL << 2) ^ (IRRED_PENTANOMIAL << 1),
    (IRRED_PENTANOMIAL << 3)
        ^ (IRRED_PENTANOMIAL << 2)
        ^ (IRRED_PENTANOMIAL << 1)
        ^ IRRED_PENTANOMIAL,
];

/// Represents an element of the Galois field GF(2^192)
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct GF2_192 {
    word: [i64; 3],
}

impl GF2_192 {
    /// Returns the 0 field element
    pub fn new() -> Self {
        GF2_192 { word: [0, 0, 0] }
    }
}

impl Default for GF2_192 {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[i64; 3]> for GF2_192 {
    fn from(word: [i64; 3]) -> Self {
        GF2_192 { word }
    }
}
