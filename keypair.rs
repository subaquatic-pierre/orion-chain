#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_private_key() {
        let pvt_key = PrivateKey::new();

        assert_eq!(pvt_key.to_bytes().len(), 32);

        let bytes = pvt_key.to_bytes();

        let pvt_key_2 = PrivateKey::from_bytes(&bytes).expect("unable to create private key");

        assert_eq!(pvt_key.to_hex(), pvt_key_2.to_hex());
        assert_eq!(64, pvt_key_2.to_hex().len());

        let hex = pvt_key.to_hex();
        let new_pvt_key = PrivateKey::from_hex(&hex).expect("unable to create private key");

        assert_eq!(pvt_key.to_hex(), new_pvt_key.to_hex());
    }

    #[test]
    fn test_public_key() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pub_bytes = pub_key.to_bytes();
        let pub_hex = pub_key.to_hex();

        let pub_key_2 = PublicKey::from_bytes(&pub_bytes).unwrap();

        assert_eq!(pub_key.to_hex(), pub_key_2.to_hex());

        let pub_key_3 = PublicKey::from_hex(&pub_hex).unwrap();

        assert_eq!(pub_key.to_hex(), pub_key_3.to_hex());

        assert_eq!(pub_key.to_bytes().len(), 33);
        assert_eq!(66, pub_key.to_hex().len());
    }

    #[test]
    fn test_sign() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let pub_key_2 = pvt_key_2.pub_key();

        let msg = b"Hello world";

        let sig = pvt_key.sign(msg);
        let is_valid = pub_key.verify(msg, sig.clone());

        let not_valid = pub_key_2.verify(msg, sig);

        assert_eq!(is_valid, true);
        assert_eq!(not_valid, false);
    }

    #[test]
    fn test_signature() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let pub_key_2 = pvt_key_2.pub_key();

        let msg = b"Hello world";

        let sig = pvt_key.sign(msg);
        let sig_bytes = sig.to_bytes();

        assert_eq!(sig_bytes.len(), 64);

        let sig_2 = Signature::from_bytes(&sig_bytes);

        assert_eq!(sig_2.is_err(), false);
        let sig_2 = sig_2.unwrap();

        assert_eq!(sig.to_hex(), sig_2.to_hex());

        let sig_3 = Signature::from_hex(&sig_2.to_hex());

        assert_eq!(sig_3.is_err(), false);
        let sig_3 = sig_3.unwrap();

        assert_eq!(sig.to_hex(), sig_3.to_hex());
    }

    #[test]
    fn test_address() {
        let pvt_key = PrivateKey::new();
        let pub_key = pvt_key.pub_key();

        let pvt_key_2 = PrivateKey::new();
        let pub_key_2 = pvt_key_2.pub_key();

        let addr = pub_key.address().unwrap();

        let bytes = pub_key.to_bytes();

        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let addr_2 = Address::from_bytes(&addr_bytes).unwrap();

        assert_eq!(addr.to_hex(), addr_2.to_hex());

        let bytes = pub_key_2.to_bytes();

        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let addr_3 = Address::from_bytes(&addr_bytes).unwrap();
        assert_ne!(addr.to_hex(), addr_3.to_hex());

        let bytes = pub_key_2.to_bytes();
        let mut addr_bytes = [0_u8; 20];

        for (i, &b) in bytes.iter().rev().enumerate() {
            if i == 20 {
                break;
            }
            addr_bytes[i] = b
        }

        let new_hex = hex::encode(&addr_bytes);
        let addr_4 = Address::from_hex(&new_hex).unwrap();

        assert_eq!(pub_key_2.address().unwrap().to_hex(), addr_4.to_hex());
    }
}
