use msrf_io::error::{CodecError, CodecResult};
use msrf_io::{ByteStream, MutByteStream, varint};

use crate::codec::{DesResult, RawDeserialiser};
use crate::reader::ParserError;
use crate::{
    codec::{
        RawSerialiser,
        constants::{HEADER_CONTENTS, HEADER_LEN, MAGIC_BYTES, RECORD_EOS, RECORD_META_MIN_LEN},
    },
    data::{Header, RecordMeta},
};

#[derive(Debug, Clone, Default)]
pub struct Deserialiser;

impl RawDeserialiser for Deserialiser {
    fn deserialise_record_meta(&self, buf: &[u8]) -> DesResult<RecordMeta> {
        let len = buf.len();
        let mut buf = buf;

        // TODO: Assert
        let tag = varint::len(*buf.get(0).ok_or(ParserError::Need(1))?);
        let length = varint::from_le_bytes(buf.extract_slice_checked(tag).map_err(ParserError::Need)?);
        match length {
            // 0 = End Of Stream indicator
            RECORD_EOS => Ok((RecordMeta::new_eos(), len - buf.len())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_MIN_LEN => Err(ParserError::Length(length)),
            // Contains contents + zero/some data
            RECORD_META_MIN_LEN.. => {
                let source_id = u16::from_le_bytes(buf.extract_checked().map_err(ParserError::Need)?);
                let type_id = u16::from_le_bytes(buf.extract_checked().map_err(ParserError::Need)?);

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

    fn deserialise_guard(&self, buf: &[u8]) -> DesResult<()> {
        let len = buf.len();
        let mut buf = buf;

        let guard = u8::from_le_bytes(buf.extract_checked().map_err(ParserError::Need)?);
        if guard == 0 {
            Ok(((), len - buf.len()))
        } else {
            Err(ParserError::Guard(guard))
        }
    }
}

pub struct Serialiser;

impl RawSerialiser for Serialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        let len = buf.len();
        let mut buf = buf;

        if HEADER_LEN > len {
            return Err(CodecError::Needed(HEADER_LEN - len));
        }

        buf.insert_slice(&MAGIC_BYTES);
        let length_varint = varint::to_le_bytes(HEADER_CONTENTS);
        let length_varint_len = varint::len(length_varint[0]);
        buf.insert_slice(&length_varint[..length_varint_len]);
        buf.insert(u8::to_le_bytes(header.version.0));
        buf.insert(u8::to_le_bytes(header.version.1));
        buf.insert(u8::to_le_bytes(0x00));

        Ok(len - buf.len())
    }

    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        let len = buf.len();
        let mut buf = buf;

        let length_varint = varint::to_le_bytes(meta.length);
        let length_varint_len = varint::len(length_varint[0]);
        buf.insert_slice(&length_varint[..length_varint_len]);
        if meta.length > RECORD_META_MIN_LEN {
            buf.insert(u16::to_le_bytes(meta.source_id));
            buf.insert(u16::to_le_bytes(meta.type_id));
        } else if meta.length != RECORD_EOS {
            return Err(CodecError::Length(meta.length));
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
        let magic_bytes = buf.extract();
        if magic_bytes != MAGIC_BYTES {
            return Err(CodecError::MagicByte(magic_bytes));
        }
        let length_len = varint::len(buf[0]);
        let length = varint::from_le_bytes(buf.extract_slice(length_len));
        let major = u8::from_be_bytes(buf.extract());
        let minor = u8::from_be_bytes(buf.extract());

        // Skip additional header data
        let remainder = (length).checked_sub(HEADER_CONTENTS).unwrap_or(0) as usize;
        if remainder > 0 {
            buf = &buf[remainder as usize..];
        }

        if u8::from_be_bytes(buf.extract()) != 0 {
            return Err(CodecError::Guard);
        }

        Ok((
            Header {
                length,
                version: (major, minor),
            },
            len - buf.len(),
        ))
    }

    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)> {
        let len = buf.len();
        let mut buf = buf;

        let length_len = varint::len(buf[0]);
        let length = varint::from_le_bytes(buf.extract_slice(length_len));
        match length {
            // 0 = End Of Stream indicator
            0 => Ok((RecordMeta::new_eos(), len - buf.len())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_MIN_LEN => Err(CodecError::Length(length)),
            // Contains contents + zero/some data
            RECORD_META_MIN_LEN.. => {
                let source_id = u16::from_le_bytes(buf.extract());
                let type_id = u16::from_le_bytes(buf.extract());

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
pub(crate) mod test {
    use super::*;

    pub(crate) const REF_HEADER: Header = Header {
        length: 3,
        version: (1, 2),
    };

    pub(crate) const REF_HEADER_BYTES: &[u8; 8] = constcat::concat_bytes!(
        &MAGIC_BYTES,  // Magic bytes
        &[0b111_u8],   // Length Pvarint(3)
        &[1_u8, 2_u8], // Version (Major, Minor)
        &[0x00]        // Guard
    );

    pub(crate) const REF_RECORD_META: RecordMeta = RecordMeta {
        length: 6,
        source_id: 16,
        type_id: 32,
    };
    pub(crate) const REF_RECORD_META_BYTES: &[u8; 5] = constcat::concat_bytes!(
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
