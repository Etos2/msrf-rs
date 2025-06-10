use std::ascii::Char as AsciiChar;

use crate::{
    data::{Header, Record, RecordFlags, RecordMeta2},
    error::{EncodeError, EncodeResult},
    io::*,
    MAGIC_BYTES,
};

// TODO: Customisability
impl EncodeInto for Header {
    fn encode_into<S>(&self, dst: S) -> EncodeResult<S>
    where
        S: ByteStream,
    {
        let mut dst = dst;
        let len: usize = 14;
        if len > dst.capacity() {
            return Err(EncodeError::Needed(len));
        }

        dst.encode::<[u8; 4]>(MAGIC_BYTES)?;
        dst.encode((len as u32 - 8) as u32)?; // TODO: Truncation detection
        dst.encode(self.version)?;
        // FIXME: Guard

        Ok(dst)
    }
}

// impl EncodeIntoStateful<RecordMeta2> for RecordMeta2 {
//     fn encode_into_with<S>(&self, dst: S, val: &mut RecordMeta2) -> EncodeResult<S>
//     where
//         S: ByteStream,
//     {
//         let mut dst = dst;

//         Ok(dst)
//     }
// }

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
