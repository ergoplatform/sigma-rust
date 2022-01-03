use derive_more::From;
use ergotree_ir::chain::header::Header;
use num_bigint::{BigInt, Sign};
use sigma_ser::ScorexSerializationError;
use sigma_util::hash::blake2b256_hash;

/// Autolykos PoW puzzle scheme reference implementation.
///
/// Based on k-sum problem, so general idea is to find k numbers in a table of size N, such that
/// sum of numbers (or a hash of the sum) is less than target value.
///
/// See <https://docs.ergoplatform.com/ErgoPow.pdf> for details
///
/// CPU Mining process is implemented in inefficient way and should not be used in real environment.
///
/// See papers/yellow/pow/ErgoPow.tex for full description
#[derive(Debug)]
pub struct AutolykosPowScheme {
    /// Represents the number of elements in one solution. **Important assumption**: `k <= 32`.
    k: u64,
    /// Let `N` denote the initial table size. Then `n` is the value satisfying `N = 2 ^ n`.
    /// **Important assumption**: `n < 31`.
    n: i32,
}

impl AutolykosPowScheme {
    /// Get hit for Autolykos header (to test it then against PoW target)
    pub fn pow_hit(&self, header: &Header) -> Result<BigInt, AutolykosPowSchemeError> {
        if header.version == 1 {
            Ok(header.autolykos_solution.pow_distance.clone())
        } else {
            use byteorder::{BigEndian, WriteBytesExt};
            // hit for version 2
            let msg = header.serialize_without_pow()?;
            let nonce = header.autolykos_solution.nonce.clone();
            let mut height_bytes = Vec::with_capacity(4);
            #[allow(clippy::unwrap_used)]
            height_bytes.write_u32::<BigEndian>(header.height).unwrap();

            let mut concat = msg.clone();
            concat.extend(&nonce);

            // `N` from autolykos paper
            let big_n = self.calc_big_n(header.version, header.height);
            let pre_i8 = BigInt::from_bytes_be(Sign::Plus, &(blake2b256_hash(&concat)[(32 - 8)..]));

            // Note that `N` parameter has an upper bound of 2,147,387,550 which can fit in a `i32` (4
            // bytes), so the truncation for `i` below is safe.
            let i = as_unsigned_byte_array(
                4,
                pre_i8.modpow(&BigInt::from(1u32), &BigInt::from(big_n)),
            )?;

            // Constant data to be added to hash function to increase its calculation time
            let big_m: Vec<u8> = (0u64..1024)
                .flat_map(|x| {
                    let mut bytes = Vec::with_capacity(8);
                    #[allow(clippy::unwrap_used)]
                    bytes.write_u64::<BigEndian>(x).unwrap();
                    bytes
                })
                .collect();
            concat = i;
            concat.extend(&height_bytes);
            concat.extend(&big_m);
            let f = blake2b256_hash(&concat);

            concat = f[1..].to_vec();
            concat.extend(msg);
            concat.extend(nonce);
            let seed = blake2b256_hash(&concat);
            let indexes = self.gen_indexes(&seed, big_n);

            let f2 = indexes.into_iter().fold(BigInt::from(0u32), |acc, idx| {
                // This is specific to autolykos v2.
                let mut concat = vec![];
                #[allow(clippy::unwrap_used)]
                concat.write_u64::<BigEndian>(idx).unwrap();
                concat.extend(&height_bytes);
                concat.extend(&big_m);
                acc + BigInt::from_bytes_be(Sign::Plus, &blake2b256_hash(&concat)[1..])
            });

            // sum as byte array is always about 32 bytes
            #[allow(clippy::unwrap_used)]
            let array = as_unsigned_byte_array(32, f2).unwrap();
            Ok(BigInt::from_bytes_be(Sign::Plus, &*blake2b256_hash(&array)))
        }
    }

    /// Returns a list of size `k` with numbers in [0,`N`)
    fn gen_indexes(&self, seed_hash: &[u8; 32], big_n: usize) -> Vec<u64> {
        let mut res = vec![];
        let mut extended_hash: Vec<u8> = seed_hash.to_vec();
        extended_hash.extend(&seed_hash[..3]);
        for i in 0..self.k {
            let i = i as usize;
            res.push(
                BigInt::from_bytes_be(Sign::Plus, &extended_hash[i..(i + 4)])
                    .modpow(&BigInt::from(1u32), &BigInt::from(big_n))
                    .to_u64_digits()
                    .1[0],
            );
        }
        res
    }

    /// Calculates table size (N value) for a given height (moment of time)
    fn calc_big_n(&self, header_version: u8, header_height: u32) -> usize {
        let n_base = 2i32.pow(self.n as u32) as usize;
        if header_version == 1 {
            n_base
        } else {
            // On this height, the table (`N` value) will stop to grow
            let n_increasement_height_max = 4198400;
            let height = usize::min(n_increasement_height_max, header_height as usize);

            // Initial height since which table (`N` value) starting to increase by 5% per `IncreasePeriodForN` blocks
            let increase_start = 600 * 1024;
            if height < increase_start {
                n_base
            } else {
                // Table size (`N`) increased every 50 * 1024 blocks
                let increase_period_for_big_n = 50 * 1024;
                let iters_number = (height - increase_start) / increase_period_for_big_n + 1;
                (1..iters_number).fold(n_base, |acc, _| acc / 100 * 105)
            }
        }
    }
}

impl Default for AutolykosPowScheme {
    fn default() -> Self {
        // The following paramter values are mandated by Ergo-node Autolykos implementation.
        AutolykosPowScheme { k: 32, n: 26 }
    }
}

/// Port of BouncyCastle's BigIntegers::asUnsignedByteArray method.
fn as_unsigned_byte_array(
    length: usize,
    big_int: BigInt,
) -> Result<Vec<u8>, AutolykosPowSchemeError> {
    let bytes = big_int.to_signed_bytes_be();
    if bytes.len() == length {
        return Ok(bytes);
    }

    let start = if bytes[0] == 0 { 1 } else { 0 };
    let count = bytes.len() - start;
    if count > length {
        return Err(AutolykosPowSchemeError::BigIntToFixedByteArrayError);
    }
    let mut res: Vec<_> = std::iter::repeat(0).take(length).collect();
    res[(length - count)..].copy_from_slice(&bytes[start..]);
    Ok(res)
}

#[derive(PartialEq, Debug, Clone, From)]
pub enum AutolykosPowSchemeError {
    /// Scorex-serialization error
    ScorexSerializationError(ScorexSerializationError),
    /// Error occurring when trying to convert a `BigInt` into a fixed-length byte-array.
    BigIntToFixedByteArrayError,
}
