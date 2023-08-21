use hex_literal::hex;
use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
    U256,
};

struct PrivateKey {
    key: SigningKey,
}

impl PrivateKey {
    pub fn new() -> Self {
        Self {
            key: SigningKey::random(&mut OsRng),
        }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            key: SigningKey::from_bytes(bytes.into()).unwrap(),
        }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut buf = [0_u8; 32];
        for (i, v) in self.key.to_bytes().iter().copied().enumerate() {
            buf[i] = v
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    pub fn pub_key(&self) -> PublicKey {
        let verifying_key = VerifyingKey::from(&self.key);
        PublicKey { key: verifying_key }
    }
}

struct PublicKey {
    key: VerifyingKey,
}

impl PublicKey {
    pub fn to_bytes_uncompressed(&self) -> [u8; 65] {
        let point = self.key.to_encoded_point(false);
        let hex = hex::encode(point);

        let mut buf = [0_u8; 65];

        let bytes = hex::decode(hex).expect("problem").to_vec();

        println!("len of hex inside to bytes: {} ", bytes.len());

        for (i, &v) in bytes.iter().enumerate() {
            buf[i] = v
        }
        buf
    }

    pub fn to_bytes(&self) -> [u8; 33] {
        let point = self.key.to_encoded_point(true);
        let hex = hex::encode(point);

        let mut buf = [0_u8; 33];

        let bytes = hex::decode(hex).expect("problem").to_vec();

        println!("len of hex inside to bytes: {} ", bytes.len());

        for (i, &v) in bytes.iter().enumerate() {
            buf[i] = v
        }
        buf
    }

    pub fn to_hex(&self, compressed: bool) -> String {
        hex::encode(self.key.to_encoded_point(compressed))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_private_key() {
        let pvt_key = PrivateKey::new();

        assert_eq!(pvt_key.to_bytes().len(), 32);

        let bytes = pvt_key.to_bytes();

        let pvt_key_2 = PrivateKey::from_bytes(&bytes);

        assert_eq!(pvt_key.to_hex(), pvt_key_2.to_hex());
        assert_eq!(64, pvt_key_2.to_hex().len());
    }

    #[test]
    fn test_public_key() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        assert_eq!(pub_key.to_bytes().len(), 33);
        assert_eq!(pub_key.to_bytes_uncompressed().len(), 65);

        assert_eq!(130, pub_key.to_hex(false).len());
        assert_eq!(66, pub_key.to_hex(true).len());
    }
}
