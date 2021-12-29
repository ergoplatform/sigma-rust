use ergotree_ir::chain::header::Header;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::autolykos_pow_scheme::AutolykosPowScheme;

/// A set of utilities for working with NiPoPoW protocol.
///
/// Based on papers:
///
/// [KMZ17] Non-Interactive Proofs of Proof-of-Work, FC 20 (published) version
///           <https://fc20.ifca.ai/preproceedings/74.pdf>
///
/// [KLS16] Proofs of Proofs of Work with Sublinear Complexity <http://fc16.ifca.ai/bitcoin/papers/KLS16.pdf>
///
/// Please note that for [KMZ17] we're using the version published @ Financial Cryptography 2020, which is different
/// from previously published versions on IACR eprint.
#[derive(Default)]
pub(crate) struct NipopowAlgos {
    /// The proof-of-work scheme
    pow_scheme: AutolykosPowScheme,
}

impl NipopowAlgos {
    /// Computes best score of a given chain.
    /// The score value depends on number of µ-superblocks in the given chain.
    ///
    /// see [KMZ17], Algorithm 4
    ///
    /// [KMZ17]:
    /// "To find the best argument of a proof π given b, best-arg_m collects all the μ
    /// indices which point to superblock levels that contain valid arguments after block b.
    /// Argument validity requires that there are at least m μ-superblocks following block b,
    /// which is captured by the comparison|π↑μ{b:}|≥m. 0 is always considered a valid level,
    /// regardless of how many blocks are present there. These level indices are collected into set M.
    /// For each of these levels, the score of their respective argument is evaluated by weighting the
    /// number of blocks by the level as 2μ|π↑μ{b:}|. The highest possible score across all levels is returned."
    ///
    /// function best-arg_m(π, b)
    /// M←{μ:|π↑μ{b:}|≥m}∪{0}
    /// return max_{μ∈M} {2μ·|π↑μ{b:}|}
    /// end function
    pub(crate) fn best_arg(&self, chain: &[&Header], m: u32) -> usize {
        // Little helper struct for loop below
        struct Acc {
            level: u32,
            acc: Vec<(u32, usize)>,
        }
        let mut res = Acc {
            level: 1,
            acc: vec![(0, chain.len())],
        };
        let acc = loop {
            let args: Vec<_> = chain
                .iter()
                .filter(|h| (self.max_level_of(h) as u32) >= res.level)
                .collect();
            if args.len() >= (m as usize) {
                res.acc.push((res.level, args.len()));
                res = Acc {
                    level: res.level + 1,
                    acc: res.acc,
                };
            } else {
                break res.acc;
            }
        };
        #[allow(clippy::unwrap_used)]
        acc.into_iter()
            .map(|(level, size)| {
                // 2^µ * |C↑µ|
                2usize.pow(level) * size
            })
            .max()
            .unwrap()
    }

    /// Computes max level (μ) of the given header, such that μ = log(T) − log(id(B))
    pub(crate) fn max_level_of(&self, header: &Header) -> i32 {
        let genesis_header = header.height == 1;
        if !genesis_header {
            // Order of the secp256k1 elliptic curve
            #[allow(clippy::unwrap_used)]
            let order = BigInt::parse_bytes(
                b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
                16,
            )
            .unwrap();
            #[allow(clippy::unwrap_used)]
            let required_target = (order / decode_compact_bits(header.n_bits))
                .to_f64()
                .unwrap();
            #[allow(clippy::unwrap_used)]
            let real_target = self.pow_scheme.pow_hit(header).unwrap().to_f64().unwrap();
            let level = required_target.log2() - real_target.log2();
            level as i32
        } else {
            i32::MAX
        }
    }

    /// Finds the last common header (branching point) between `left_chain` and `right_chain`.
    pub(crate) fn lowest_common_ancestor(
        &self,
        left_chain: &[&Header],
        right_chain: &[&Header],
    ) -> Option<Header> {
        if let Some(head_left) = left_chain.first() {
            if let Some(head_right) = right_chain.first() {
                if *head_left != *head_right {
                    return None;
                }
            }
        }
        let mut common = vec![];
        let mut right_ix_start = 0;
        for left_header in left_chain {
            let start_ix = right_ix_start;
            for (i, right_header) in right_chain.iter().enumerate().skip(start_ix) {
                if **left_header == **right_header {
                    right_ix_start = i + 1;
                    common.push(*left_header);
                }
            }
        }
        common.last().cloned().cloned()
    }
}

