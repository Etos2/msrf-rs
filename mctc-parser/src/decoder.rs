use std::{error::Error, fmt::Display};

use ascii::AsciiString;

use crate::{
    data::{CodecEntry, CodecTable, Header, Record, RecordMeta},
    io::{
        Decodable, DecodeError, DecodeExt, DecodeExt2, FromByteResult, FromByteSlice,
        FromByteSliceBounded,
    },
    MAGIC_BYTES,
};

const HEADER_MINUMUM_REQUIRED: usize = 8;

#[derive(Debug)]
pub enum DecoderError {
    NeedData(usize),
    InvalidLength(usize), // TODO: More info
    Invalid,
}

impl Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderError::NeedData(n) => writeln!(f, "need {n} bytes"),
            DecoderError::InvalidLength(n) => writeln!(f, "invalid length ({n})"),
            DecoderError::Invalid => writeln!(f, "invalid"),
        }
    }
}

impl Error for DecoderError {}

fn take_checked<const LENGTH: usize>(data: &mut &[u8]) -> Option<[u8; LENGTH]> {
    let (out, remainder) = data.split_at_checked(LENGTH)?;
    *data = remainder;
    Some(out.try_into().unwrap())
}

fn take<const LENGTH: usize>(data: &mut &[u8]) -> [u8; LENGTH] {
    let (out, remainder) = data.split_at(LENGTH);
    *data = remainder;
    out.try_into().unwrap()
}

fn take_slice_checked<'a>(data: &mut &'a [u8], len: usize) -> Option<&'a [u8]> {
    let (out, remainder) = data.split_at_checked(len)?;
    *data = remainder;
    Some(out)
}

fn take_slice<'a>(data: &mut &'a [u8], len: usize) -> &'a [u8] {
    let (out, remainder) = data.split_at(len);
    *data = remainder;
    out
}

fn take_u8_checked(data: &mut &[u8]) -> Option<u8> {
    Some(u8::from_le_bytes(take_checked(data)?))
}

fn take_u16_checked(data: &mut &[u8]) -> Option<u16> {
    Some(u16::from_le_bytes(take_checked(data)?))
}

fn take_u32_checked(data: &mut &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(take_checked(data)?))
}

fn take_u8(data: &mut &[u8]) -> u8 {
    u8::from_le_bytes(take(data))
}

fn take_u16(data: &mut &[u8]) -> u16 {
    u16::from_le_bytes(take(data))
}

fn take_u32(data: &mut &[u8]) -> u32 {
    u32::from_le_bytes(take(data))
}

// TODO: Move into lib.rs (make decode.rs internal decoding logic)
#[derive(Debug)]
pub struct Decoder {}

impl Decoder {
    pub fn new() -> Self {
        Decoder {}
    }

    pub fn try_decode_header(&mut self, data: &[u8]) -> Result<(Header, usize), DecoderError> {
        if data.len() < HEADER_MINUMUM_REQUIRED {
            return Err(DecoderError::NeedData(HEADER_MINUMUM_REQUIRED - data.len()));
        }

        let data = &mut &data[..];
        let magic_bytes: [u8; 4] = data.read_decode().unwrap();
        if magic_bytes != MAGIC_BYTES {
            return Err(DecoderError::Invalid);
        }

        let length: u32 = data.read_decode().unwrap();
        if length as usize > data.len() {
            return Err(DecoderError::NeedData(length as usize - data.len()));
        }

        let version: u16 = data.read_decode().unwrap();

        todo!()
    }

    pub fn try_decode_record(&mut self, data: &[u8]) -> Result<(Record, usize), DecoderError> {
        todo!()
    }
}

fn try_decode_header(data: &[u8]) -> Option<Header> {
    todo!()
}

impl Decodable for Header {
    fn bytes_needed(src: &[u8]) -> Option<usize> {
        todo!()
    }

