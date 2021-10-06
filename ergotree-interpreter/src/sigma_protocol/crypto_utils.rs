use elliptic_curve::rand_core::RngCore;

/// Generate cryptographically secure random bytes
pub fn secure_random_bytes(how_many: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0; how_many];
    secure_rng().fill_bytes(&mut bytes);
    bytes
}

/// Returns cryptographically secure PRNG
/// Uses ThreadRng - <https://rust-random.github.io/rand/rand/rngs/struct.ThreadRng.html>
/// which is StgRng (ChaCha block cipher with 12 as of July 2021 -
/// <https://rust-random.github.io/rand/rand/rngs/struct.StdRng.html>) seeded from OsRng -
/// <https://rust-random.github.io/rand/rand/rngs/struct.OsRng.html> which is a random number
/// generator that retrieves randomness from the operating system.
pub fn secure_rng() -> impl RngCore {
    use rand::thread_rng;
    thread_rng()
}
