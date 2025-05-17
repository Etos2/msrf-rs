use std::ascii::Char as AsciiChar;

use crate::{
    data::{CodecEntry, CodecTable, Header, Record},
    error::{DecodeError, DecodeResult},
    io::{DecodeExt, DecodeSlice, PVarint},
    MAGIC_BYTES,
};

// TODO: Move into lib.rs? (make decode.rs internal decoding logic)
#[derive(Debug)]
pub struct Decoder {}

impl<'a> DecodeSlice<'a> for Header {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let mut input = input;
        let _magic_bytes = input
            .decode_assert::<[u8; 4]>(MAGIC_BYTES)?
            .ok_or(DecodeError::Badness)?;
        let _length = input.decode::<u32>()? as usize;
        let version = input.decode::<u16>()?;
        let flags = input.decode::<u16>()?.into();
        let entries = input.decode::<u16>()? as usize;

        let mut codec_table = CodecTable::new();
        for _ in 0..entries {
            codec_table.push(input.decode::<Option<CodecEntry>>()?);
        }

        Ok((
            input,
            Header {
                version,
                flags,
                codec_table,
            },
        ))
    }
}

impl<'a> DecodeSlice<'a> for Option<CodecEntry> {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let mut input = input;
        let length = input.decode::<u8>()? as usize;
        match length {
            0 => Ok((input, None)),
            1..=2 => Err(DecodeError::Badness), // TODO: Enforce minimum name len? A name of 0 is useless for identifying which decoder to use
            3.. => {
                let version = input.decode::<u16>()?;
                let name = input.decode_len::<&[AsciiChar]>(length - 3)?;
                let _guard = input
                    .decode_assert::<u8>(0)?
                    .ok_or(DecodeError::ExpectedGuard)?;

                Ok((input, Some(CodecEntry::new_ascii(version, name))))
            }
        }
    }
}

impl<'a> DecodeSlice<'a> for Record<'a> {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let mut input = input;
        let codec_id = input.decode::<PVarint>()?.into();
        if codec_id != u64::MAX {
            let type_id = input.decode::<PVarint>()?.into();
            let length = input.decode::<PVarint>()?.get() as usize;
            if length != 0 {
                let val = input.decode_len::<&[u8]>(length - 1)?;
                let _guard = input
                    .decode_assert::<u8>(0)?
                    .ok_or(DecodeError::ExpectedGuard)?;
                Ok((input, Record::new(codec_id, type_id, val)))
            } else {
                Ok((input, Record::new_empty(codec_id, type_id)))
            }
        } else {
            Ok((input, Record::new_eos()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test;

    #[test]
    fn decode_header() {
        let header = test::ref_header();
        let header_bytes = test::ref_header_bytes();

        assert_eq!(header_bytes.len(), 51);
        let mut data = header_bytes.as_slice();
        let output = data.decode::<Header>().unwrap();

        // Verify decode
        assert_eq!(header, output);

        // Verify all data is read
        assert_eq!(data.len(), 0);
    }
}
