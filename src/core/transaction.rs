use borsh::{BorshDeserialize, BorshSerialize};
use k256::sha2::Sha256;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_with::base64::{Base64, Bcrypt, BinHex, Standard};
use serde_with::serde_as;

use crate::crypto::address::{random_sender_receiver, Address};
use crate::crypto::utils::random_hash;
use crate::crypto::{
    hash::Hash,
    private_key::PrivateKey,
    public_key::{PublicKey, PublicKeyBytes},
    signature::{Signature, SignatureBytes},
    utils::random_bytes,
};

use super::{
    encoding::{ByteEncoding, HexEncoding},
    error::CoreError,
};

#[serde_as]
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Transaction {
    pub tx_type: TxType,
    pub data: Vec<u8>,
    pub receiver: Address,
    pub sender: Address,
    pub blockhash: Hash,
    pub hash: Option<Hash>,
    pub gas_limit: u64,
    pub signature: Option<SignatureBytes>,
    pub signer: Option<PublicKeyBytes>,
}

pub struct TxVerificationData {
    pub signature: SignatureBytes,
    pub signer: PublicKeyBytes,
    pub hash: Hash,
}

impl Transaction {
    pub fn new(
        tx_type: TxType,
        blockhash: Hash,
        receiver: Address,
        sender: Address,
        data: &[u8],
        gas_limit: u64,
    ) -> Result<Self, CoreError> {
        let data = data.to_vec();

        Ok(Self {
            tx_type,
            data,
            receiver,
            sender,
            blockhash,
            gas_limit,
            signature: None,
            signer: None,
            hash: None,
        })
    }

