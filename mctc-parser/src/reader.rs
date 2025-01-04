use crate::{
    data::{CodecEntry, Header, HeaderOwned, RecordOwned},
    error::{PError, PResult},
    DefaultOptions, CODEC_ID_EOS, MAGIC_BYTES,
};
use std::{collections::HashMap, io::Read, ops::RangeBounds};

pub struct Reader {
    _options: DefaultOptions,
}

impl Reader {
    pub fn new() -> Self {
        Reader {
            _options: DefaultOptions::default(),
        }
    }

    pub fn records<R: Read>(mut rdr: R) -> PResult<RecordsIter<R>> {
        let header = parse_header(&mut rdr)?;
        Ok(RecordsIter {
            rdr,
            header,
            error: None,
            finished: false,
        })
    }
}

impl From<DefaultOptions> for Reader {
    fn from(value: DefaultOptions) -> Self {
        Reader { _options: value }
    }
}

pub struct RecordsIter<R: Read> {
    rdr: R,
    header: HeaderOwned,
    error: Option<PError>,
    finished: bool,
}

impl<R: Read> RecordsIter<R> {
    pub fn into_error(self) -> Option<PError> {
        self.error
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn header(&self) -> Header {
        self.header.as_ref()
    }
}

impl<R: Read> Iterator for RecordsIter<R> {
    type Item = RecordOwned;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.error.is_some() {
            return None;
        }

        match parse_record(&mut self.rdr) {
            Ok(record) => {
                if !record.is_eos() {
                    Some(record)
                } else {
                    self.finished = true;
                    None
                }
            }
            Err(e) => {
                self.error = Some(e);
                None
            }
        }
    }
}

pub fn parse_header(mut rdr: impl Read) -> PResult<HeaderOwned> {
    header(&mut rdr)
}

pub fn parse_record(mut rdr: impl Read) -> PResult<RecordOwned> {
    record(&mut rdr)
}

trait ReadExt {
    fn read_null(&mut self) -> PResult<()>;
    fn read_u8(&mut self) -> PResult<u8>;
    fn read_u16(&mut self) -> PResult<u16>;
    fn read_pv(&mut self) -> PResult<u64>;
    fn read_vec(&mut self, bytes: usize) -> PResult<Vec<u8>>;
    fn read_equals(&mut self, comp: &[u8]) -> PResult<()>;
}

#[inline]
fn read_array<const N: usize>(mut rdr: impl Read) -> PResult<[u8; N]> {
    let mut buf = [0u8; N];
    rdr.read_exact(&mut buf)?;
    Ok(buf)
}

impl<R: Read> ReadExt for R {
    #[inline]
    fn read_null(&mut self) -> PResult<()> {
        let data = read_array::<1>(self)?[0];
        if data == 0x00 {
            Ok(())
        } else {
            Err(PError::MismatchBytes {
                found: vec![data; 1],
                expected: vec![0x00; 1],
            })
        }
    }

    #[inline]
    fn read_u8(&mut self) -> PResult<u8> {
        Ok(read_array::<1>(self)?[0])
    }

    #[inline]
    fn read_u16(&mut self) -> PResult<u16> {
        Ok(u16::from_le_bytes(read_array(self)?))
    }

    #[inline]
    fn read_pv(&mut self) -> PResult<u64> {
        let tag = self.read_u8()?;
        let len = tag.trailing_zeros() as usize;
        let mut data = [0; 8];

        Ok(
            // Catch single byte varint
            if len == 0 {
                (tag >> 1) as u64
            // Catch tag w/data (0bXXXXXX10...0bX100000)
            } else if len < 7 {
                let remainder = tag >> (len + 1); // Remove bit then shift
                self.read_exact(&mut data[..len])?;
                (u64::from_le_bytes(data) << (7 - len)) + remainder as u64
            // Catch tag w/o data (0b1000000 + 0b00000000)
            } else {
                self.read_exact(&mut data[..len])?;
                u64::from_le_bytes(data)
            },
        )
    }

