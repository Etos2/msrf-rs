use std::io::Write;

use crate::{
    data::{CodecTable, Header, HeaderFlags, Record},
    error::{PError, PResult},
    util::WriteExt,
    Codec, Options, CODEC_ID_EOS, CODEC_NAME_BOUNDS, CURRENT_VERSION, MAGIC_BYTES,
};

// TODO: Handling of multiple writers (a codec registered in one can then be passed to another)
#[derive(Debug)]
pub struct WriterHandle<'a, C: Codec> {
    codec: &'a mut C,
    id: u64,
}

impl<'a, C: Codec> WriterHandle<'a, C> {
    fn new(codec: &'a mut C, id: u64) -> Self {
        WriterHandle { codec, id }
    }

    fn codec_id(&self) -> u64 {
        self.id
    }

    fn as_parts(&'a mut self) -> (u64, &'a mut C) {
        (self.id, self.codec)
    }

    pub fn write(
        &'a mut self, wtr: &'a mut impl Write,
        value: C::Rec,
    ) -> PResult<()> {
        write_custom(wtr, self, value)
    }
}

pub struct WriterBuilder {
    opt: Options,
    codecs: CodecTable,
}

impl WriterBuilder {
    pub(crate) fn new(opt: Options) -> Self {
        WriterBuilder {
            opt,
            codecs: CodecTable::new(),
        }
    }

    pub fn register<'a, C: Codec>(&mut self, codec: &'a mut C) -> Option<WriterHandle<'a, C>> {
        let id = self.codecs.register::<C>()?;
        Some(WriterHandle::new(codec, id))
    }

    pub fn begin<W: Write>(self, wtr: &mut W) -> PResult<Writer> {
        let header = Header {
            version: CURRENT_VERSION,
            flags: HeaderFlags::empty(),
            codec_table: self.codecs,
        };

        write_header(wtr, &header)?;
        Ok(Writer::new())
    }
}

impl Default for WriterBuilder {
    fn default() -> Self {
        WriterBuilder {
            opt: Options::default(),
            codecs: CodecTable::new(),
        }
    }
}

// TODO: Customisability?
pub struct Writer {}

impl Writer {
    fn new() -> Self {
        Writer {  }
    }

    pub fn finish(self, wtr: &mut impl Write) -> PResult<()> {
        write_record(&mut *wtr, &Record::new_eos())?;
        wtr.flush()?;
        Ok(())
    }
}

pub fn write_header(mut wtr: impl Write, header: &Header) -> PResult<()> {
    wtr.write_all(&MAGIC_BYTES)?;
    wtr.write_u16(header.version)?;
    wtr.write_u16(header.flags.bits())?;
    wtr.write_u16(header.codec_table.len() as u16)?;

    for opt_codec in header.codec_table.as_ref().iter() {
        if let Some(codec) = opt_codec {
            if CODEC_NAME_BOUNDS.contains(&(codec.name.len() as u64)) {
                let len = codec.name.len() + 8;
                wtr.write_u8(len as u8)?;
                wtr.write_u16(codec.version)?;
                wtr.write_all(codec.name.as_bytes())?;
                wtr.write_null()?;
            } else {
                return Err(PError::new_range(
                    codec.name.len() as u64,
                    CODEC_NAME_BOUNDS,
                ));
            }
        } else {
            // Length 0
            wtr.write_null()?;
        }
    }

    Ok(())
}

pub fn write_record(mut wtr: impl Write, record: &Record) -> PResult<()> {
    wtr.write_pv(record.codec_id)?;
    if record.codec_id != CODEC_ID_EOS {
        wtr.write_pv(record.type_id)?;
        if let Some(val) = &record.val {
            wtr.write_pv(val.len() as u64)?;
            wtr.write_all(&val)?;
            wtr.write_null()?;
        } else {
            wtr.write_pv(0)?;
        }
    }

    Ok(())
}

