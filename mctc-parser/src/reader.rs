use crate::{
    data::{CodecEntry, Header, Record, RecordMeta},
    error::{PError, PResult},
    io::ReadExt,
    Options, CODEC_ENTRY_LENGTH_BOUNDS, CODEC_ID_EOS, MAGIC_BYTES,
};
use std::io::Read;

pub struct Reader {
    _options: Options,
}

impl Reader {
    pub fn new() -> Self {
        Reader {
            _options: Options::default(),
        }
    }
}

impl From<Options> for Reader {
    fn from(value: Options) -> Self {
        Reader { _options: value }
    }
}

pub fn parse_header(mut rdr: impl Read) -> PResult<Header> {
    header(&mut rdr)
}

pub fn parse_record(mut rdr: impl Read) -> PResult<Record> {
    record(&mut rdr)
}

pub fn parse_record_prefix(mut rdr: impl Read) -> PResult<RecordMeta> {
    record_meta(&mut rdr)
}

fn header(mut rdr: impl Read) -> PResult<Header> {
    let magic_number = rdr.read_u32()?;
    if magic_number != u32::from_le_bytes(MAGIC_BYTES) {
        return Err(PError::MismatchBytes {
            found: magic_number.to_le_bytes().to_vec(),
            expected: MAGIC_BYTES.to_vec(),
        });
    }

    let version = rdr.read_u16()?;
    let flags = rdr.read_u16()?.into();
    let codec_entries = rdr.read_u16()?;
    let mut codec_table = Vec::with_capacity(codec_entries as usize);
    for _ in 0..codec_entries {
        // 2 byte version + 4-64 chars OR empty
        let length = rdr.read_u8()? as usize;
        if length != 0 {
            // TODO: Assert error on fields (e.g. String too long) rather than (entry too long)
            if !CODEC_ENTRY_LENGTH_BOUNDS.contains(&(length as u64)) {
                return Err(PError::new_range(length as u64, CODEC_ENTRY_LENGTH_BOUNDS));
            }

            let version = rdr.read_u16()?;
            let name = rdr.read_chunk(length - 2).map(String::from_utf8)??;

            // TODO: Allow longer strings?
            // TODO: Redundant check (previously asserted above)
            if !(4..=64).contains(&name.len()) {
                return Err(PError::new_range(name.len() as u64, 4..=64));
            }

            let guard = rdr.read_u8()?;
            if guard != 0 {
                return Err(PError::MismatchBytes {
                    found: vec![guard; 1],
                    expected: vec![0x00; 1],
                });
            }

            codec_table.push(Some(CodecEntry { version, name }));
        } else {
            codec_table.push(None);
        }
    }

    Ok(Header {
        version,
        flags,
        codec_table: codec_table.into(),
    })
}

fn record(mut rdr: impl Read) -> PResult<Record> {
    match rdr.read_pvarint()? {
        CODEC_ID_EOS => Ok(Record::new_eos()),
        codec_id => {
            let type_id = rdr.read_pvarint()?;
            let length = rdr.read_pvarint()?;

            let val = if length != 0 {
                let val = rdr.read_chunk(length as usize)?.into();
                let guard = rdr.read_u8()?;
                if guard != 0 {
                    return Err(PError::MismatchBytes {
                        found: vec![guard; 1],
                        expected: vec![0x00; 1],
                    });
                }

                Some(val)
            } else {
                None
            };

            Ok(Record {
                codec_id,
                type_id,
                val,
            })
        }
    }
}

