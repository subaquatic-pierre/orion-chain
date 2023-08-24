use std::error::Error;

pub trait ByteEncoding<T, E>
where
    E: Error,
{
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(data: &[u8]) -> Result<T, E>;
}

pub trait HexEncoding<T, E>
where
    E: Error,
{
    fn to_hex(&self) -> String;
    fn from_hex(data: &str) -> Result<T, E>;
}
