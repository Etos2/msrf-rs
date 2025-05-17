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
    use crate::data::HeaderFlags;

    #[test]
    fn decode_header() {
        let mut data = Vec::new();
        // Header
        data.extend_from_slice(&MAGIC_BYTES); // Magic bytes
        data.extend_from_slice(&43_u32.to_le_bytes()); // Length
        data.extend_from_slice(&0_u16.to_le_bytes()); // Version
        data.extend_from_slice(&0_u16.to_le_bytes()); // Flags
                                                      // Codec Table
        data.extend_from_slice(&3_u16.to_le_bytes()); // Codec Entries
                                                      // Codec Entry 1
        data.extend_from_slice(&7_u8.to_le_bytes()); // Length
        data.extend_from_slice(&1_u16.to_le_bytes()); // Version
        data.extend_from_slice(b"TEST"); // Name
        data.extend_from_slice(&0_u8.to_le_bytes()); // Guard
                                                     // Codec Entry 2 (empty)
        data.extend_from_slice(&0_u8.to_le_bytes()); // Length
                                                     // Codec Entry 3
        data.extend_from_slice(&27_u8.to_le_bytes()); // Length
        data.extend_from_slice(&(u16::MAX).to_le_bytes()); // Version
        data.extend_from_slice(b"SomeLongStringThatIsLong"); // Name
        data.extend_from_slice(&0_u8.to_le_bytes()); // Guard

        assert_eq!(data.len(), 51);
        let mut data = data.as_slice();
        let header = data.decode::<Header>().unwrap();

        // Verify decode
        assert_eq!(
            header,
            Header {
                version: 0,
                flags: HeaderFlags::empty(),
                codec_table: CodecTable::from(vec![
                    Some(CodecEntry {
                        version: 1,
                        name: b"TEST".as_ascii().unwrap().to_vec(),
                    }),
                    None,
                    Some(CodecEntry {
                        version: u16::MAX,
                        name: b"SomeLongStringThatIsLong".as_ascii().unwrap().to_vec(),
                    })
                ])
            }
        );

        // Verify all data is read
        assert_eq!(data.len(), 0);
    }
}