fn record_meta(mut rdr: impl Read) -> PResult<RecordMeta> {
    match rdr.read_pvarint()? {
        CODEC_ID_EOS => Ok(RecordMeta::new_eos()),
        codec_id => {
            let type_id = rdr.read_pvarint()?;
            let length = rdr.read_pvarint()?;

            Ok(RecordMeta {
                codec_id,
                type_id,
                length: length as usize,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;
    use crate::{
        data::{CodecEntry, CodecTable, HeaderFlags},
        io::*,
        MAGIC_BYTES,
    };

    #[test]
    fn test_header() {
        let mut input = Vec::new();
        input.extend_from_slice(&MAGIC_BYTES); //       Magic Bytes "MCTC"
        input.extend_from_slice(&[0x00, 0x00]); //      Version
        input.extend_from_slice(&[0x00, 0x00]); //      Flags (Unused)
        input.extend_from_slice(&[0x03, 0x00]); //      Codec Entries

        input.extend_from_slice(&[0x06]); //            Length
        input.extend_from_slice(&[0x00, 0x00]); //      Version
        input.extend_from_slice(b"TEST"); //            Name
        input.extend_from_slice(&[0x00]); //            Guard (null byte)

        input.extend_from_slice(&[0x00]); //            Length (Empty Entry)

        input.extend_from_slice(&[0x42]); //            Length
        input.extend_from_slice(&[0x00, 0x01]); //      Version
        input.extend_from_slice(&vec![b'A'; 64]); //    Name
        input.extend_from_slice(&[0x00]); //            Guard (null byte)

        let mut rdr = Cursor::new(&input);
        let result = header(&mut rdr);

        assert!(result.is_ok(), "parse error: {:?}", result);
        assert_eq!(
            result.unwrap(),
            Header {
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
                    })
                ])
            }
        );

        assert!(
            rdr.position() as usize == input.len(),
            "expected eof ({} bytes remaining)",
            input.len() - rdr.position() as usize
        );
    }

    #[test]
    fn test_record() {
        let mut input = Vec::new();
        input.extend_from_slice(&[0x25]); //        CodecID
        input.extend_from_slice(&[0x03]); //        TypeID
        input.extend_from_slice(&[0xFE, 0x03]); //  Length
        input.extend_from_slice(&vec![0; 255]); //  Value
        input.extend_from_slice(&[0x00]); //        Guard

        let mut rdr = Cursor::new(&input);
        let result = record(&mut rdr);

        assert!(result.is_ok(), "parse error: {:?}", result);
        assert_eq!(
            result.unwrap(),
            Record {
                codec_id: 18,
                type_id: 1,
                val: Some(vec![0; 255].into()),
            }
        );

        assert!(
            rdr.position() as usize == input.len(),
            "expected eof ({} bytes remaining)",
            input.len() - rdr.position() as usize
        );
    }

    #[test]
    fn test_record_empty() {
        let mut input = Vec::new();
        input.extend_from_slice(&[0x25]); // CodecID
        input.extend_from_slice(&[0x03]); // TypeID
        input.extend_from_slice(&[0x01]); // Length

        let mut rdr = Cursor::new(&input);
        let result = record(&mut rdr);

        assert!(result.is_ok(), "parse error: {:?}", result);
        assert_eq!(
            result.unwrap(),
            Record {
                codec_id: 18,
                type_id: 1,
                val: None,
            }
        );

        assert!(
            rdr.position() as usize == input.len(),
            "expected eof ({} bytes remaining)",
            input.len() - rdr.position() as usize
        );
    }

    #[test]
    fn test_record_eos() {
        // CodecID
        let input = vec![0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let mut rdr = Cursor::new(&input);

        let result = record(&mut rdr);
        assert!(result.is_ok(), "parse error: {:?}", result);

        let record = result.unwrap();
        assert!(record.is_eos());
        assert_eq!(
            record,
            Record {
                codec_id: CODEC_ID_EOS,
                type_id: 0,
                val: None,
            }
        );

        assert!(
            rdr.position() as usize == input.len(),
            "expected eof ({} bytes remaining)",
            input.len() - rdr.position() as usize
        );
    }

    #[test]
    fn test_pv() {
        fn harness<T: AsRef<[u8]>>(data: T, expected: (u64, usize)) {
            let mut rdr = IoCounter::new(Cursor::new(data));
            let data = rdr.read_pvarint().unwrap();
            assert_eq!(data, expected.0);
            assert_eq!(rdr.count(), expected.1);
        }

        // Empty (1 byte)
        harness([0x01], (0, 1));
        // Partial (1 byte)
        harness([0x25], (18, 1));
        // Partial (2 byte)
        harness([(0xFF << 2) | 0x02, 0x03], (255, 2));
        // Partial (8 byte)
        harness(
            [0x80, 0xFA, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            (0xFFFFFFFFFFFFFA, 8),
        );
        // Full (9 byte)
        harness(
            [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            (u64::MAX, 9),
        );
    }
}
