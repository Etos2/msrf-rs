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

#[inline]
fn insert_impl(buf: &mut &mut [u8], data: &[u8]) -> Result<(), usize> {
    let (dst, rem) = std::mem::take(buf)
        .split_at_mut_checked(data.len())
        .ok_or_else(|| data.len() - buf.len())?;
    dst.copy_from_slice(&data);
    *buf = rem;
    Ok(())
}

#[inline]
fn extract_impl<'a>(buf: &mut &'a [u8], len: usize) -> Result<&'a [u8], usize> {
    let (out, rem) = buf.split_at_checked(len).ok_or_else(|| len - buf.len())?;
    *buf = rem;
    Ok(out) // SAFETY: out has len of N
}

pub trait MutByteStream {
    fn insert(&mut self, data: &[u8]) -> Result<(), usize>;
    fn insert_varint(&mut self, data: u64) -> Result<(), usize>;
    fn insert_u8(&mut self, data: u8) -> Result<(), usize> {
        self.insert(&data.to_le_bytes())
    }
    fn insert_u16(&mut self, data: u16) -> Result<(), usize> {
        self.insert(&data.to_le_bytes())
    }
    fn insert_u32(&mut self, data: u32) -> Result<(), usize> {
        self.insert(&data.to_le_bytes())
    }
    fn insert_u64(&mut self, data: u64) -> Result<(), usize> {
        self.insert(&data.to_le_bytes())
    }
}

impl<'a> MutByteStream for &'a mut [u8] {
    fn insert(&mut self, data: &[u8]) -> Result<(), usize> {
        insert_impl(self, data)
    }

    fn insert_varint(&mut self, data: u64) -> Result<(), usize> {
        let mut buf = [0; 9];
        let len = varint::encode(&mut buf, data);
        insert_impl(self, &buf[..len])
    }
}

pub trait ByteStream {
    fn extract(&mut self, len: usize) -> Result<&[u8], usize>;
    fn extract_varint(&mut self) -> Result<u64, usize>;
    fn extract_u8(&mut self) -> Result<u8, usize> {
        Ok(u8::from_le_bytes(self.extract(1)?.try_into().unwrap()))
    }
    fn extract_u16(&mut self) -> Result<u16, usize> {
        Ok(u16::from_le_bytes(self.extract(2)?.try_into().unwrap()))
    }
    fn extract_u32(&mut self) -> Result<u32, usize> {
        Ok(u32::from_le_bytes(self.extract(4)?.try_into().unwrap()))
    }
    fn extract_u64(&mut self) -> Result<u64, usize> {
        Ok(u64::from_le_bytes(self.extract(8)?.try_into().unwrap()))
    }
    fn skip(&mut self, len: usize) -> Result<(), usize>;
}

impl<'a> ByteStream for &'a [u8] {
    fn extract(&mut self, len: usize) -> Result<&[u8], usize> {
        // SAFETY: slice has len of N
        Ok(extract_impl(self, len)?)
    }

    fn extract_varint(&mut self) -> Result<u64, usize> {
        let tag = self.get(0).ok_or(1usize)?;
        let data = extract_impl(self, varint::len(*tag))?;
        Ok(varint::decode(data))
    }

    fn skip(&mut self, len: usize) -> Result<(), usize> {
        *self = &self.get(len as usize..).ok_or_else(|| len - self.len())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytestream_array() {
        let mut buf = [0; 10];
        let val = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        buf.as_mut_slice().insert(&val).unwrap();
        assert_eq!(buf, val);
        let mut buf_mut = buf.as_slice();
        let expected = buf_mut.extract(val.len()).unwrap();
        assert_eq!(expected, val);
    }

    #[test]
    fn bytestream_u8() {
        let mut buf = [0; 1];
        let val = u8::MAX;

        buf.as_mut_slice().insert_u8(val).unwrap();
        assert_eq!(buf, val.to_le_bytes());
        let expected = buf.as_slice().extract_u8().unwrap();
        assert_eq!(expected, val);
    }

    #[test]
    fn bytestream_u16() {
        let mut buf = [0; 2];
        let val = u16::MAX;

        buf.as_mut_slice().insert_u16(val).unwrap();
        assert_eq!(buf, val.to_le_bytes());
        let expected = buf.as_slice().extract_u16().unwrap();
        assert_eq!(expected, val);
    }

    #[test]
    fn bytestream_varint() {
        let val = u64::MAX;
        let mut buf = [0; 9];
        let mut varint_buf = [0; 9];
        let _ = varint::encode(&mut varint_buf, val);

        buf.as_mut_slice().insert_varint(val).unwrap();
        assert_eq!(buf, varint_buf);
        let expected = buf.as_slice().extract_varint().unwrap();
        assert_eq!(expected, val);
    }

    #[test]
    fn bytestream_skip() {
        let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut buf = data.as_slice();
        buf.skip(4).unwrap();
        assert_eq!(buf, &data[4..]);
    }
}
