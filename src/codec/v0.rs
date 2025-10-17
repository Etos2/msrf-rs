use crate::RecordMeta;
use crate::codec::{DesOptions, RawDeserialiser, varint};
use crate::error::{IoError, ParserError};

pub const HEADER_LEN: usize = 2;
pub const RECORD_META_LEN: usize = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Deserialiser {
    options: DesOptions,
}

impl From<DesOptions> for Deserialiser {
    fn from(options: DesOptions) -> Self {
        Deserialiser { options }
    }
}

impl RawDeserialiser for Deserialiser {
    const VERSION: usize = 0;

    fn read_record(&self, mut rdr: impl std::io::Read) -> Result<RecordMeta, IoError<ParserError>> {
        let mut buf = [0; 13];
        rdr.read_exact(&mut buf[..5])?;
        let source_id = u16::from_le_bytes(buf[..2].try_into().unwrap()); // Safety: buf[..2].len() == 2
        let type_id = u16::from_le_bytes(buf[2..4].try_into().unwrap()); // Safety: buf[2..4].len() == 2
        let length_len = varint::len(buf[4]);
        if length_len > 1 {
            rdr.read_exact(&mut buf[4..4 + length_len - 1])?;
        }
        let length = varint::from_le_bytes(&buf[4..]);

        Ok(RecordMeta {
            length,
            source_id,
            type_id,
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub(crate) const REF_RECORD_META: RecordMeta = RecordMeta {
        length: 6,
        source_id: 16,
        type_id: 32,
    };
    pub(crate) const REF_RECORD_META_BYTES: &[u8; 5] = constcat::concat_bytes!(
        &16_u16.to_le_bytes(), // Source ID
        &32_u16.to_le_bytes(), // Type ID
        &[0b1101_u8],          // Length: PV(6)
    );
}
