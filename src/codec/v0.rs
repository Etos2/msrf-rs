use std::io::{Read, Write};

use crate::codec::{DesOptions, RawDeserialiser, RawSerialiser, SerOptions};
use crate::error::{IoError, ParserError};
use crate::io::{PVarint, ReadExt, WriteExt};
use crate::{RECORD_EOS, RecordMeta, TYPE_CONTAINER_MASK};

pub const VERSION: usize = 0;
pub const ID_LEN: usize = 4;

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
        wtr.write_u16(meta.source_id)?;

        if !meta.is_eos() {
            if let Some(c) = meta.contained() {
                wtr.write_u16(meta.type_id | TYPE_CONTAINER_MASK)?;
                wtr.write_varint(meta.length)?;
                wtr.write_u16(c)?;
            } else {
                wtr.write_u16(meta.type_id)?;
                wtr.write_varint(meta.length)?;
            }
        }

        Ok(())
    }

    fn encoded_meta_len(&self, user_len: usize) -> usize {
        // TODO: Avoid full encode when len is needed?
        let pv = PVarint::encode(user_len as u64);
        pv.len() + ID_LEN
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
        let source_id = rdr.read_u16()?;
        if source_id == RECORD_EOS {
            return Ok(RecordMeta::new_eos());
        }

        let mut type_id = rdr.read_u16()?;
        let length = rdr.read_varint()?;
        let contained = (type_id & TYPE_CONTAINER_MASK > 0)
            .then(|| rdr.read_u16())
            .transpose()?;

        type_id &= !TYPE_CONTAINER_MASK;
        Ok(RecordMeta {
            source_id,
            type_id,
            length,
            contained,
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

    // TODO: const new()
    pub(crate) const REF_RECORD_META: RecordMeta = RecordMeta {
        source_id: 16,
        type_id: 32,
        length: 6,
        contained: None,
    };

    pub(crate) const REF_RECORD_META_CONTAINER: RecordMeta = RecordMeta {
        source_id: 16,
        type_id: 32,
        length: 6,
        contained: Some(5),
    };

    pub(crate) const REF_RECORD_META_BYTES: &[u8; 5] = constcat::concat_bytes!(
        &16_u16.to_le_bytes(), // Source ID
        &32_u16.to_le_bytes(), // Type ID
        &[0b1101_u8],          // Length: PV(6)
    );

    pub(crate) const REF_RECORD_META_CONTAINER_BYTES: &[u8; 7] = constcat::concat_bytes!(
        &16_u16.to_le_bytes(),                         // Source ID
        &(32_u16 | TYPE_CONTAINER_MASK).to_le_bytes(), // Type ID
        &[0b1101_u8],                                  // Length: PV(6)
        &5_u16.to_le_bytes(),                          // Contained
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
        assert_eq!(rdr.position(), buf.len() as u64);
    }

    #[test]
    fn serdes_record_eos() {
        let des = Deserialiser::default();
        let ser = Serialiser::default();
        let mut buf = [0u8; 2];

        ser.write_meta(RecordMeta::new_eos(), buf.as_mut_slice())
            .expect("ser fail");
        assert_eq!(&buf, &RECORD_EOS.to_le_bytes());

        let mut rdr = Cursor::new(buf);
        let meta = des.read_meta(&mut rdr).expect("des fail");
        assert_eq!(meta, RecordMeta::new_eos());
        assert_eq!(rdr.position(), buf.len() as u64);
    }

    #[test]
    fn serdes_record_container() {
        let des = Deserialiser::default();
        let ser = Serialiser::default();
        let mut buf = [0u8; 7];

        ser.write_meta(REF_RECORD_META_CONTAINER, buf.as_mut_slice())
            .expect("ser fail");
        assert_eq!(&buf, REF_RECORD_META_CONTAINER_BYTES);

        let mut rdr = Cursor::new(buf);
        let meta = des.read_meta(&mut rdr).expect("des fail");
        assert_eq!(meta, REF_RECORD_META_CONTAINER);
        assert_eq!(rdr.position(), buf.len() as u64);
    }
}
