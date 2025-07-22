use msrf_io::error::{CodecError, CodecResult};
use msrf_io::{ByteStream, MutByteStream};

use crate::{
    codec::{
        RawSerialiser,
        constants::{HEADER_CONTENTS, HEADER_LEN, MAGIC_BYTES, RECORD_EOS, RECORD_META_MIN_LEN},
    },
    data::{Header, RecordMeta},
};

pub struct Serialiser;

impl RawSerialiser for Serialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        let len = buf.len();
        let mut buf = buf;

        if HEADER_LEN > len {
            return Err(CodecError::Needed(HEADER_LEN - len));
        }

        buf.insert(&MAGIC_BYTES)?;
        buf.insert_varint(HEADER_CONTENTS as u64)?;
        buf.insert_u8(header.version.0)?;
        buf.insert_u8(header.version.1)?;
        buf.insert_u8(0x00)?;

        Ok(len - buf.len())
    }

    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        let len = buf.len();
        let mut buf = buf;

        let record_len = meta.length;
        buf.insert_varint(record_len)?;
        if record_len > RECORD_META_MIN_LEN {
            buf.insert_u16(meta.source_id)?;
            buf.insert_u16(meta.type_id)?;
        } else if record_len != RECORD_EOS {
            return Err(CodecError::Length(record_len));
        }

        Ok(len - buf.len())
    }

    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)> {
        let len = buf.len();
        let mut buf = buf;

        if HEADER_LEN > buf.len() {
            return Err(CodecError::Needed(HEADER_LEN - buf.len()));
        }

        // SAFETY: [u8; 4].len() == 4
        let magic_bytes = buf.extract(4)?.try_into().unwrap();
        if magic_bytes != MAGIC_BYTES {
            return Err(CodecError::MagicByte(magic_bytes));
        }

        let length = buf.extract_varint()?;
        let major = buf.extract_u8()?;
        let minor = buf.extract_u8()?;

        // Skip additional header data
        if let Some(unknown) = (length).checked_sub(HEADER_CONTENTS) {
            buf.skip(unknown as usize)?;
        }

        if buf.extract_u8()? != 0 {
            return Err(CodecError::Guard);
        }

        Ok((
            Header {
                version: (major, minor),
            },
            len - buf.len(),
        ))
    }

    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)> {
        let len = buf.len();
        let mut buf = buf;

        let length = buf.extract_varint()?;
        match length {
            // 0 = End Of Stream indicator
            0 => Ok((RecordMeta::new_eos(), len - buf.len())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_MIN_LEN => Err(CodecError::Length(length)),
            // Contains contents + zero/some data
            RECORD_META_MIN_LEN.. => {
                let source_id = buf.extract_u16()?;
                let type_id = buf.extract_u16()?;

                Ok((
                    RecordMeta {
                        length,
                        source_id,
                        type_id,
                    },
                    len - buf.len(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const REF_HEADER: Header = Header { version: (1, 2) };
    const REF_HEADER_BYTES: &[u8; 8] = constcat::concat_bytes!(
        &MAGIC_BYTES,  // Magic bytes
        &[0b111_u8],   // Length Pvarint(3)
        &[1_u8, 2_u8], // Version (Major, Minor)
        &[0x00]        // Guard
    );

    const REF_RECORD_META: RecordMeta = RecordMeta {
        length: 6,
        source_id: 16,
        type_id: 32,
    };
    const REF_RECORD_META_BYTES: &[u8; 5] = constcat::concat_bytes!(
        &[0b1101_u8],          // Length
        &16_u16.to_le_bytes(), // Source ID
        &32_u16.to_le_bytes(), // Source ID
    );

    #[test]
    fn serialise_header() {
        let ser = Serialiser {};
        let (out, read) = ser.deserialise_header(REF_HEADER_BYTES.as_slice()).unwrap();

        assert_eq!(REF_HEADER, out);
        assert_eq!(REF_HEADER_BYTES.len(), read);

        let mut buf = [0; 8];
        let written = ser.serialise_header(&mut buf, &REF_HEADER).unwrap();

        assert_eq!(REF_HEADER_BYTES, &buf);
        assert_eq!(buf.len(), written);
    }

    #[test]
    fn serialise_record_meta() {
        let ser = Serialiser {};
        let (out, read) = ser
            .deserialise_record_meta(REF_RECORD_META_BYTES.as_slice())
            .unwrap();

        assert_eq!(REF_RECORD_META, out);
        assert_eq!(REF_RECORD_META_BYTES.len(), read);

        let mut buf = [0; 5];
        let written = ser
            .serialise_record_meta(&mut buf, &REF_RECORD_META)
            .unwrap();

        assert_eq!(REF_RECORD_META_BYTES, &buf);
        assert_eq!(buf.len(), written);
    }
}
