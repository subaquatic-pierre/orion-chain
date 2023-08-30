use std::{
    io::{BufReader, BufWriter, Error, Read, Write},
    net::TcpStream,
};

use crate::core::{
    block::Block,
    encoding::{ByteDecoding, ByteEncoding},
    error::CoreError,
    transaction::Transaction,
};

pub trait Encoder<T: ByteEncoding> {
    fn encode(&mut self, data: &T) -> Result<(), Error>;
}

pub trait Decoder<T>
where
    T: ByteDecoding<Target = T, Error = CoreError>,
{
    fn decode(&mut self) -> Result<T, Error>;
}

pub struct BlockEncoder<T: Write> {
    writer: T,
}

impl BlockEncoder<BufWriter<Vec<u8>>> {
    pub fn new_buf_encoder(writer: BufWriter<Vec<u8>>) -> Self {
        Self { writer }
    }

    pub fn bytes(&mut self) -> Vec<u8> {
        let mut vec = vec![];
        for b in self.writer.buffer() {
            vec.push(*b)
        }

        vec
    }
}

impl Encoder<Block> for BlockEncoder<BufWriter<Vec<u8>>> {
    fn encode(&mut self, data: &Block) -> Result<(), Error> {
        self.writer.write_all(&data.to_bytes())?;
        self.writer.flush()
    }
}

impl BlockEncoder<TcpStream> {
    pub fn new_stream_encoder(stream: TcpStream) -> Self {
        Self { writer: stream }
    }
}

impl Encoder<Block> for BlockEncoder<TcpStream> {
    fn encode(&mut self, data: &Block) -> Result<(), Error> {
        self.writer.write_all(&data.to_bytes())?;
        self.writer.flush()
    }
}

pub struct BlockDecoder<T: Read> {
    reader: T,
}

impl BlockDecoder<BufReader<TcpStream>> {
    pub fn new_stream_decoder(reader: BufReader<TcpStream>) -> Self {
        Self { reader }
    }
}

impl BlockDecoder<BufReader<VecStream>> {
    pub fn new_buf_decoder(reader: BufReader<VecStream>) -> Self {
        Self { reader }
    }
}

impl Decoder<Block> for BlockDecoder<BufReader<TcpStream>> {
    fn decode(&mut self) -> Result<Block, Error> {
        let mut buf = vec![];
        self.reader.read_to_end(&mut buf)?;

        match Block::from_bytes(&buf) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

impl Decoder<Block> for BlockDecoder<BufReader<VecStream>> {
    fn decode(&mut self) -> Result<Block, Error> {
        let mut buf = vec![];
        self.reader.read_to_end(&mut buf)?;

        match Block::from_bytes(&buf) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

pub struct TxEncoder {
    writer: Box<dyn Write>,
}

// impl Encoder<Transaction> for TxEncoder {
//     fn writer(&mut self) -> &mut Box<dyn Write> {
//         &mut self.writer
//     }
// }

impl TxEncoder {
    fn new(writer: impl Write + 'static) -> Self {
        Self {
            writer: Box::new(writer),
        }
    }
}

pub struct TxDecoder {
    reader: Box<dyn Read>,
}

impl TxDecoder {
    pub fn new(reader: Box<dyn Read>) -> Self {
        Self { reader }
    }
}

// impl Decoder<Transaction> for TxDecoder {
//     fn reader(&mut self) -> &mut Box<dyn Read> {
//         &mut self.reader
//     }
// }

// ---
// Generic Wrapper for Vec<u8>
// to provide Read implementation
// not very effective, better to just return
// bytes, this is used mostly for testing
pub struct VecStream {
    inner: Vec<u8>,
}

impl VecStream {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            inner: bytes.to_vec(),
        }
    }
}

impl Read for VecStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut total = 0;

        for b in &self.inner {
            buf[total] = *b;
            total += 1;
        }

        Ok(total)
    }
}

#[cfg(test)]
mod test {
    use std::io::BufWriter;

    use crate::{
        core::{block::random_block, header::random_header},
        crypto::utils::random_hash,
    };

    use super::*;

    #[test]
    fn test_block_encoder() {
        let header = random_header(1, random_hash());
        let block = random_block(header);

        let block_bytes = block.to_bytes();
        let first_bytes = block_bytes.first().unwrap();
        let last_bytes = block_bytes.first().unwrap();
        let byte_len = block_bytes.len();

        // encode block into vev buf writer
        let buf = BufWriter::new(vec![]);
        let mut enc = BlockEncoder::new_buf_encoder(buf);
        enc.encode(&block).ok();

        // assert encoded bytes same as bytes above
        let encoded_bytes = enc.bytes();
        // assert_eq!(encoded_bytes.len(), byte_len);
    }

    // #[test]
    // fn test_block_decoder() {
    //     let header = random_header(1, random_hash());
    //     let block = random_block(header);

    //     let block_bytes = block.to_bytes();

    //     // encode block into vev buf writer
    //     let stream = VecStream::new(&block_bytes);
    //     let reader = BufReader::new(stream);
    //     let mut dec = BlockDecoder::new_buf_decoder(reader);
    //     if let Ok(decoded_block) = dec.decode() {
    //         println!("decoded_block:{:?}", decoded_block);
    //     }
    // }

    // #[test]
    // fn test_transaction_encoder() {}
    // #[test]
    // fn test_transaction_decoder() {}
}
