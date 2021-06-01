/// Generate cryptographically secure random bytes
pub fn secure_random_bytes(how_many: usize) -> Vec<u8> {
    use rand::rngs::OsRng;
    use rand::RngCore;
    let mut bytes: Vec<u8> = vec![0; how_many];
    OsRng.fill_bytes(&mut bytes);
    bytes
}
