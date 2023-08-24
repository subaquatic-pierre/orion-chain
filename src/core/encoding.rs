use std::any;
use std::error::Error;
use std::io::{Read, Write};

use super::block::Block;
use super::error::CoreError;

// pub trait Encoder {
//     fn encode(&self, writer: impl Write, data: impl ByteEncoding) -> Result<(), Box<dyn Error>>;
// }

// pub trait Decoder<T, E>
// where
//     E: Error,
// {
//     fn decode(writer: impl Read, data: &T) -> Result<(), E>;
// }

// pub struct BlockEncoder {
//     // type Error: Error;
// }

// impl Encoder for BlockEncoder {
//     fn encode(
//         &self,
//         mut writer: impl Write,
//         data: impl ByteEncoding,
//     ) -> Result<(), Box<dyn Error>> {
//         writer.write_all(&data.to_bytes())?;
//         Ok(())
//     }
// }

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
