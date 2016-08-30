use byteorder;
use crc;
use error::{self, ErrorKind};
use flate2;
use snap;
use std::io;
use super::util;

pub trait Limit {
    fn take_limit(&mut self) -> io::Result<bool>;
}

pub struct Direct<R> {
    input: R,
    remaining: usize,
}

pub enum Codec {
    Null,
    Deflate,
    Snappy,
}

pub struct Blocks<R>
    where R: io::Read
{
    input: R,
    codec: Codec,
    sync_marker: Vec<u8>,
    current_block: io::Cursor<Vec<u8>>,
    remaining: usize,
}

impl Codec {
    pub fn parse(codec: Option<&[u8]>) -> error::Result<Codec> {
        match codec {
            None | Some(b"null") => Ok(Codec::Null),
            Some(b"deflate") => Ok(Codec::Deflate),
            Some(b"snappy") => Ok(Codec::Snappy),
            Some(codec) => {
                Err(ErrorKind::UnsupportedCodec(String::from_utf8_lossy(codec).into_owned()).into())
            },
        }
    }
}

impl<R> Direct<R>
    where R: io::Read
{
    pub fn new(input: R, remaining: usize) -> Direct<R> {
        Direct {
            input: input,
            remaining: remaining,
        }
    }
}

impl<R> io::Read for Direct<R>
    where R: io::Read
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl<R> Limit for Direct<R>
    where R: io::Read
{
    fn take_limit(&mut self) -> io::Result<bool> {
        if self.remaining == 0 {
            Ok(false)
        } else {
            self.remaining -= 1;
            Ok(true)
        }
    }
}

impl<R> Blocks<R>
    where R: io::Read
{
    pub fn new(input: R, codec: Codec, sync_marker: Vec<u8>) -> Blocks<R> {
        Blocks {
            input: input,
            codec: codec,
            sync_marker: sync_marker,
            current_block: io::Cursor::new(Vec::new()),
            remaining: 0,
        }
    }

    fn read_next_block(&mut self) -> io::Result<()> {
        use std::io::Read;

        // Make sure the current block is empty
        assert_eq!(0, self.remaining);
        let pos = self.current_block.position() as usize;
        let len = self.current_block.get_ref().len();
        assert_eq!(pos, len);

        self.current_block.set_position(0);
        let mut buffer = self.current_block.get_mut();
        buffer.clear();

        let obj_count = match util::read_long(&mut self.input) {
            Ok(v) => v,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };

        let compressed_size = try!(util::read_long(&mut self.input));
        debug!("Loading block with compressed size {} containing {} objects",
               compressed_size,
               obj_count);

        match self.codec {
            Codec::Null => {
                debug!("Copying block data with null codec");
                let mut limited = (&mut self.input).take(compressed_size as u64);
                buffer.reserve(compressed_size as usize);
                try!(limited.read_to_end(buffer));
            },
            Codec::Deflate => {
                debug!("Copying block data with deflate codec");
                let limited = (&mut self.input).take(compressed_size as u64);
                let mut reader = flate2::read::DeflateDecoder::new(limited);
                try!(reader.read_to_end(buffer));
            },
            Codec::Snappy => {
                use byteorder::ByteOrder;

                debug!("Copying block data with snappy codec");
                let mut compressed = vec![0; compressed_size as usize - 4];
                try!(self.input.read_exact(&mut compressed));
                let decompressed_size = try!(snap::decompress_len(&compressed));
                debug!("Decompressed block is expected to be {} bytes",
                       decompressed_size);
                buffer.resize(decompressed_size, 0);
                try!(snap::Decoder::new().decompress(&compressed, buffer));

                let mut crc_buffer = [0; 4];
                try!(self.input.read_exact(&mut crc_buffer));
                let expected_crc = byteorder::BigEndian::read_u32(&crc_buffer);
                let actual_crc = crc::crc32::checksum_ieee(&buffer);

                if expected_crc != actual_crc {
                    let m = format!("bad CRC32; expected {:x} but got {:x}", expected_crc, actual_crc);
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, m));
                }
            },
        }
        debug!("Uncompressed block contains {} bytes", buffer.len());

        let mut sync_marker = vec![0; 16];
        try!(self.input.read_exact(&mut sync_marker));

        self.remaining = obj_count as usize;

        if self.sync_marker != sync_marker {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "bad sync marker"))
        } else {
            Ok(())
        }
    }
}

impl<R> io::Read for Blocks<R>
    where R: io::Read
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.current_block.read(buf)
    }
}

impl<R> Limit for Blocks<R>
    where R: io::Read
{
    fn take_limit(&mut self) -> io::Result<bool> {
        if self.remaining == 0 {
            try!(self.read_next_block());
        }

        if self.remaining == 0 {
            Ok(false)
        } else {
            self.remaining -= 1;
            Ok(true)
        }
    }
}
