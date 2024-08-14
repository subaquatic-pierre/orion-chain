use rand::random;

use super::hash::Hash;

pub fn random_bytes(num_bytes: u32) -> Vec<u8> {
    (0..num_bytes).map(|_| random::<u8>()).collect()
}

pub fn random_hash() -> Hash {
    let mut buf = [0_u8; 32];
    for (i, b) in random_bytes(32).iter().enumerate() {
        buf[i] = b.clone()
    }
    Hash::new(&buf).unwrap()
}
