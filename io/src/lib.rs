use std::error::Error;

pub mod error;
pub mod varint;

pub trait RecordSerialise
where
    Self: Sized,
{
    type Err: Error;
    type Record;

    fn deserialise_record(&self, id: u16, value: &[u8]) -> Result<Self::Record, Self::Err>;
    fn serialise_record(&self, value: &mut [u8], record: &Self::Record)
    -> Result<usize, Self::Err>;
}

pub trait MutByteStream {
    fn insert_checked(&mut self, data: &[u8]) -> Result<(), usize>;
    fn insert_slice(&mut self, data: &[u8]);

    #[inline]
    fn insert_array_checked<const N: usize>(&mut self, data: [u8; N]) -> Result<(), usize> {
        self.insert_checked(data.as_slice())
    }

    #[inline]
    fn insert<const N: usize>(&mut self, data: [u8; N]) {
        self.insert_slice(data.as_slice())
    }
}

impl<'a> MutByteStream for &'a mut [u8] {
    #[inline]
    fn insert_checked(&mut self, data: &[u8]) -> Result<(), usize> {
        let (dst, rem) = std::mem::take(self)
            .split_at_mut_checked(data.len())
            .ok_or_else(|| data.len() - self.len())?;
        dst.copy_from_slice(&data);
        *self = rem;
        Ok(())
    }

    #[inline]
    fn insert_slice(&mut self, data: &[u8]) {
        let (dst, rem) = std::mem::take(self).split_at_mut(data.len());
        dst.copy_from_slice(&data);
        *self = rem;
    }
}

pub trait ByteStream {
    fn extract_slice_checked(&mut self, len: usize) -> Result<&[u8], usize>;
    fn extract_slice(&mut self, len: usize) -> &[u8];

    #[inline]
    fn extract_checked<const N: usize>(&mut self) -> Result<[u8; N], usize> {
        // SAFETY: self.extract(N) returns &[u8; N]
        Ok(self.extract_slice_checked(N)?.try_into().unwrap())
    }

    #[inline]
    fn extract<const N: usize>(&mut self) -> [u8; N] {
        // SAFETY: self.extract(N) returns &[u8; N]
        self.extract_slice(N).try_into().unwrap()
    }
}

impl<'a> ByteStream for &'a [u8] {
    #[inline]
    fn extract_slice_checked(&mut self, len: usize) -> Result<&[u8], usize> {
        let (out, rem) = self.split_at_checked(len).ok_or_else(|| len - self.len())?;
        *self = rem;
        Ok(out)
    }

    #[inline]
    fn extract_slice(&mut self, len: usize) -> &[u8] {
        assert!(len <= self.len());
        let (out, rem) = self.split_at(len);
        *self = rem;
        out
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytestream() {
        let mut buf = [0; 10];
        let val = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        buf.as_mut_slice().insert_checked(&val).unwrap();
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.extract_slice_checked(val.len()).unwrap();
        assert_eq!(expected, val);

        buf.as_mut_slice().insert_slice(&val);
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.extract_slice(val.len());
        assert_eq!(expected, val);

        buf.as_mut_slice().insert_array_checked(val).unwrap();
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.extract_checked().unwrap();
        assert_eq!(expected, val);

        buf.as_mut_slice().insert(val);
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.extract();
        assert_eq!(expected, val);
    }

    // #[test]
    // fn bytestream_skip() {
    //     let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    //     let mut buf = data.as_slice();
    //     buf.skip(4).unwrap();
    //     assert_eq!(buf, &data[4..]);
    // }
}