    pub fn new_transfer(
        receiver: Address,
        sender: Address,
        blockhash: Hash,
        data: &[u8],
        gas_limit: u64,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            tx_type: TxType::Transfer,
            receiver,
            sender,
            data: data.to_vec(),
            blockhash,
            gas_limit,
            signature: None,
            signer: None,
            hash: None,
        })
    }

    pub fn hash(&self) -> Result<Hash, CoreError> {
        match self.hash {
            Some(d) => Ok(d),
            None => Err(CoreError::Transaction("no hash on transaction".to_string())),
        }
    }

    pub fn signature(&self) -> Result<SignatureBytes, CoreError> {
        match &self.signature {
            Some(d) => Ok(d.clone()),
            None => Err(CoreError::Transaction(
                "no signature on transaction".to_string(),
            )),
        }
    }
    pub fn signer(&self) -> Result<PublicKeyBytes, CoreError> {
        match &self.signer {
            Some(d) => Ok(d.clone()),
            None => Err(CoreError::Transaction(
                "no public key on transaction".to_string(),
            )),
        }
    }

    pub fn data_str(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }

    pub fn hashable_data(&self) -> Vec<u8> {
        let mut buf = vec![];

        // Include the transaction type
        buf.extend_from_slice(&self.tx_type.to_bytes().unwrap());

        // Include the sender's address
        buf.extend_from_slice(&self.sender.to_bytes().unwrap());

        // Include the receiver's address
        buf.extend_from_slice(&self.receiver.to_bytes().unwrap());

        // Include the transaction data
        buf.extend_from_slice(&self.data);

        // Include the block hash
        buf.extend_from_slice(&self.blockhash.to_bytes().unwrap());
        buf
    }

    pub fn sign(&mut self, private_key: &PrivateKey) -> Result<TxVerificationData, CoreError> {
        if self.signer.is_some() | self.signature.is_some() {
            return Err(CoreError::Transaction(
                "transaction already is already signed".to_string(),
            ));
        }

        let hash_data = self.hashable_data();

        let sig = private_key.sign(&hash_data);
        let sig_bytes = SignatureBytes::new(&sig.to_bytes()?)?;
        let pub_key_bytes = PublicKeyBytes::new(&private_key.pub_key().to_bytes()?)?;

        let hash = Hash::sha256(&sig_bytes.to_bytes()?)?;

        self.signer = Some(pub_key_bytes.clone());
        self.signature = Some(sig_bytes.clone());
        self.hash = Some(hash.clone());

        Ok(TxVerificationData {
            signature: sig_bytes,
            signer: pub_key_bytes,
            hash: hash,
        })
    }

    pub fn verify(&self) -> Result<(), CoreError> {
        if self.signature.is_none() {
            return Err(CoreError::Transaction(
                "transaction has no signature".to_string(),
            ));
        }

        if self.hash.is_none() {
            return Err(CoreError::Transaction(
                "transaction has no hash".to_string(),
            ));
        }

        match (&self.signer, &self.signature) {
            (Some(key_bytes), Some(sig_bytes)) => {
                let key = PublicKey::from_bytes(&key_bytes.to_bytes()?)?;
                let signature = Signature::from_bytes(&sig_bytes.to_bytes()?)?;

                let data = self.hashable_data();

                if !key.verify(&data, &signature) {
                    return Err(CoreError::Transaction(
                        "invalid transaction signature".to_string(),
                    ));
                }
            }
            _ => {
                return Err(CoreError::Transaction(
                    "transaction has no public key or signature".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl ByteEncoding<Transaction> for Transaction {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<Transaction, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

impl HexEncoding<Transaction> for Transaction {
    fn from_hex(data: &str) -> Result<Transaction, CoreError> {
        Ok(Self::from_bytes(&hex::decode(data)?)?)
    }

    fn to_hex(&self) -> Result<String, CoreError> {
        Ok(hex::encode(self.to_bytes()?))
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum TxType {
    Transfer,
    SmartContract,
    BlockReward,
    GasReward,
}

impl ByteEncoding<TxType> for TxType {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<TxType, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TransferData {
    pub to: Address,
    pub from: Address,
    pub amount: u64,
}

impl ByteEncoding<TransferData> for TransferData {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<TransferData, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SmartContractData {
    pub contract_address: Address,
    pub method: String,
    pub params: Vec<u8>,
}

impl ByteEncoding<SmartContractData> for SmartContractData {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<SmartContractData, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct BlockRewardData {
    pub to: Address,
    pub amount: u64,
}

impl ByteEncoding<BlockRewardData> for BlockRewardData {
    fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        match borsh::to_vec(self) {
            Ok(b) => Ok(b),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }

    fn from_bytes(data: &[u8]) -> Result<BlockRewardData, CoreError> {
        match borsh::from_slice(data) {
            Ok(t) => Ok(t),
            Err(e) => Err(CoreError::Parsing(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{address::random_sender_receiver, utils::random_hash};

    use super::*;
    #[test]
    fn test_transaction_sign() {
        let r_hash = random_hash();

        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";
        let (sender, receiver) = random_sender_receiver();

        let mut tx = Transaction::new_transfer(sender, receiver, r_hash, data, 3).unwrap();

        assert!(matches!(tx.verify(), Err(_)));

        tx.sign(&priv_key).unwrap();
        assert!(tx.verify().is_ok());

        // try change data
        tx.data = b"changed data".to_vec();
        assert!(matches!(tx.verify(), Err(_)));

        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";
        let (sender, receiver) = random_sender_receiver();

        let mut tx = Transaction::new_transfer(sender, receiver, r_hash, data, 3).unwrap();

        // try double sign
        tx.sign(&priv_key).unwrap();
        assert!(matches!(tx.sign(&priv_key), Err(_)));
    }

    #[test]
    fn test_transaction_data_str() {
        let r_hash = random_hash();
        let _priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";
        let (sender, receiver) = random_sender_receiver();

        let tx = Transaction::new_transfer(sender, receiver, r_hash, data, 3).unwrap();
        assert_eq!(tx.data_str(), "Hello world, Data is cool");
    }

    #[test]
    fn test_transaction_parse_bytes() {
        let r_hash = random_hash();
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";
        let (sender, receiver) = random_sender_receiver();

        let mut tx = Transaction::new_transfer(sender, receiver, r_hash, data, 3).unwrap();

        tx.sign(&priv_key).unwrap();
        let bytes = &tx.to_bytes().unwrap();

        let tx_1_sig = tx.signature.unwrap();

        let tx_2 = Transaction::from_bytes(&bytes);

        assert!(tx_2.is_ok());

        let tx_2 = tx_2.unwrap();

        assert_eq!(tx_2.data_str(), "Hello world, Data is cool");

        assert!(tx_2.verify().is_ok());

        let tx_2_sig = tx_2.signature.unwrap();
        let tx_2_pub_key = tx_2.signer.unwrap();

        let pub_key = priv_key.pub_key().to_bytes().unwrap();

        assert_eq!(tx_1_sig, tx_2_sig);
        assert_eq!(tx_2_pub_key.to_bytes().unwrap(), pub_key)
    }

    #[test]
    fn test_transaction_parse_hex() {
        let priv_key = PrivateKey::new();
        let data = b"Hello world, Data is cool";
        let (sender, receiver) = random_sender_receiver();
        let r_hash = random_hash();

        let mut tx = Transaction::new_transfer(sender, receiver, r_hash, data, 3).unwrap();
        let _hex_str = tx.to_hex().unwrap();

        tx.sign(&priv_key).unwrap();
        let hex_str = tx.to_hex().unwrap();

        let _tx_1_hash = tx.hash();
        let tx_1_sig = tx.signature.unwrap();

        let tx_2 = Transaction::from_hex(&hex_str);

        assert!(tx_2.is_ok());

        let tx_2 = tx_2.unwrap();

        assert_eq!(tx_2.data_str(), "Hello world, Data is cool");

        assert!(tx_2.verify().is_ok());

        let tx_2_hash = tx_2.hash().unwrap();
        let tx_2_sig = tx_2.signature.unwrap();

        assert_eq!(tx_2_hash.len(), 32);

        let tx_2_pub_key = tx_2.signer.unwrap();

        let pub_key = priv_key.pub_key().to_bytes().unwrap();

        assert_eq!(tx_1_sig, tx_2_sig);
        assert_eq!(tx_2_pub_key.to_bytes().unwrap(), pub_key);

        assert_eq!(tx_2_hash, tx_2_hash);
    }
}

pub fn random_tx() -> Transaction {
    let r_hash = random_hash();
    let (sender, receiver) = random_sender_receiver();
    let bytes = TransferData {
        to: receiver.clone(),
        from: sender.clone(),
        amount: 42,
    }
    .to_bytes()
    .unwrap();
    Transaction::new_transfer(sender, receiver, r_hash, &bytes, 3).unwrap()
}

pub fn random_signed_tx() -> Transaction {
    let mut tx = random_tx();
    let pvt = PrivateKey::new();
    tx.sign(&pvt).unwrap();
    tx
}