    #[inline]
    fn read_vec(&mut self, bytes: usize) -> PResult<Vec<u8>> {
        let mut buf = vec![0; bytes];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    fn read_equals(&mut self, expect: &[u8]) -> PResult<()> {
        let mut buf = vec![0; expect.len()];
        self.read_exact(&mut buf)?;
        if buf != *expect {
            Err(PError::MismatchBytes {
                found: buf.to_vec(),
                expected: expect.to_vec(),
            })
        } else {
            Ok(())
        }
    }
}

fn header(mut rdr: impl Read) -> PResult<HeaderOwned> {
    rdr.read_equals(&MAGIC_BYTES)?;
    let version = rdr.read_u16()?;
    let flags = rdr.read_u16()?.into();
    let codec_entries = rdr.read_u16()?;

    let mut codec_table = HashMap::with_capacity(codec_entries as usize);
    for _ in 0..codec_entries {
        let length = verify_range(rdr.read_u8()?, 6..=66)?; // TODO: Allow longer strings? Currently 4-64 chars.
        let codec_id = rdr.read_u16()?;
        let name = rdr
            .read_vec((length - 2) as usize)
            .map(String::from_utf8)??;
        rdr.read_null()?;

        if codec_table.contains_key(&codec_id) {
            return Err(PError::DuplicateCodec(codec_id));
        }
        codec_table.insert(codec_id, CodecEntry { name });
    }

    Ok(HeaderOwned {
        version,
        flags,
        codec_table,
    })
}

fn record(mut rdr: impl Read) -> PResult<RecordOwned> {
    match rdr.read_pv()? {
        CODEC_ID_EOS => Ok(RecordOwned::from_eos()),
        codec_id => {
            let type_id = rdr.read_pv()?;
            let length = rdr.read_pv()?;

            let val = if length != 0 {
                let val = rdr.read_vec(length as usize)?.into();
                rdr.read_null()?;
                Some(val)
            } else {
                None
            };

            Ok(RecordOwned {
                codec_id,
                type_id,
                val,
            })
        }
    }
}

#[inline]
fn verify_range<T>(data: T, range: impl RangeBounds<T>) -> PResult<T>
where
    T: PartialOrd + Into<u64> + Copy,
{
    if range.contains(&data) {
        Ok(data)
    } else {
        Err(PError::new_range(data, range))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        data::{CodecEntry, HeaderFlags},
        MAGIC_BYTES,
    };

    use std::{collections::HashMap, io::Cursor};

    use super::*;

    #[test]
    fn test_header() {
        let mut input = Vec::new();
        input.extend_from_slice(&MAGIC_BYTES); //       Magic Bytes "MCTC"
        input.extend_from_slice(&[0x00, 0x00]); //      Version
        input.extend_from_slice(&[0x00, 0x00]); //      Flags (Unused)
        input.extend_from_slice(&[0x02, 0x00]); //      Codec Entries

        input.extend_from_slice(&[0x06]); //            Length
        input.extend_from_slice(&[0x12, 0x00]); //      CodecID
        input.extend_from_slice(b"TEST"); //            Name
        input.extend_from_slice(&[0x00]); //            Guard (null byte)

        input.extend_from_slice(&[0x42]); //            Length
        input.extend_from_slice(&[0x05, 0x01]); //      CodecID
        input.extend_from_slice(&vec![b'A'; 64]); //    Name
        input.extend_from_slice(&[0x00]); //            Guard (null byte)

        let mut rdr = Cursor::new(&input);
        let result = header(&mut rdr);

        assert!(result.is_ok(), "parse error: {:?}", result);
        assert_eq!(
            result.unwrap(),
            HeaderOwned {
                version: 0,
                flags: HeaderFlags::empty(),
                codec_table: HashMap::from([
                    (
                        18,
                        CodecEntry {
                            name: String::from("TEST")
                        }
                    ),
                    (
                        261,
                        CodecEntry {
                            name: String::from_utf8(vec![b'A'; 64]).unwrap()
                        }
                    ),
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
            RecordOwned {
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
            RecordOwned {
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
            RecordOwned {
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
        // Full (9 byte)
        let mut rdr = Cursor::new([0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let data = rdr.read_pv().unwrap();
        assert_eq!(data, u64::MAX);
        // Empty (1 byte)
        let mut rdr = Cursor::new([0x01]);
        let data = rdr.read_pv().unwrap();
        assert_eq!(data, 0x00);
        // Partial (1 byte)
        let mut rdr = Cursor::new([0x25]);
        let data = rdr.read_pv().unwrap();
        assert_eq!(data, 18);
        // Partial (2 byte)
        let mut rdr = Cursor::new([(0xFF << 2) | 0x02, 0x03]);
        let data = rdr.read_pv().unwrap();
        assert_eq!(data, 0xFF);
        // Partial (8 byte)
        let mut rdr = Cursor::new([0x80, 0xFA, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let data = rdr.read_pv().unwrap();
        assert_eq!(data, 0xFFFFFFFFFFFFFA);
    }
}
