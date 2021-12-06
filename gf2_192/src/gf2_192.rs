//!  By Leonid Reyzin
//!  This is free and unencumbered software released into the public domain.
//!
//!  Anyone is free to copy, modify, publish, use, compile, sell, or
//!  distribute this software, either in source code form or as a compiled
//!  binary, for any purpose, commercial or non-commercial, and by any
//!  means.
//!
//!  In jurisdictions that recognize copyright laws, the author or authors
//!  of this software dedicate any and all copyright interest in the
//!  software to the public domain. We make this dedication for the benefit
//!  of the public at large and to the detriment of our heirs and
//!  successors. We intend this dedication to be an overt act of
//!  relinquishment in perpetuity of all present and future rights to this
//!  software under copyright law.
//!
//!  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
//!  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//!  MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
//!  IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
//!  OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
//!  ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
//!  OTHER DEALINGS IN THE SOFTWARE.
//!
//!  For more information, please refer to <http://unlicense.org>

use crate::lrs;

/// using irreducible polynomial x^192+x^7+x^2+x+1
/// We need only the last word
static IRRED_PENTANOMIAL: i64 = (1i64 << 7) | (1i64 << 2) | (1i64 << 1) | 1i64;

/// IRRED_PENTANOMIAL times 0, 1, x, x+1, x^2, x^2+1, x^2+x, x^2+x+1, x^3, x^3+1, x^3+x, x^3+x+1, x^3+x^2, x^3+x^2+1, x^3+x^2+x, x^3+x^2x+1.
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

    pub fn is_zero(&self) -> bool {
        self.word[0] == 0 && self.word[1] == 0 && self.word[2] == 0
    }

    pub fn is_one(&self) -> bool {
        self.word[0] == 1 && self.word[1] == 0 && self.word[2] == 0
    }

    /// Return a new instance of `GF2_192` which is the sum of `a` and `b`.
    pub fn add(a: &GF2_192, b: &GF2_192) -> GF2_192 {
        let mut word = [0, 0, 0];
        word[0] = a.word[0] ^ b.word[0];
        word[1] = a.word[1] ^ b.word[1];
        word[2] = a.word[2] ^ b.word[2];
        GF2_192 { word }
    }

    /// Computes a times b and puts the result into res.
    /// Uses table lookups, which may not preserve
    /// the secrecy of the inputs in case of side-channel attacks.
    pub fn mul(a: &GF2_192, b: &GF2_192) -> GF2_192 {
        // Implements a sort of times-x-and-add algorithm, except instead of multiplying by x
        // we multiply by x^4 and then add one of possible 16 precomputed values

        // contains a*0, a*1, a*x, a*(x+1), a*x^2, a*(x^2+1), a*(x^2+x), a*(x^2+x+1)
        // a*x^3, a*(x^3+1), a*(x^3+x), a*(x^3+x+1), a*(x^3+x^2), a*(x^3+x^2+1), a*(x^3+x^2+x), a*(x^3+x^2+x+1), all mod reduced
        // First word of each is in a0 muls, second word of each is in a1muls, third word of each is in a2muls
        let mut a0muls: [i64; 16] = Default::default();
        let mut a1muls: [i64; 16] = Default::default();
        let mut a2muls: [i64; 16] = Default::default();

        // a0muls[0], a1muls[0] and a2muls[0] are already correctly initialized to 0

        a0muls[1] = a.word[0];
        a1muls[1] = a.word[1];
        a2muls[1] = a.word[2];

        // a*x, a*x^2, a*x^3
        for i in 2..=8 {
            // multiply a*x^{log_2 i/2} by x to get a*x^{log_2 i}
            let prev = i / 2;
            a0muls[i] = a0muls[prev] << 1;
            a1muls[i] = (a1muls[prev] << 1) | lrs(a0muls[prev], 63);
            a2muls[i] = (a2muls[prev] << 1) | lrs(a1muls[prev], 63);
            // mod reduce
            a0muls[i] ^= IRRED_MULS[lrs(a2muls[prev], 63) as usize];
        }

        // a*(x+1)
        a0muls[3] = a0muls[1] ^ a0muls[2];
        a1muls[3] = a1muls[1] ^ a1muls[2];
        a2muls[3] = a2muls[1] ^ a2muls[2];

        // a*(x^2+1), a*(x^2+x), a*(x^2+x+1)
        for i in 1..4 {
            a0muls[4 | i] = a0muls[4] ^ a0muls[i];
            a1muls[4 | i] = a1muls[4] ^ a1muls[i];
            a2muls[4 | i] = a2muls[4] ^ a2muls[i];
        }

        // a*(x^3+1), a*(x^3+x), a*(x^3+x+1), a*(x^3+x^2), a*(x^3+x^2+1), a*(x^3+x^2+x), a*(x^3+x^2+x+1)
        for i in 1..8 {
            a0muls[8 | i] = a0muls[8] ^ a0muls[i];
            a1muls[8 | i] = a1muls[8] ^ a1muls[i];
            a2muls[8 | i] = a2muls[8] ^ a2muls[i];
        }

        let mut w0 = 0;
        let mut w1 = 0;
        let mut w2 = 0;

        for j in (0..=2).rev() {
            let multiplier = b.word[j];
            for i in (0..=60).rev().step_by(4) {
                // Multiply by x^4
                let mod_reduce_index = lrs(w2, 60) as usize;
                w2 = (w2 << 4) | lrs(w1, 60);
                w1 = (w1 << 4) | lrs(w0, 60);
                // MOD REDUCE ACCORDING TO mod_reduce_index by XORing the right value
                w0 = (w0 << 4) ^ IRRED_MULS[mod_reduce_index];
                //w0 = (w0<<4)^(IRRED_PENTANOMIAL*(mod_reduce_index&8))^(IRRED_PENTANOMIAL*(mod_reduce_index&4))^(IRRED_PENTANOMIAL*(mod_reduce_index&2))^(IRRED_PENTANOMIAL*(mod_reduce_index&1));

                // Add the correct multiple of a
                let index = (lrs(multiplier, i) & 15) as usize;
                w0 ^= a0muls[index];
                w1 ^= a1muls[index];
                w2 ^= a2muls[index];
            }
        }
        GF2_192 { word: [w0, w1, w2] }
    }
}