/// The "compact" format is an encoding of a whole number `N` using an unsigned 32 bit number.
/// This number encodes a base-256 scientific notation representation of `N` (similar to a floating
/// point format):
///  - The most significant 8 bits represent the number of bytes necessary to represent `N` in
///    two's-complement form; denote it by `exponent`.
///  - The lower 23 bits are the mantissa(significand).
///  - Bit number 24 (0x800000) represents the sign of N.
///
/// There are 2 cases to consider:
///  - If `exponent >= 3` then `N` is represented by
///      `(-1^sign) * mantissa * 256^(exponent-3)`
///    E.g. suppose the compact form is given in hex-format by `0x04123456`. Mantissa is `0x123456`
///    an `exponent == 4`. So `N == 0x123456 * 265^1`. Now note that we need exactly 4 bytes to
///    represent `N`; 3 bytes for the mantissa and 1 byte for the rest. In base-256:
///      `N == B(0x12)B(0x34)B(0x56)0`
///    where `B(y)` denotes the base-256 representation of a hex-number `y` (note how each base-256
///    digit is represented by a single-byte).
///  - If `exponent < 3` then `N` is represented by the `exponent`-most-significant-bytes of the
///    mantissa. E.g. suppose the compact form is given in hex-format by `0x01003456`. Noting that
///    each hex-digit is represented by 4-bits, our `exponent == 0x01` which is `1` base-10.  The
///    mantissa is represented by `0x003456` and it's most signficant byte is `0x00`. Therefore
///    `N == 0`.
///
/// Satoshi's original implementation used BN_bn2mpi() and BN_mpi2bn(). MPI uses the most
/// significant bit of the first byte as sign. Thus 0x1234560000 is compact 0x05123456 and
/// 0xc0de000000 is compact 0x0600c0de. Compact 0x05c0de00 would be -0x40de000000.
///
/// Bitcoin only uses this "compact" format for encoding difficulty targets, which are unsigned
/// 256bit quantities.  Thus, all the complexities of the sign bit and using base 256 are probably
/// an implementation accident.
fn decode_compact_bits(n_bits: u64) -> BigInt {
    let compact = n_bits as i64;
    let size = ((compact >> 24) as i32) & 0xFF;
    if size == 0 {
        return BigInt::from(0);
    }
    let mut buf: Vec<i8> = std::iter::repeat(0).take(size as usize).collect();
    if size >= 1 {
        // Store the first byte of the mantissa
        buf[0] = (((compact >> 16) as i32) & 0xFF) as i8;
    }
    if size >= 2 {
        buf[1] = (((compact >> 8) as i32) & 0xFF) as i8;
    }
    if size >= 3 {
        buf[2] = ((compact as i32) & 0xFF) as i8;
    }

    let is_negative = (buf[0] as i32) & 0x80 == 0x80;
    if is_negative {
        buf[0] &= 0x7f;
        let buf: Vec<_> = buf.into_iter().map(|x| x as u8).collect();
        -BigInt::from_signed_bytes_be(&buf)
    } else {
        let buf: Vec<_> = buf.into_iter().map(|x| x as u8).collect();
        BigInt::from_signed_bytes_be(&buf)
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::ToBigInt;

    #[test]
    fn test_decode_n_bits() {
        // Following example taken from https://btcinformation.org/en/developer-reference#target-nbits
        let n_bits = 0x181bc330;
        assert_eq!(
            decode_compact_bits(n_bits),
            BigInt::parse_bytes(b"1bc330000000000000000000000000000000000000000000", 16).unwrap()
        );

        let n_bits = 0x01003456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x00).unwrap()
        );

        let n_bits = 0x01123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x12).unwrap()
        );

        let n_bits = 0x04923456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&-0x12345600).unwrap()
        );

        let n_bits = 0x04123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x12345600).unwrap()
        );

        let n_bits = 0x05123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x1234560000i64).unwrap()
        );
    }
}
