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
    use crate::data::HeaderFlags;

    #[test]
    fn encode_header() {
        let header = Header {
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
                }),
            ]),
        };

        let mut buf = [0; 51];
        let mut dst = &mut buf[..];
        dst.encode::<Header>(&header).unwrap();
        assert_eq!(dst.len(), 0);

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

        assert_eq!(data, buf);
    }
}