impl Default for GF2_192 {
    /// Returns the 0 field element
    fn default() -> Self {
        Self::new()
    }
}

impl From<[i64; 3]> for GF2_192 {
    fn from(word: [i64; 3]) -> Self {
        GF2_192 { word }
    }
}

pub struct I8Subslice<'a> {
    slice: &'a [i8],
    pos: usize,
}

impl<'a> TryFrom<I8Subslice<'a>> for GF2_192 {
    type Error = String;

    fn try_from(value: I8Subslice) -> Result<Self, Self::Error> {
        let (that, pos) = (value.slice, value.pos);
        if that.len() < pos + 24 {
            return Err("".into());
        }
        let mut word: [i64; 3] = [0, 0, 0];
        for i in 0..8 {
            word[0] |= (that[i + pos] as i64 & 0xFF) << (i << 3);
        }
        for i in 0..8 {
            word[1] |= (that[i + pos + 8] as i64 & 0xFF) << (i << 3);
        }
        for i in 0..8 {
            word[2] |= (that[i + pos + 16] as i64 & 0xFF) << (i << 3);
        }
        Ok(GF2_192 { word })
    }
}

fn write_to_i8_slice(value: GF2_192, slice: &mut [i8], pos: usize) -> Result<(), String> {
    if slice.len() < pos + 24 {
        return Err("".into());
    }
    for j in 0..3 {
        for i in 0..8 {
            slice[pos + i + 8 * j] = i8::try_from((value.word[j] >> (i << 3)) & 0xFF).unwrap();
        }
    }
    Ok(())
}
