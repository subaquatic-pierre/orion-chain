use std::{
    io::{self, Error, Read, Write},
    net::TcpStream,
};

use serde::{Deserialize, Serialize};

use crate::core::{
    block::Block, encoding::ByteEncoding, error::CoreError, transaction::Transaction,
};

pub trait Encoder<T>
where
    T: ByteEncoding<T>,
{
    fn encode(&mut self, data: &T) -> Result<(), Error>;
}

pub trait Decoder<T>
where
    T: ByteEncoding<T>,
{
    fn decode(&mut self) -> Result<T, Error>;
}

pub struct BlockEncoder<T: Write> {
    writer: T,
}

impl BlockEncoder<VecBuf> {
    pub fn new_buf_encoder(writer: VecBuf) -> Self {
        Self { writer }
    }

    pub fn inner_bytes(&mut self) -> Vec<u8> {
        self.writer.inner_bytes()
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush()
    }
}

impl Encoder<Block> for BlockEncoder<VecBuf> {
    fn encode(&mut self, data: &Block) -> Result<(), Error> {
        let bytes = match data.to_bytes() {
            Ok(b) => b,
            Err(e) => return Err(Error::new(io::ErrorKind::InvalidData, e)),
        };

        self.writer.write_all(&bytes)
    }
}

impl BlockEncoder<TcpStream> {
    pub fn new_stream_encoder(stream: TcpStream) -> Self {
        Self { writer: stream }
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush()
    }
}

impl Encoder<Block> for BlockEncoder<TcpStream> {
    fn encode(&mut self, data: &Block) -> Result<(), Error> {
        let bytes = match data.to_bytes() {
            Ok(b) => b,
            Err(e) => return Err(Error::new(io::ErrorKind::InvalidData, e)),
        };

        self.writer.write_all(&bytes)
    }
}

pub struct BlockDecoder<T: Read> {
    reader: T,
}

impl BlockDecoder<TcpStream> {
    pub fn new_stream_decoder(reader: TcpStream) -> Self {
        Self { reader }
    }
}

impl BlockDecoder<VecBuf> {
    pub fn new_buf_decoder(reader: VecBuf) -> Self {
        Self { reader }
    }
}