// TODO: Test!
fn write_custom<'a, C: Codec>(
    wtr: &'a mut impl Write,
    handle: &'a mut WriterHandle<'a, C>,
    value: C::Rec,
) -> PResult<()> {
    let (codec_id, codec) = handle.as_parts();
    wtr.write_pv(codec_id)?;
    if codec_id != CODEC_ID_EOS {
        wtr.write_pv(codec.type_id(&value))?;
        let len = codec.size(&value);
        if len > 0 {
            wtr.write_pv(len as u64)?;
            codec.write_value(&mut *wtr, &value).unwrap(); // TODO: Need to return User Error + Parser Error (Fun!)
            wtr.write_null()?;
        } else {
            wtr.write_pv(0)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{io::Cursor, u64};

    use crate::data::{CodecEntry, Header, HeaderFlags, Record};

    use super::*;

    #[test]
    fn test_header() {
        let mut buf = [0u8; 102];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let header_data = Header {
            version: 0,
            flags: HeaderFlags::empty(),
            codec_table: CodecTable::from(vec![
                Some(CodecEntry {
                    version: 0,
                    name: String::from("TEST"),
                }),
                None,
                Some(CodecEntry {
                    version: 256,
                    name: String::from_utf8(vec![b'A'; 64]).unwrap(),
                }),
            ]),
        };

        let mut output = Vec::new();
        output.extend_from_slice(&MAGIC_BYTES); //       Magic Bytes "MCTC"
        output.extend_from_slice(&[0x00, 0x00]); //      Version
        output.extend_from_slice(&[0x00, 0x00]); //      Flags (Unused)
        output.extend_from_slice(&[0x02, 0x00]); //      Codec Entries

        output.extend_from_slice(&[0x0C]); //            Length
        output.extend_from_slice(&[0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // CodecID
        output.extend_from_slice(&[0x00, 0x00]); //      Version
        output.extend_from_slice(b"TEST"); //            Name
        output.extend_from_slice(&[0x00]); //            Guard (null byte)

        output.extend_from_slice(&[0x00]); //            Length (empty)

        output.extend_from_slice(&[0x48]); //            Length
        output.extend_from_slice(&[0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // CodecID
        output.extend_from_slice(&[0x00, 0x01]); //      Version
        output.extend_from_slice(&vec![b'A'; 64]); //    Name
        output.extend_from_slice(&[0x00]); //            Guard (null byte)

        let result = write_header(&mut wtr, &header_data);
        assert!(result.is_ok(), "write error: {:?}", result);
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record() {
        let mut buf = [0u8; 260];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = Record {
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

        write_record(&mut wtr, &record_data).unwrap();
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record_empty() {
        let mut buf = [0u8; 3];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = Record {
            codec_id: 18,
            type_id: 1,
            val: None,
        };

        let mut output = Vec::new();
        output.extend_from_slice(&[0x25]); // CodecID
        output.extend_from_slice(&[0x03]); // TypeID
        output.extend_from_slice(&[0x01]); // Length

        write_record(&mut wtr, &record_data).unwrap();
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_record_eos() {
        let mut buf = [0u8; 9];
        let mut wtr = Cursor::new(buf.as_mut_slice());
        let record_data = Record {
            codec_id: CODEC_ID_EOS,
            type_id: 0,
            val: None,
        };

        // CodecID
        let output = vec![0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

        write_record(&mut wtr, &record_data).unwrap();
        assert_eq!(wtr.into_inner(), &output);
    }

    #[test]
    fn test_pv() {
        // Empty (1 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0x00;
        let output = [0x01];
        wtr.write_pv(input).unwrap();
        let result = wtr.into_inner();
        assert_eq!(result, output);

        // Full (9 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = u64::MAX;
        let output = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        wtr.write_pv(input).unwrap();
        let result = wtr.into_inner();
        assert_eq!(result, output);

        // Partial (1 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0x21;
        let output = [(0x21 << 1) | 0x01];
        wtr.write_pv(input).unwrap();
        let result = wtr.into_inner();
        assert_eq!(result, output);

        // Partial (2 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0xFF;
        let output = [(0xFF << 2) | 0x02, 0x03];
        wtr.write_pv(input).unwrap();
        let result = wtr.into_inner();
        assert_eq!(result, output);

        // Partial (8 byte)
        let mut wtr = Cursor::new(Vec::new());
        let input = 0xFFFFFFFFFFFFFA;
        let output = [0x80, 0xFA, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        wtr.write_pv(input).unwrap();
        let result = wtr.into_inner();
        assert_eq!(result, output);
    }
}
