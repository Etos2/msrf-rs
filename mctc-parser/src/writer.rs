use std::io::Write;

use crate::{
    data::{Header, Record},
    error::{PError, PResult},
    CODEC_ID_EOS, CODEC_NAME_BOUNDS, MAGIC_BYTES,
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
    fn write_pv(&mut self, data: u64) -> PResult<()>;
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

    #[inline]
    fn write_pv(&mut self, data: u64) -> PResult<()> {
        let mut buf = [0u8; 9];
        let zeros = data.leading_zeros();

        // Catch empty u64
        if zeros == 64 {
            self.write_all(&[0x01])?;
        // Catch full u64
        } else if zeros == 0 {
            buf[1..].copy_from_slice(&data.to_le_bytes());
            self.write_all(&buf)?;
        // Catch var u64
        } else {
            let offset = 8 - ((zeros - 1) / 7) as usize;
            let data = data << offset + 1;
            buf[..=offset].copy_from_slice(&data.to_le_bytes()[..=offset]);
            buf[0] |= if offset >= u8::BITS as usize {
                0
            } else {
                0x01 << offset
            };
            self.write_all(&buf[..=offset])?;
        }

        Ok(())
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
        if CODEC_NAME_BOUNDS.contains(&(v.name.len() as u64)) {
            let len = 2 + v.name.len();
            wtr.write_u8(len as u8)?;
            wtr.write_u16(*k)?;
            wtr.write_all(v.name.as_bytes())?;
            wtr.write_null()?;
        } else {
            return Err(PError::new_range(v.name.len() as u64, CODEC_NAME_BOUNDS));
        }
    }

    Ok(())
}

fn write_record(mut wtr: impl Write, record: Record) -> PResult<()> {
    wtr.write_pv(record.codec_id)?;
    if record.codec_id != CODEC_ID_EOS {
        wtr.write_pv(record.type_id)?;
        if let Some(val) = record.val {
            wtr.write_pv(val.len() as u64)?;
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
    use std::{collections::HashMap, io::Cursor, u64};

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
        let mut buf = [0u8; 260];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = RecordOwned {
            codec_id: 18,
            type_id: 1,
            val: Some(vec![0; 0xFF].into()),
        };

        let mut output = Vec::new();
        output.extend_from_slice(&[0x25]); //        CodecID
        output.extend_from_slice(&[0x03]); //        TypeID
        output.extend_from_slice(&[0xFE, 0x03]); //  Length
        output.extend_from_slice(&vec![0; 255]); //  Value
        output.extend_from_slice(&[0x00]); //        Guard

        write_record(&mut wtr, record_data.as_ref()).unwrap();
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record_empty() {
        let mut buf = [0u8; 3];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = RecordOwned {
            codec_id: 18,
            type_id: 1,
            val: None,
        };

        let mut output = Vec::new();
        output.extend_from_slice(&[0x25]); // CodecID
        output.extend_from_slice(&[0x03]); // TypeID
        output.extend_from_slice(&[0x01]); // Length

        write_record(&mut wtr, record_data.as_ref()).unwrap();
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record_eos() {
        let mut buf = [0u8; 9];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = RecordOwned {
            codec_id: CODEC_ID_EOS,
            type_id: 0,
            val: None,
        };

        // CodecID
        let output = vec![0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

        write_record(&mut wtr, record_data.as_ref()).unwrap();
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_pv() {
        // Empty (1 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0x00;
        wtr.write_pv(input).unwrap();
        assert_eq!(wtr.into_inner(), [0x01]);

        // Full (9 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = u64::MAX;
        wtr.write_pv(input).unwrap();
        assert_eq!(
            wtr.into_inner(),
            [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        );

        // Partial (1 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0x21;
        wtr.write_pv(input).unwrap();
        assert_eq!(wtr.into_inner(), [(0x21 << 1) | 0x01,]);

        // Partial (2 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0xFF;
        wtr.write_pv(input).unwrap();
        assert_eq!(wtr.into_inner(), [(0xFF << 2) | 0x02, 0x03,]);

        // Partial (8 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0xFFFFFFFFFFFFFA;
        wtr.write_pv(input).unwrap();
        assert_eq!(
            wtr.into_inner(),
            [0x80, 0xFA, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        );
    }
}
