use msrf_io::{TakeExt, varint};

use crate::codec::{DesResult, RawDeserialiser};
use crate::reader::ParserError;
use crate::{
    codec::constants::{RECORD_EOS, RECORD_META_MIN_LEN},
    data::RecordMeta,
};

#[derive(Debug, Clone, Default)]
pub struct Deserialiser;

impl RawDeserialiser for Deserialiser {
    fn deserialise_record_meta(&self, buf: &[u8]) -> DesResult<RecordMeta> {
        let len = buf.len();
        let mut buf = buf;

        // TODO: Assert
        // let length_len = varint::len(buf[0]);
        // let length = varint::from_le_bytes(buf.take_slice(length_len).ok_or_else(|| ParserError::Need(todo!()))?);
        // let major = u8::from_le_bytes(buf.take_chunk().ok_or_else(|| ParserError::Need(todo!()))?);
        // let minor = u8::from_le_bytes(buf.take_chunk().ok_or_else(|| ParserError::Need(todo!()))?);
        let tag = varint::len(buf[0]);
        let length = varint::from_le_bytes(
            buf.take_slice(tag)
                .ok_or_else(|| ParserError::Need(todo!()))?,
        );
        match length {
            // 0 = End Of Stream indicator
            RECORD_EOS => Ok((RecordMeta::new_eos(), len - buf.len())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_MIN_LEN => Err(ParserError::Length(length)),
            // Contains contents + zero/some data
            RECORD_META_MIN_LEN.. => {
                let source_id =
                    u16::from_le_bytes(buf.take_chunk().ok_or_else(|| ParserError::Need(todo!()))?);
                let type_id =
                    u16::from_le_bytes(buf.take_chunk().ok_or_else(|| ParserError::Need(todo!()))?);

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

        let guard = u8::from_le_bytes(buf.take_chunk().ok_or_else(|| ParserError::Need(todo!()))?);
        if guard == 0 {
            Ok(((), len - buf.len()))
        } else {
            Err(ParserError::Guard(guard))
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{codec::constants::MAGIC_BYTES, data::Header};

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
}
