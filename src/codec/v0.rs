use std::io::{Read, Write};

use crate::RecordMeta;
use crate::codec::{DesOptions, RawDeserialiser, RawSerialiser, SerOptions, varint};
use crate::error::{IoError, ParserError};

pub const VERSION: usize = 0;
pub const HEADER_LEN: usize = 2;
pub const RECORD_META_LEN: usize = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Serialiser {
    options: SerOptions,
}

impl RawSerialiser for Serialiser {
    fn write_meta(
        &self,
        meta: RecordMeta,
        mut wtr: impl Write,
    ) -> Result<(), IoError<ParserError>> {
        wtr.write_all(&meta.source_id.to_le_bytes())?;
        wtr.write_all(&meta.type_id.to_le_bytes())?;
        let varint_bytes = varint::to_le_bytes(meta.length);
        let varint_len = varint::len(varint_bytes[0]);
        wtr.write_all(&varint_bytes[..varint_len])?;

        Ok(())
    }
}

impl From<SerOptions> for Serialiser {
    fn from(options: SerOptions) -> Self {
        Serialiser { options }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Deserialiser {
    options: DesOptions,
}

impl RawDeserialiser for Deserialiser {
    fn read_meta(&self, mut rdr: impl Read) -> Result<RecordMeta, IoError<ParserError>> {
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

impl From<DesOptions> for Deserialiser {
    fn from(options: DesOptions) -> Self {
        Deserialiser { options }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::io::Cursor;

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

    #[test]
    fn serdes_record() {
        let des = Deserialiser::default();
        let ser = Serialiser::default();
        let mut buf = [0u8; 5];

        ser.write_meta(REF_RECORD_META, buf.as_mut_slice())
            .expect("ser fail");
        assert_eq!(&buf, REF_RECORD_META_BYTES);

        let mut rdr = Cursor::new(buf);
        let meta = des.read_meta(&mut rdr).expect("des fail");
        assert_eq!(meta, REF_RECORD_META);
        assert_eq!(rdr.position(), 5);
    }
}
