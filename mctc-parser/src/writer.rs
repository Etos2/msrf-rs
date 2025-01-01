use std::io::Write;

use crate::{
    data::{Header, Record},
    error::PResult,
    CODEC_ID_EOS, MAGIC_BYTES,
};

pub fn parse_header(mut wtr: impl Write, header: Header) -> PResult<()> {
    write_header(&mut wtr, header)
}

pub fn parse_record(mut wtr: impl Write, record: Record) -> PResult<()> {
    write_record(&mut wtr, record)
}

trait WriteExt {
    fn write_null(&mut self) -> PResult<()>;
    fn write_u8(&mut self, data: u8) -> PResult<()>;
    fn write_u16(&mut self, data: u16) -> PResult<()>;
    fn write_u32(&mut self, data: u32) -> PResult<()>;
}

impl<W: Write> WriteExt for W {
    #[inline]
    fn write_null(&mut self) -> PResult<()> {
        Ok(self.write_all(&[0x00])?)
    }

    #[inline]
    fn write_u8(&mut self, data: u8) -> PResult<()> {
        Ok(self.write_all(&data.to_le_bytes())?)
    }

    #[inline]
    fn write_u16(&mut self, data: u16) -> PResult<()> {
        Ok(self.write_all(&data.to_le_bytes())?)
    }

    #[inline]
    fn write_u32(&mut self, data: u32) -> PResult<()> {
        Ok(self.write_all(&data.to_le_bytes())?)
    }
}

fn write_header(mut wtr: impl Write, header: Header) -> PResult<()> {
    wtr.write_all(&MAGIC_BYTES)?;
    wtr.write_u16(header.version)?;
    wtr.write_u16(header.flags.bits())?;
    wtr.write_u16(header.codec_table.len() as u16)?;

    // TODO: Cheaper alternative to sort? Currently used for consistent tests
    let mut keys: Vec<_> = header.codec_table.iter().collect();
    keys.sort_by_key(|(k, _)| *k);
    for (k, v) in keys {
        let len = 2 + v.name.len();
        wtr.write_u8(len as u8)?;
        wtr.write_u16(*k)?;
        wtr.write_all(v.name.as_bytes())?;
        wtr.write_null()?;
    }

    Ok(())
}

fn write_record(mut wtr: impl Write, record: Record) -> PResult<()> {
    wtr.write_u16(record.codec_id)?;
    if record.codec_id != CODEC_ID_EOS {
        wtr.write_u16(record.type_id)?;
        if let Some(val) = record.val {
            wtr.write_u32(val.len() as u32)?;
            wtr.write_all(val)?;
            wtr.write_null()?;
        } else {
            wtr.write_u32(0)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, io::Cursor};

    use crate::data::{CodecEntry, HeaderFlags, HeaderOwned, RecordOwned};

    use super::*;

    #[test]
    fn test_header() {
        let mut buf = [0u8; 86];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let header_data = HeaderOwned {
            version: 0,
            flags: HeaderFlags::empty(),
            codec_table: HashMap::from([
                (
                    18,
                    CodecEntry {
                        name: String::from("TEST"),
                    },
                ),
                (
                    261,
                    CodecEntry {
                        name: String::from_utf8(vec![b'A'; 64]).unwrap(),
                    },
                ),
            ]),
        };

        let mut output = Vec::new();
        output.extend_from_slice(&MAGIC_BYTES); //       Magic Bytes "MCTC"
        output.extend_from_slice(&[0x00, 0x00]); //      Version
        output.extend_from_slice(&[0x00, 0x00]); //      Flags (Unused)
        output.extend_from_slice(&[0x02, 0x00]); //      Codec Entries

        output.extend_from_slice(&[0x06]); //            Length
        output.extend_from_slice(&[0x12, 0x00]); //      CodecID
        output.extend_from_slice(b"TEST"); //            Name
        output.extend_from_slice(&[0x00]); //            Guard (null byte)

        output.extend_from_slice(&[0x42]); //            Length
        output.extend_from_slice(&[0x05, 0x01]); //      CodecID
        output.extend_from_slice(&vec![b'A'; 64]); //    Name
        output.extend_from_slice(&[0x00]); //            Guard (null byte)

        assert!(write_header(&mut wtr, header_data.as_ref()).is_ok());
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record() {
        let mut buf = [0u8; 264];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = RecordOwned {
            codec_id: 18,
            type_id: 1,
            val: Some(vec![0; 0xFF].into()),
        };

        let mut output = Vec::new();
        output.extend_from_slice(&[0x12, 0x00]); //              CodecID
        output.extend_from_slice(&[0x01, 0x00]); //              TypeID
        output.extend_from_slice(&[0xFF, 0x00, 0x00, 0x00]); //  Length
        output.extend_from_slice(&vec![0; 255]); //              Value
        output.extend_from_slice(&[0x00]); //                    Guard

        assert!(write_record(&mut wtr, record_data.as_ref()).is_ok());
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record_empty() {
        let mut buf = [0u8; 8];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = RecordOwned {
            codec_id: 18,
            type_id: 1,
            val: None,
        };

        let mut output = Vec::new();
        output.extend_from_slice(&[0x12, 0x00]); //              CodecID
        output.extend_from_slice(&[0x01, 0x00]); //              TypeID
        output.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); //  Length

        assert!(write_record(&mut wtr, record_data.as_ref()).is_ok());
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record_eos() {
        let mut buf = [0u8; 2];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = RecordOwned {
            codec_id: CODEC_ID_EOS,
            type_id: 0,
            val: None,
        };

        let output = vec![0xFF, 0xFF]; // CodecID

        assert!(write_record(&mut wtr, record_data.as_ref()).is_ok());
        assert_eq!(wtr.into_inner(), &output);
    }
}