    fn decode_from(src: &[u8]) -> Self {
        let mut src = src;
        let magic_bytes: [u8; 4] = src.read_decode().unwrap();
        if magic_bytes != MAGIC_BYTES {
            panic!(
                "{} != {}",
                str::from_utf8(&magic_bytes).unwrap(),
                str::from_utf8(&MAGIC_BYTES).unwrap()
            );
        }

        let length: u32 = src.read_decode().unwrap();
        if length as usize > src.len() {
            panic!("Need more data ({} < {})", src.len(), length);
        }

        let version: u16 = src.read_decode().unwrap();

        todo!()
    }
}

impl Decodable for CodecTable {
    fn bytes_needed(src: &[u8]) -> Option<usize> {
        todo!()
    }

    fn decode_from(src: &[u8]) -> Self {
        let mut src = src;
        let mut needed = Option::<CodecEntry>::bytes_needed(src).unwrap_or(0);
        let mut table = CodecTable::new();
        while needed > 0 {
            table.0.push(Option::<CodecEntry>::decode_from(take_slice(
                &mut src, needed,
            )));
            needed = Option::<CodecEntry>::bytes_needed(src).unwrap_or(0);
        }

        table
    }
}

impl Decodable for Option<CodecEntry> {
    fn bytes_needed(src: &[u8]) -> Option<usize> {
        if src.len() == 0 {
            None
        } else {
            Some(src[0] as usize + 1)
        }
    }

    fn decode_from(src: &[u8]) -> Self {
        if src[0] == 0 {
            None
        } else {
            let mut src = src;
            let length: u8 = src.read_decode().unwrap();
            let version: u16 = src.read_decode().unwrap();
            let codec_name = take_slice(&mut src, length as usize - 3);
            let guard: u8 = src.read_decode().unwrap();
            if guard != 0 {
                // TODO: return error
                panic!("guard is not null");
            }

            // TODO: return error
            Some(CodecEntry::new_from_ascii(
                version,
                AsciiString::from_ascii(codec_name).unwrap(),
            ))
        }
    }
}

impl<'a> FromByteSlice<'a> for CodecEntry {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
        let mut input = input;
        let length = input.decode::<u8>()? as usize;
        let version = input.decode::<u16>()?;
        let name = input.decode_len::<&str>(length)?;
        let _ = input
            .decode_assert::<u8>(0)?
            .ok_or(DecodeError::InvalidGuard);

        Ok((input, CodecEntry::new(version, name)))
    }

    fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self> {
        let length = *input.get(0).ok_or(DecodeError::Needed(1))? as usize;
        if length < input.len() {
            Self::from_bytes(input)
        } else {
            Err(DecodeError::Needed(length))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_header() {
        let mut data = Vec::new();
        // Header
        data.extend_from_slice(&MAGIC_BYTES); // Magic bytes
        data.extend_from_slice(&43_u32.to_le_bytes()); // Length
        data.extend_from_slice(&0_u16.to_le_bytes()); // Version
        data.extend_from_slice(&0_u16.to_le_bytes()); // Flags
        data.extend_from_slice(&3_u16.to_le_bytes()); // Codec Entries
                                                      // Codec 1
        data.extend_from_slice(&6_u8.to_le_bytes()); // Length
        data.extend_from_slice(&1_u16.to_le_bytes()); // Version
        data.extend_from_slice(b"TEST"); // Name
        data.extend_from_slice(&0_u8.to_le_bytes()); // Guard
                                                     // Codec 2
        data.extend_from_slice(&0_u8.to_le_bytes()); // Length
                                                     // Codec 3
        data.extend_from_slice(&26_u8.to_le_bytes()); // Length
        data.extend_from_slice(&0_u16.to_le_bytes()); // Version
        data.extend_from_slice(b"SomeLongStringThatIsLong"); // Name
        data.extend_from_slice(&0_u8.to_le_bytes()); // Guard

        let mut decoder = Decoder::new();
        decoder.try_decode_header(&data).unwrap();
    }
}
