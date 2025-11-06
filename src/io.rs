use std::io::{Read, Result as IoResult, Take, copy, sink};

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

    pub fn drain(&mut self) -> IoResult<()> {
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
