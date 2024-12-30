use crate::{
    error::{PError, PResult},
    CodecEntry, CodecTable, HeaderOwned, Record, RecordOwned, MAGIC_BYTES,
};
use std::{collections::HashMap, io::Read, ops::Range};

pub struct Reader {}

impl Reader {
    pub fn parse_header(data: &[u8]) -> PResult<HeaderOwned> {
        todo!()
    }

    pub fn parse_record(data: &[u8]) -> PResult<Record> {
        todo!()
    }
}

#[inline]
fn take<const N: usize>(mut rdr: impl Read) -> PResult<[u8; N]> {
    let mut buf = [0u8; N];
    rdr.read_exact(&mut buf)?;
    Ok(buf)
}

fn verify_range<T: PartialOrd + Into<u64>>(data: T, range: Range<T>) -> PResult<T> {
    if range.contains(&data) {
        Ok(data)
    } else {
        Err(PError::OutsideRange {
            found: data.into(),
            range: range.start.into()..range.end.into(),
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
    let data = take(&mut rdr)?;
    if data != expected {
        Err(PError::MismatchBytes {
            found: data.to_vec(),
            expected: expected.to_vec(),
        })
    } else {
        Ok(())
    }
}

#[inline]
fn read_null(mut rdr: impl Read) -> PResult<()> {
    let data = take::<1>(&mut rdr)?[0];
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
fn read_u8(mut rdr: impl Read) -> PResult<u8> {
    Ok(take::<1>(&mut rdr)?[0])
}

#[inline]
fn read_u16(mut rdr: impl Read) -> PResult<u16> {
    Ok(u16::from_le_bytes(take(&mut rdr)?))
}

#[inline]
fn read_u32(mut rdr: impl Read) -> PResult<u32> {
    Ok(u32::from_le_bytes(take(&mut rdr)?))
}

#[inline]
fn read_u64(mut rdr: impl Read) -> PResult<u64> {
    Ok(u64::from_le_bytes(take(&mut rdr)?))
}

#[inline]
fn read_uint(mut rdr: impl Read, bytes: usize) -> PResult<u64> {
    verify_range(bytes as u64, 0..8)?;
    let mut buf = [0u8; 8];
    rdr.read_exact(&mut buf[..bytes])?;
    Ok(u64::from_le_bytes(buf))
}

fn header(mut rdr: impl Read) -> PResult<HeaderOwned> {
    read_assert(&mut rdr, MAGIC_BYTES)?;
    let version = read_u16(&mut rdr)?;
    let flags = read_u16(&mut rdr)?.into();
    let codec_entries = read_u16(&mut rdr)?;

    let mut codec_table = HashMap::new();
    for _ in 0..codec_entries {
        let length = verify_range(read_u8(&mut rdr)?, 6..66)?; // TODO: Allow longer strings? Currently 4-64 chars.
        let codec_id = read_u8(&mut rdr)?;
        let layout = read_u8(&mut rdr)?;
        let type_id_len = verify_range(layout & 0x0F, 0..8)?; // TODO: Allow full 16 range? Currently 0-8 bytes.
        let type_length_len = verify_range(layout & 0xF0 >> 4, 0..8)?; // TODO: Allow full 16 range? Currently 0-8 bytes.
        let name = String::from_utf8(read_vec(&mut rdr, (length - 2) as usize)?)?;
        read_null(&mut rdr)?;

        codec_table.insert(
            codec_id,
            CodecEntry {
                type_id_len,
                type_length_len,
                name,
            },
        );
    }

    Ok(HeaderOwned {
        version,
        flags,
        codec_table,
    })
}

fn record(mut rdr: impl Read, codec_state: &CodecTable) -> PResult<RecordOwned> {
    let codec_id = read_u8(&mut rdr)?;
    let codec = codec_state
        .get(&codec_id)
        .ok_or(PError::NoCodec(codec_id))?;

    let type_id = read_uint(&mut rdr, codec.type_id_len as usize)?;
    let length = read_uint(&mut rdr, codec.type_length_len as usize)?;
    let val = if length != 0 {
        let val = read_vec(&mut rdr, length as usize)?.into();
        read_null(&mut rdr)?;
        val
    } else {
        Vec::new().into()
    };

    Ok(RecordOwned {
        codec_id,
        type_id,
        val,
    })
}

#[cfg(test)]
mod test {
    use crate::{CodecEntry, HeaderFlags, MAGIC_BYTES};

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
        input.extend_from_slice(&[0x12]); //            CodecID
        input.extend_from_slice(&[0x11]); //            Layout
        input.extend_from_slice(b"TEST"); //            Name
        input.extend_from_slice(&[0x00]); //            Guard (null byte)

        input.extend_from_slice(&[0x08]); //            Length
        input.extend_from_slice(&[0x05]); //            CodecID
        input.extend_from_slice(&[0x22]); //            Layout
        input.extend_from_slice(b"FOOBAR"); //          Name
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
                        0x12,
                        CodecEntry {
                            type_id_len: 1,
                            type_length_len: 1,
                            name: String::from("TEST")
                        }
                    ),
                    (
                        0x05,
                        CodecEntry {
                            type_id_len: 2,
                            type_length_len: 2,
                            name: String::from("FOOBAR")
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
        let codec_table = HashMap::from([(
            0x12,
            CodecEntry {
                type_id_len: 2,
                type_length_len: 2,
                name: "TEST".to_string(),
            },
        )]);

        let mut input = Vec::new();
        input.extend_from_slice(&[0x12]); //        CodecID
        input.extend_from_slice(&[0x01, 0x00]); //  TypeID
        input.extend_from_slice(&[0xFF, 0x00]); //  Length
        input.extend_from_slice(&vec![0; 255]); //  Value
        input.extend_from_slice(&[0x00]); //        Guard

        let mut rdr = Cursor::new(&input);
        let result = record(&mut rdr, &codec_table);

        assert!(result.is_ok(), "parse error: {:?}", result);
        assert_eq!(
            result.unwrap(),
            RecordOwned {
                codec_id: 18,
                type_id: 1,
                val: vec![0; 0xFF].into(),
            }
        );

        assert!(
            rdr.position() as usize == input.len(),
            "expected eof ({} bytes remaining)",
            input.len() - rdr.position() as usize
        );
    }
}
