use crate::{
    data::{CodecEntry, CodecTable, Header},
    io::*,
    MAGIC_BYTES,
};

// TODO: Customisability
// struct MctcEncoder {}

// impl Encoder for MctcEncoder {}

impl Encodable for Header {
    fn len_needed(&self) -> usize {
        14 + self.codec_table.len_needed()
    }

    fn encode_into(&self, dst: &mut [u8]) {
        let mut dst = dst;
        dst.write_encodable(MAGIC_BYTES);
        dst.write_encodable((self.len_needed() - 8) as u32);
        dst.write_encodable(self.version);
        dst.write_encodable(self.flags.into_inner());
        dst.write_encodable(self.codec_table.len_needed() as u16);
        dst.write_encodable(&self.codec_table);
    }
}

impl Encodable for CodecTable {
    fn len_needed(&self) -> usize {
        self.as_ref().iter().map(Encodable::len_needed).sum()
    }

    fn encode_into(&self, dst: &mut [u8]) {
        let mut dst = dst;
        for opt_entry in self.as_ref() {
            dst.write_encodable(opt_entry);
        }
    }
}

impl Encodable for Option<CodecEntry> {
    fn len_needed(&self) -> usize {
        self.as_ref().map(Encodable::len_needed).unwrap_or(1)
    }

    fn encode_into(&self, dst: &mut [u8]) {
        let mut dst = dst;
        match self {
            Some(entry) => dst.write_encodable(entry),
            None => dst.write_encodable(0u8),
        }
    }
}

impl Encodable for CodecEntry {
    fn len_needed(&self) -> usize {
        4 + self.name.len()
    }

    fn encode_into(&self, dst: &mut [u8]) {
        let mut dst = dst;
        let len = u8::try_from(self.len_needed() - 1).unwrap();
        dst.write_encodable(len);
        dst.write_encodable(self.version);
        dst.write_encodable(self.name.as_str());
        dst.write_encodable(0u8);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::HeaderFlags;
    use crate::util::AsciiCharExt;
    use std::ascii::Char as AsciiChar;

    #[test]
    fn encode_header() {
        let codec_table = CodecTable(vec![
            Some(CodecEntry {
                version: 1,
                name: <[AsciiChar]>::new(b"Test").to_owned(),
            }),
            None,
            Some(CodecEntry {
                version: 2,
                name: <[AsciiChar]>::new(b"SomeLongStringThatIsLong").to_owned(),
            }),
        ]);
        let header = Header {
            version: 0x00,
            flags: HeaderFlags::empty(),
            codec_table,
        };

        let codec_entries = header.codec_table.as_ref();
        let mut buf = [0; 51];
        let mut dst = &mut buf[..];
        dst.write_encodable_checked(&header)
            .expect("buffer too small");
        assert_eq!(dst.len(), 0);

        // Header
        assert_eq!(buf[0..4], MAGIC_BYTES);
        assert_eq!(buf[4..8], 43_u32.to_le_bytes());
        assert_eq!(buf[8..10], header.version.to_le_bytes());
        assert_eq!(buf[10..12], header.flags.into_inner().to_le_bytes());
        assert_eq!(
            buf[12..14],
            (header.codec_table.len_needed() as u16).to_le_bytes()
        );

        // Codec Entry 1
        assert_eq!(
            buf[14..15],
            (codec_entries[0].len_needed() as u8 - 1).to_le_bytes()
        );
        assert_eq!(
            buf[15..17],
            codec_entries[0].clone().unwrap().version.to_le_bytes()
        );
        assert_eq!(buf[17..21], *b"Test");
        assert_eq!(buf[21..22], [0]);

        // Codec Entry 2
        assert_eq!(buf[22..23], [0]);

        // Codec Entry 1
        assert_eq!(
            buf[23..24],
            (codec_entries[2].len_needed() as u8 - 1).to_le_bytes()
        );
        assert_eq!(
            buf[24..26],
            codec_entries[2].clone().unwrap().version.to_le_bytes()
        );
        assert_eq!(buf[26..50], *b"SomeLongStringThatIsLong");
        assert_eq!(buf[50..51], [0]);
    }
}
