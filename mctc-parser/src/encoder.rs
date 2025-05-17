use std::ascii::Char as AsciiChar;

use crate::{
    data::{CodecEntry, CodecTable, Header},
    error::{EncodeError, EncodeResult},
    io::*,
    MAGIC_BYTES,
};

// TODO: Customisability
// struct MctcEncoder {}

// impl Encoder for MctcEncoder {}

// impl Encodable for Header {
//     fn len_needed(&self) -> usize {
//         14 + self.codec_table.len_needed()
//     }

//     fn encode_into(&self, dst: &mut [u8]) {
//         let mut dst = dst;
//         dst.write_encodable(MAGIC_BYTES);
//         dst.write_encodable((self.len_needed() - 8) as u32);
//         dst.write_encodable(self.version);
//         dst.write_encodable(self.flags.into_inner());
//         dst.write_encodable(self.codec_table.len_needed() as u16);
//         dst.write_encodable(&self.codec_table);
//     }
// }

// impl Encodable for CodecTable {
//     fn len_needed(&self) -> usize {
//         self.as_ref().iter().map(Encodable::len_needed).sum()
//     }

//     fn encode_into(&self, dst: &mut [u8]) {
//         let mut dst = dst;
//         for opt_entry in self.as_ref() {
//             dst.write_encodable(opt_entry);
//         }
//     }
// }

impl EncodeSlice for Header {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
        let mut dst = dst;
        let len: usize = 14
            + self
                .codec_table
                .as_ref()
                .iter()
                .map(|opt_entry| {
                    opt_entry
                        .as_ref()
                        .map(|entry| entry.name.len() + 4)
                        .unwrap_or(1)
                })
                .sum::<usize>();
        if len > dst.len() {
            return Err(EncodeError::Needed(len));
        }

        dst.encode::<[u8; 4]>(MAGIC_BYTES)?;
        dst.encode((len as u32 - 8) as u32)?; // TODO: Truncation detection
        dst.encode(self.version)?;
        dst.encode(self.flags.into_inner())?;
        dst.encode(self.codec_table.len() as u16)?; // TODO: Truncation detection
        for opt_entry in self.codec_table.as_ref() {
            dst.encode::<Option<CodecEntry>>(opt_entry)?;
        }

        Ok(dst)
    }
}

impl EncodeSlice for Option<CodecEntry> {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
        let mut dst = dst;
        if let Some(val) = self {
            let len = 4 + val.name.len();
            if len > dst.len() {
                return Err(EncodeError::Needed(len));
            }

            dst.encode(u8::try_from(len - 1).map_err(|_| EncodeError::Badness)?)?;
            dst.encode(val.version)?;
            dst.encode::<&[AsciiChar]>(val.name.as_slice())?;
        }
        dst.encode(0u8)?;

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
