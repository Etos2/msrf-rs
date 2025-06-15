use crate::{
    data::{Header, RecordMeta},
    error::{EncodeError, EncodeResult},
    io::{util::{Guard, PVarint}, *},
    MAGIC_BYTES, RECORD_LENGTH_EOS,
};

pub const HEADER_LENGTH: usize = 8;

// TODO: Customisability? (Version, Additional data, etc)
impl EncodeInto for Header {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
        let mut dst = dst;
        if HEADER_LENGTH > dst.len() {
            return Err(EncodeError::Needed(HEADER_LENGTH - dst.len()));
        }

        let header_len = HEADER_LENGTH as u64 - 4; // Exlusive of MagicBytes & Length
        dst.encode::<[u8; 4]>(MAGIC_BYTES)?;
        dst.encode(PVarint::from(header_len))?;
        dst.encode(self.version)?;
        dst.encode(Guard::from(header_len).get())?;

        Ok(dst)
    }
}

impl EncodeInto for RecordMeta {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
        let mut dst = dst;
        let len = self.length as u64;

        dst.encode(PVarint::from(len))?;
        if len != RECORD_LENGTH_EOS {
            dst.encode(self.source_id)?;
            dst.encode(self.type_id)?;
        }

        Ok(dst)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test;

    #[test]
    fn encode_header() {
        let header = test::ref_header();
        let header_bytes = test::ref_header_bytes();

        let mut output_bytes = [0; 51];
        let mut buf = &mut output_bytes[..];
        buf.encode::<Header>(&header).unwrap();

        // Verify all data is written
        assert_eq!(buf.len(), 0);

        // Verify encode
        assert_eq!(header_bytes, output_bytes);
    }
}