impl Decoder<Block> for BlockDecoder<TcpStream> {
    fn decode(&mut self) -> Result<Block, Error> {
        let mut buf = vec![];
        self.reader.read_to_end(&mut buf)?;

        match Block::from_bytes(&buf) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

impl Decoder<Block> for BlockDecoder<VecBuf> {
    fn decode(&mut self) -> Result<Block, Error> {
        if self.reader.cur == self.reader.inner.len() {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "reader is already consumed",
            ));
        }

        let mut buf = vec![];
        self.reader.read_to_end(&mut buf)?;

        match Block::from_bytes(&buf) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

// ---
// TX ENCODER / DECODER
// ---

pub struct TxEncoder<T: Write> {
    writer: T,
}

impl TxEncoder<VecBuf> {
    pub fn new_buf_encoder(writer: VecBuf) -> Self {
        Self { writer }
    }

    pub fn inner_bytes(&mut self) -> Vec<u8> {
        self.writer.inner_bytes()
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush()
    }
}

impl Encoder<Transaction> for TxEncoder<VecBuf> {
    fn encode(&mut self, data: &Transaction) -> Result<(), Error> {
        let bytes = match data.to_bytes() {
            Ok(b) => b,
            Err(e) => return Err(Error::new(io::ErrorKind::InvalidData, e)),
        };

        self.writer.write_all(&bytes)
    }
}

impl TxEncoder<TcpStream> {
    pub fn new_stream_encoder(stream: TcpStream) -> Self {
        Self { writer: stream }
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush()
    }
}

impl Encoder<Transaction> for TxEncoder<TcpStream> {
    fn encode(&mut self, data: &Transaction) -> Result<(), Error> {
        let bytes = match data.to_bytes() {
            Ok(b) => b,
            Err(e) => return Err(Error::new(io::ErrorKind::InvalidData, e)),
        };

        self.writer.write_all(&bytes)
    }
}

pub struct TxDecoder<T: Read> {
    reader: T,
}

impl TxDecoder<TcpStream> {
    pub fn new_stream_decoder(reader: TcpStream) -> Self {
        Self { reader }
    }
}

impl TxDecoder<VecBuf> {
    pub fn new_buf_decoder(reader: VecBuf) -> Self {
        Self { reader }
    }
}

impl Decoder<Transaction> for TxDecoder<TcpStream> {
    fn decode(&mut self) -> Result<Transaction, Error> {
        let mut buf = vec![];
        self.reader.read_to_end(&mut buf)?;

        match Transaction::from_bytes(&buf) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

impl Decoder<Transaction> for TxDecoder<VecBuf> {
    fn decode(&mut self) -> Result<Transaction, Error> {
        if self.reader.cur == self.reader.inner.len() {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "reader is already consumed",
            ));
        }

        let mut buf = vec![];
        self.reader.read_to_end(&mut buf)?;

        match Transaction::from_bytes(&buf) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

// ---
// Generic Wrapper for Vec<u8>
// to provide Read implementation
// not very effective, better to just return
// bytes, this is used mostly for testing
pub struct VecBuf {
    inner: Vec<u8>,
    cur: usize,
}

impl VecBuf {
    pub fn new_writer() -> Self {
        Self {
            inner: vec![],
            cur: 0,
        }
    }

    pub fn new_reader(bytes: &[u8]) -> Self {
        Self {
            inner: bytes.to_vec(),
            cur: 0,
        }
    }

    pub fn inner_bytes(&self) -> Vec<u8> {
        self.inner.clone()
    }
}

impl Write for VecBuf {
    fn write(&mut self, b: &[u8]) -> Result<usize, Error> {
        self.inner = b.to_vec();
        Ok(b.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.inner = vec![];
        Ok(())
    }
}

impl Read for VecBuf {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cur == self.inner.len() {
            return Ok(0);
        }

        let mut total_bytes = 0;

        for i in 0..buf.len() {
            if self.cur == self.inner.len() {
                return Ok(total_bytes);
            };

            buf[i] = self.inner[self.cur];
            self.cur += 1;
            total_bytes += 1;
        }

        Ok(total_bytes)
    }
}

#[cfg(test)]
mod test {
    use std::io::BufWriter;

    use crate::{
        core::{block::random_block, header::random_header, transaction::random_signed_tx},
        crypto::utils::random_hash,
    };

    use super::*;

    #[test]
    fn test_block_encoder() {
        let header = random_header(1, random_hash());
        let block = random_block(header);

        let block_bytes = block.to_bytes();

        // encode block into vev buf writer
        let buf = VecBuf::new_writer();
        let mut enc = BlockEncoder::new_buf_encoder(buf);
        enc.encode(&block).ok();

        // assert encoded bytes same as bytes above
        let encoded_bytes = enc.inner_bytes();
        assert_eq!(format!("{encoded_bytes:?}"), format!("{block_bytes:?}"));
    }

    #[test]
    fn test_block_decoder() {
        let header = random_header(1, random_hash());
        let block = random_block(header);

        let block_bytes = block.to_bytes().unwrap();

        let reader = VecBuf::new_reader(&block_bytes);
        let mut dec = BlockDecoder::new_buf_decoder(reader);

        if let Ok(b) = dec.decode() {
            assert_eq!(format!("{block:?}"), format!("{b:?}"));
        }

        let msg = "reader is already consumed".to_string();
        match dec.decode() {
            Ok(_) => {}
            Err(e) => assert_eq!(e.to_string(), msg),
        }
    }

    #[test]
    fn test_tx_encoder() {
        let tx = random_signed_tx();

        let bytes = tx.to_bytes();

        // encode tx into vec buf writer
        let buf = VecBuf::new_writer();
        let mut enc = TxEncoder::new_buf_encoder(buf);
        enc.encode(&tx).ok();

        // assert encoded bytes same as bytes above
        let encoded_bytes = enc.inner_bytes();
        assert_eq!(format!("{encoded_bytes:?}"), format!("{bytes:?}"));
    }

    #[test]
    fn test_tx_decoder() {
        let tx = random_signed_tx();

        let bytes = tx.to_bytes().unwrap();

        let reader = VecBuf::new_reader(&bytes);
        let mut dec = TxDecoder::new_buf_decoder(reader);

        if let Ok(decoded) = dec.decode() {
            assert_eq!(format!("{tx:?}"), format!("{decoded:?}"));
        }

        let msg = "reader is already consumed".to_string();
        match dec.decode() {
            Ok(_) => {}
            Err(e) => assert_eq!(e.to_string(), msg),
        }
    }

    #[test]
    fn test_vec_buf_read() {
        let bytes: [u8; 32] = rand::random();
        let mut vec_writer = VecBuf::new_writer();

        vec_writer.write_all(&bytes);

        let encoded_bytes = vec_writer.inner_bytes();

        assert_eq!(format!("{bytes:?}"), format!("{encoded_bytes:?}"));
    }

    #[test]
    fn test_vec_buf_write() {
        let bytes: Vec<u8> = (0..1025).map(|_| rand::random::<u8>()).collect();
        let mut vec_reader = VecBuf::new_reader(&bytes);

        let mut read_bytes = vec![];

        vec_reader.read_to_end(&mut read_bytes).unwrap();

        assert_eq!(format!("{bytes:?}"), format!("{read_bytes:?}"));
    }
}
