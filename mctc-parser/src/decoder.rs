use std::ascii::Char as AsciiChar;

use crate::{
    data::{Header, Record, RecordFlags, RecordMeta, RecordMeta2},
    error::{DecodeError, DecodeResult},
    io::{DecodeExt, DecodeSlice, Guard, PVarint},
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
        let length = input.decode::<PVarint>()?.get() as usize;
        let version = input.decode::<(u8, u8)>()?;
        let guard = input.decode::<u8>()?;

        if Guard::from(length).get() != guard {
            return Err(DecodeError::Badness);
        }

        Ok((input, Header { version }))
    }
}

impl<'a> DecodeSlice<'a> for RecordMeta2 {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let mut input = input;

        let length = input.decode::<PVarint>()?.get() as usize;
        let flags = RecordFlags::from_bits_truncate(length as u8);

        let source_id = if flags.contains(RecordFlags::SOURCE_INHERIT) {
            todo!()
        } else {
            input.decode::<u16>()?
        };

        let type_id = if flags.contains(RecordFlags::TYPE_INHERIT) {
            todo!()
        } else {
            input.decode::<u16>()?
        };

        Ok((
            input,
            RecordMeta2 {
                source_id,
                type_id,
                length,
            },
        ))
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
