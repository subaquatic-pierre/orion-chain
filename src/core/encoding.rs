use std::error::Error;

pub trait ByteEncoding {
    fn to_bytes(&self) -> Vec<u8>;
}
pub trait ByteDecoding {
    type Target;
    type Error: Error;
    fn from_bytes(data: &[u8]) -> Result<Self::Target, Self::Error>;
}

pub trait HexDecoding {
    type Target;
    type Error: Error;

    fn from_hex(data: &str) -> Result<Self::Target, Self::Error>;
}

pub trait HexEncoding {
    fn to_hex(&self) -> String;
}
