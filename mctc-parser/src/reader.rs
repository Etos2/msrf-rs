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

fn header(mut rdr: impl Read) -> PResult<HeaderOwned> {
    read_assert(&mut rdr, MAGIC_BYTES)?;
    let version = read_u16(&mut rdr)?;
    let flags = read_u16(&mut rdr)?.into();
    let codec_entries = read_u16(&mut rdr)?;

    let mut codec_table = HashMap::new();
    for _ in 0..codec_entries {
        let length = verify_range(read_u8(&mut rdr)?, 6..=66)?; // TODO: Allow longer strings? Currently 4-64 chars.
        let codec_id = read_u16(&mut rdr)?;
        let name = String::from_utf8(read_vec(&mut rdr, (length - 2) as usize)?)?;
        read_null(&mut rdr)?;

        codec_table.insert(codec_id, CodecEntry { name });
    }

    Ok(HeaderOwned {
        version,
        flags,
        codec_table,
    })
}

fn record(mut rdr: impl Read) -> PResult<RecordOwned> {
    match read_u16(&mut rdr)? {
        CODEC_ID_EOS => Ok(RecordOwned::from_eos()),
        codec_id => {
            let type_id = read_u16(&mut rdr)?;
            let length = read_u32(&mut rdr)?;

            let val = if length != 0 {
                let val = read_vec(&mut rdr, length as usize)?.into();
                read_null(&mut rdr)?;
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
fn read<const N: usize>(mut rdr: impl Read) -> PResult<[u8; N]> {
    let mut buf = [0u8; N];
    rdr.read_exact(&mut buf)?;
    Ok(buf)
}

#[inline]
fn read_u8(mut rdr: impl Read) -> PResult<u8> {
    Ok(read::<1>(&mut rdr)?[0])
}

#[inline]
fn read_u16(mut rdr: impl Read) -> PResult<u16> {
    Ok(u16::from_le_bytes(read(&mut rdr)?))
}

#[inline]
fn read_u32(mut rdr: impl Read) -> PResult<u32> {
    Ok(u32::from_le_bytes(read(&mut rdr)?))
}

#[inline]
fn read_null(mut rdr: impl Read) -> PResult<()> {
    let data = read::<1>(&mut rdr)?[0];
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
fn read_vec(mut rdr: impl Read, bytes: usize) -> PResult<Vec<u8>> {
    let mut buf = vec![0; bytes];
    rdr.read_exact(&mut buf)?;
    Ok(buf)
}

#[inline]
fn read_assert<const N: usize>(mut rdr: impl Read, expected: [u8; N]) -> PResult<()> {
    let data = read(&mut rdr)?;
    if data != expected {
        Err(PError::MismatchBytes {
            found: data.to_vec(),
            expected: expected.to_vec(),
        })
    } else {
        Ok(())
    }
}

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
        input.extend_from_slice(&[0x12, 0x00]); //              CodecID
        input.extend_from_slice(&[0x01, 0x00]); //              TypeID
        input.extend_from_slice(&[0xFF, 0x00, 0x00, 0x00]); //  Length
        input.extend_from_slice(&vec![0; 255]); //              Value
        input.extend_from_slice(&[0x00]); //                    Guard

        let mut rdr = Cursor::new(&input);
        let result = record(&mut rdr);

        assert!(result.is_ok(), "parse error: {:?}", result);
        assert_eq!(
            result.unwrap(),
            RecordOwned {
                codec_id: 18,
                type_id: 1,
                val: Some(vec![0; 0xFF].into()),
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
        input.extend_from_slice(&[0x12, 0x00]); //              CodecID
        input.extend_from_slice(&[0x01, 0x00]); //              TypeID
        input.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); //  Length

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
        let input = vec![0xFF, 0xFF]; // CodecID
        let mut rdr = Cursor::new(&input);

        let result = record(&mut rdr);
        assert!(result.is_ok(), "parse error: {:?}", result);

        let record = result.unwrap();
        assert!(record.is_eos());
        assert_eq!(
            record,
            RecordOwned {
                codec_id: 65535,
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
}
