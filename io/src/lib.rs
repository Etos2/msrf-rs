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

pub trait TakeExt {
    fn take_slice(&mut self, len: usize) -> Option<&[u8]>;
    fn take_chunk<const N: usize>(&mut self) -> Option<[u8; N]>;
}

impl<'a> TakeExt for &'a [u8] {
    #[inline]
    fn take_slice(&mut self, len: usize) -> Option<&[u8]> {
        let (out, rem) = self.split_at_checked(len)?;
        *self = rem;
        Some(out)
    }

    #[inline]
    fn take_chunk<const N: usize>(&mut self) -> Option<[u8; N]> {
        // SAFETY: self.extract(N) returns &[u8].len() == N
        Some(self.take_slice(N)?.try_into().unwrap())
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
        let expected = buf_mut.take_slice(val.len()).unwrap();
        assert_eq!(expected, val);

        buf.as_mut_slice().insert_slice(&val);
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.take_slice(val.len()).unwrap();
        assert_eq!(expected, val);

        buf.as_mut_slice().insert_array_checked(val).unwrap();
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.take_chunk().unwrap();
        assert_eq!(expected, val);

        buf.as_mut_slice().insert(val);
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.take_chunk().unwrap();
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
