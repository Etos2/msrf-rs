use crate::{
    data::{CodecEntry, CodecTable, Header}, error::DecodeError, io::{
        DecodeExt, FromByteResult, FromByteSlice,
    }, MAGIC_BYTES
};


// TODO: Move into lib.rs? (make decode.rs internal decoding logic)
#[derive(Debug)]
pub struct Decoder {}

impl<'a> FromByteSlice<'a> for Header {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
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

    fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self> {
        let length_slice = input.get(4..8).ok_or(DecodeError::Needed(8))?;
        let needed = length_slice.decode_peek_checked::<u32>()? as usize;
        if input.len() >= needed {
            Self::from_bytes(input)
        } else {
            Err(DecodeError::Needed(needed))
        }
    }
}

impl<'a> FromByteSlice<'a> for Option<CodecEntry> {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
        let mut input = input;
        let length = input.decode::<u8>()? as usize;
        match length {
            0 => Ok((input, None)),
            1..=2 => Err(DecodeError::Badness), // TODO: Enforce minimum name len? A name of 0 is useless for identifying which decoder to use
            3.. => {
                let version = input.decode::<u16>()?;
                let name = input.decode_len::<&str>(length - 2)?;
                let _guard = input
                    .decode_assert::<u8>(0)?
                    .ok_or(DecodeError::ExpectedGuard)?;

                Ok((input, Some(CodecEntry::new(version, name))))
            }
        }
    }

    fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self> {
        let needed = input.decode_peek_checked::<u8>()? as usize;
        if input.len() >= needed {
            Self::from_bytes(input)
        } else {
            Err(DecodeError::Needed(needed))
        }
    }
}

#[cfg(test)]
mod test {
    use std::ascii::Char as AsciiChar;
    use crate::data::HeaderFlags;
    use crate::util::AsciiCharExt;
    use super::*;

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
        data.extend_from_slice(&6_u8.to_le_bytes()); // Length
        data.extend_from_slice(&1_u16.to_le_bytes()); // Version
        data.extend_from_slice(b"TEST"); // Name
        data.extend_from_slice(&0_u8.to_le_bytes()); // Guard
        // Codec Entry 2 (empty)
        data.extend_from_slice(&0_u8.to_le_bytes()); // Length
        // Codec Entry 3
        data.extend_from_slice(&26_u8.to_le_bytes()); // Length
        data.extend_from_slice(&(u16::MAX).to_le_bytes()); // Version
        data.extend_from_slice(b"SomeLongStringThatIsLong"); // Name
        data.extend_from_slice(&0_u8.to_le_bytes()); // Guard

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
                        name: <[AsciiChar]>::from_bytes_owned(b"TEST").unwrap(),
                    }),
                    None,
                    Some(CodecEntry {
                        version: u16::MAX,
                        name: <[AsciiChar]>::from_bytes_owned(b"SomeLongStringThatIsLong").unwrap(),
                    })
                ])
            }
        );

        // Verify all data is read
        assert_eq!(data.len(), 0);
    }
}
