use std::io::{Read, Result as IoResult, Take, Write, copy, sink};

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
    limit: u64,
    wtr: &'a mut W,
}

impl<'a, W: Write> RecordSink<'a, W> {
    pub(crate) fn new(wtr: &'a mut W, limit: u64) -> Self {
        Self { limit, wtr }
    }

    pub fn len(&self) -> u64 {
        self.limit
    }

    pub fn is_empty(&self) -> bool {
        self.limit == 0
    }

    pub(crate) fn drain(&mut self) -> IoResult<()> {
        if self.limit > 0 {
            copy(&mut std::io::repeat(0).take(self.limit), &mut self.wtr).map(|_| ())
        } else {
            Ok(())
        }
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

impl<'a, W: Write> Drop for RecordSink<'a, W> {
    fn drop(&mut self) {
        // BufWriter<W> drop impl also performs IO (flushing) on drop, we shall pretend this is normal
        let _res = self.drain();
        let _res = self.wtr.write_all(&[0u8]);
        let _res = self.wtr.flush();
    }
}

pub trait SizedRecord<S> {
    fn encoded_len(&self, ser: &S) -> usize;
}