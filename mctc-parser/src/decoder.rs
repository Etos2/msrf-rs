use crate::{
    data::{Header, RecordMeta},
    error::{DecodeError, DecodeResult},
    io::{util::{Guard, PVarint}, DecodeExt, DecodeInto},
    MAGIC_BYTES,
};

// TODO: Move into lib.rs? (make decode.rs internal decoding logic)
#[derive(Debug)]
pub struct Decoder {}

impl<'a> DecodeInto<'a> for Header {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let mut input = input;
        let _magic_bytes = input
            .decode_assert::<[u8; 4]>(MAGIC_BYTES)?
            .ok_or(DecodeError::Badness)?;
        let length = input.decode::<PVarint>()?.get();
        let version = input.decode::<(u8, u8)>()?; // TODO: handle versions (especially MAJOR version)

        // Skip additional header data
        if length > 3 {
            input.skip(length as usize - 3)?;
        }

        let _guard = input
            .decode_assert::<u8>(Guard::from(length).get())?
            .ok_or(DecodeError::Badness)?;

        Ok((input, Header { version }))
    }
}

impl<'a> DecodeInto<'a> for RecordMeta {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let mut input = input;

        // TODO: Handle len invariance (EOS > InvalidLen >= 4)
        let length = input.decode::<PVarint>()?.get() as usize;
        if length != 0 {
            let source_id = input.decode::<u16>()?;
            let type_id = input.decode::<u16>()?;

            Ok((
                input,
                RecordMeta {
                    length,
                    source_id,
                    type_id,
                },
            ))
        } else {
            Ok((input, RecordMeta::new_eos()))
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

        assert_eq!(header_bytes.len(), 8);
        let mut data = header_bytes.as_slice();
        let output = data.decode::<Header>().unwrap();

        // Verify decode
        assert_eq!(header, output);

        // Verify all data is read
        assert_eq!(data.len(), 0);
    }
}
