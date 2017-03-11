use error::{self, ErrorKind};
use std::io;

pub fn read_block_size<R: io::Read>(reader: &mut R) -> error::Result<usize> {
    let n = read_long(reader)?;
    let n = if n < 0 {
        read_long(reader)?; // discard
        n.abs()
    } else {
        n
    };
    Ok(n as usize)
}

pub fn read_int<R: io::Read>(reader: &mut R) -> error::Result<i32> {
    let v = read_long(reader)?;
    if v < (i32::min_value() as i64) || v > (i32::max_value() as i64) {
        Err(ErrorKind::IntegerOverflow.into())
    } else {
        Ok(v as i32)
    }
}

pub fn read_long<R: io::Read>(reader: &mut R) -> io::Result<i64> {
    let unsigned = decode_var_len_u64(reader)?;
    Ok(decode_zig_zag(unsigned))
}

// Taken from the rust-avro functions with the same name...
// TODO: credit this when creating an ATTRIBUTIONS file or something

fn decode_var_len_u64<R: io::Read>(reader: &mut R) -> io::Result<u64> {
    use byteorder::ReadBytesExt;

    let mut num = 0;
    let mut i = 0;
    loop {
        let byte = reader.read_u8()?;

        if i >= 9 && byte & 0b1111_1110 != 0 {
            // 10th byte
            return Err(io::Error::new(io::ErrorKind::InvalidData, "integer overflow"));
        }
        num |= (byte as u64 & 0b0111_1111) << (i * 7);
        if byte & 0b1000_0000 == 0 {
            break;
        }
        i += 1;
    }
    Ok(num)
}

fn decode_zig_zag(num: u64) -> i64 {
    if num & 1 == 1 {
        !(num >> 1) as i64
    } else {
        (num >> 1) as i64
    }
}
