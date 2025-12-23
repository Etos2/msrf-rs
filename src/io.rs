use std::io::{Error as IoError, Read, Result as IoResult, Take, Write, copy, sink};

use crate::codec::varint;

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
        let len = varint::len(buf[0]);
        self.read_exact(&mut buf[1..len])?;
        Ok(varint::from_le_bytes(&buf))
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
        let varint = varint::to_le_bytes(val);
        let len = varint::len(varint[0]);
        self.write_all(&varint[..len])
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

    pub fn len(&self) -> u64 {
        self.0.limit()
    }

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

impl<'a, R: Read> Read for RecordChunk<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.0.read(buf)
    }
}

impl<'a, R: Read> Drop for RecordChunk<'a, R> {
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

    pub fn len(&self) -> u64 {
        self.limit
    }

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

impl<'a, W: Write> Write for RecordSink<'a, W> {
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
impl<'a, W: Write> Drop for RecordSink<'a, W> {
    fn drop(&mut self) {
        let _ = self.finish_impl();
    }
}

pub trait SizedRecord<S> {
    fn encoded_len(&self, ser: &S) -> usize;
}
