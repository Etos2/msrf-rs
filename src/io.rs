#![allow(clippy::len_without_is_empty)]
use std::io::{Error as IoError, Read, Result as IoResult, Take, Write, copy, sink};

const TAG_CONTAINS_DATA_LEN: usize = 7;

pub struct PVarint([u8; 9]);

impl PVarint {
    #[must_use] 
    pub fn new(data: [u8; 9]) -> Self {
        PVarint(data)
    }

    #[must_use] 
    pub fn encode(val: u64) -> Self {
        let zeros = val.leading_zeros();
        let mut buf = [0; 9];

        // Catch empty u64
        if zeros == 64 {
            buf[0] = 0x01;
        // Catch full u64
        } else if zeros == 0 {
            buf[1..].copy_from_slice(&val.to_le_bytes());
        // Catch var u64
        } else {
            let bytes = 8 - ((zeros - 1) / TAG_CONTAINS_DATA_LEN as u32) as usize;
            let data = val << (bytes + 1);
            buf[..=bytes].copy_from_slice(&data.to_le_bytes()[..=bytes]);
            buf[0] |= if bytes >= 8 { 0 } else { 0x01 << bytes };
        }

        PVarint(buf)
    }

    #[must_use] 
    pub fn decode(&self) -> u64 {
        let mut out = [0; 8];
        let len = self.len();

        if len <= TAG_CONTAINS_DATA_LEN {
            out[..len].copy_from_slice(&self.0[..len]);
            u64::from_le_bytes(out) >> len
        } else {
            out[..len - 1].copy_from_slice(&self.0[1..len]);
            u64::from_le_bytes(out)
        }
    }

    #[must_use] 
    pub fn as_slice(&self) -> &[u8] {
        let len = self.len();
        &self.0[..len]
    }

    #[must_use] 
    pub fn len(&self) -> usize {
        Self::len_from_tag(self.0[0])
    }

    #[must_use] 
    pub fn len_from_tag(tag: u8) -> usize {
        tag.trailing_zeros() as usize + 1
    }
}

impl From<PVarint> for u64 {
    fn from(pv: PVarint) -> Self {
        pv.decode()
    }
}

pub trait ReadExt {
    fn read_chunk<const N: usize>(&mut self) -> Result<[u8; N], IoError>;
    fn read_varint(&mut self) -> Result<u64, IoError>;
    fn read_u16(&mut self) -> Result<u16, IoError>;
}

impl<R: Read> ReadExt for R {
    fn read_chunk<const N: usize>(&mut self) -> Result<[u8; N], IoError> {
        let mut buf = [0; N];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    // TODO: Change varint api (struct wrapper)
    fn read_varint(&mut self) -> Result<u64, IoError> {
        let mut buf = [0; 9];
        self.read_exact(&mut buf[..1])?;
        let len = PVarint::len_from_tag(buf[0]);
        self.read_exact(&mut buf[1..len])?;
        let pv = PVarint::new(buf);
        Ok(pv.decode())
    }

    fn read_u16(&mut self) -> Result<u16, IoError> {
        Ok(u16::from_le_bytes(self.read_chunk()?))
    }
}

pub trait WriteExt {
    fn write_varint(&mut self, val: u64) -> Result<(), IoError>;
    fn write_u16(&mut self, val: u16) -> Result<(), IoError>;
}

impl<W: Write> WriteExt for W {
    fn write_varint(&mut self, val: u64) -> Result<(), IoError> {
        let varint = PVarint::encode(val);
        self.write_all(varint.as_slice())
    }

    fn write_u16(&mut self, val: u16) -> Result<(), IoError> {
        self.write_all(&val.to_le_bytes())
    }
}

pub struct RecordChunk<'a, R: Read>(Take<&'a mut R>);

impl<'a, R: Read> RecordChunk<'a, R> {
    pub(crate) fn new(rdr: &'a mut R, limit: u64) -> Self {
        Self(rdr.take(limit))
    }

    #[must_use] 
    pub fn len(&self) -> u64 {
        self.0.limit()
    }

    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.0.limit() == 0
    }

    pub(crate) fn drain(&mut self) -> IoResult<()> {
        if self.0.limit() > 0 {
            copy(&mut self.0, &mut sink()).map(|_| ())
        } else {
            Ok(())
        }
    }
}

impl<R: Read> Read for RecordChunk<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.0.read(buf)
    }
}

impl<R: Read> Drop for RecordChunk<'_, R> {
    fn drop(&mut self) {
        // BufWriter<W> drop impl also performs IO (flushing) on drop, we shall pretend this is normal
        let _res = self.drain();
    }
}

pub struct RecordSink<'a, W: Write> {
    wtr: &'a mut W,
    limit: u64,
}

impl<'a, W: Write> RecordSink<'a, W> {
    pub(crate) fn new(wtr: &'a mut W, limit: u64) -> Self {
        Self { wtr, limit }
    }

    #[must_use] 
    pub fn limit(&self) -> u64 {
        self.limit
    }

    #[must_use] 
    pub fn is_finished(&self) -> bool {
        self.limit == 0
    }

    fn finish_impl(&mut self) -> IoResult<u64> {
        let zeros = self.limit + 1;
        let blanked = copy(&mut std::io::repeat(0).take(zeros), &mut self.wtr)?;
        self.wtr.flush()?;
        Ok(blanked - 1)
    }

    pub fn finish(mut self) -> IoResult<u64> {
        self.finish_impl()
    }
}

impl<W: Write> Write for RecordSink<'_, W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let len = buf.len().min(self.limit as usize);
        self.limit -= len as u64;
        self.wtr.write(&buf[..len])
    }

    fn flush(&mut self) -> IoResult<()> {
        self.wtr.flush()
    }
}

// BufWriter<W> drop impl also performs IO (flushing) on drop, we shall pretend this is normal
impl<W: Write> Drop for RecordSink<'_, W> {
    fn drop(&mut self) {
        let _ = self.finish_impl();
    }
}

pub trait SizedRecord<S> {
    fn encoded_len(&self, ser: &S) -> usize;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialise_pvarint_2() {
        fn harness(val: u64) {
            let varint = PVarint::encode(val);
            let varint_value = varint.decode();
            assert_eq!(
                val, varint_value,
                "failed to manually encode/decode {val:X} != {varint_value:X}"
            );
        }

        harness(0x00); // 2^7-1
        harness(0x7F); // 2^7-1
        harness(0x3FFF); // 2^14-1
        harness(0x1FFFFF); // 2^21-1
        harness(0xFFFFFFF); // 2^28-1
        harness(0x7FFFFFFFF); // 2^35-1
        harness(0x3FFFFFFFFFF); // 2^42-1
        harness(0x1FFFFFFFFFFFF); // 2^49-1
        harness(0xFFFFFFFFFFFFFF); // 2^56-1
        harness(0xFFFFFFFFFFFFFFFF); // 2^64
    }
}
