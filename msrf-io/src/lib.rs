use crate::error::{CodecError, CodecResult};

pub mod error;
pub mod varint;

#[inline]
fn insert_impl(buf: &mut &mut [u8], data: &[u8]) -> CodecResult<()> {
    let (dst, rem) = std::mem::take(buf)
        .split_at_mut_checked(data.len())
        .ok_or_else(|| CodecError::Needed(data.len() - buf.len()))?;
    dst.copy_from_slice(&data);
    *buf = rem;
    Ok(())
}

#[inline]
fn extract_impl<'a>(buf: &mut &'a [u8], len: usize) -> CodecResult<&'a [u8]> {
    let (out, rem) = buf
        .split_at_checked(len)
        .ok_or_else(|| CodecError::Needed(len - buf.len()))?;
    *buf = rem;
    Ok(out) // SAFETY: out has len of N
}

pub trait MutByteStream {
    fn insert<const N: usize>(&mut self, data: [u8; N]) -> CodecResult<()>;
    fn insert_varint(&mut self, data: u64) -> CodecResult<()>;
    fn insert_u8(&mut self, data: u8) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_u16(&mut self, data: u16) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
}

impl<'a> MutByteStream for &'a mut [u8] {
    fn insert<const N: usize>(&mut self, data: [u8; N]) -> CodecResult<()> {
        insert_impl(self, data.as_slice())
    }

    fn insert_varint(&mut self, data: u64) -> CodecResult<()> {
        let mut buf = [0; 9];
        let len = varint::encode(&mut buf, data);
        insert_impl(self, &buf[..len])
    }
}

pub trait ByteStream {
    fn extract<const N: usize>(&mut self) -> CodecResult<[u8; N]>;
    fn extract_varint(&mut self) -> CodecResult<u64>;
    fn extract_u8(&mut self) -> CodecResult<u8> {
        Ok(u8::from_le_bytes(self.extract()?))
    }
    fn extract_u16(&mut self) -> CodecResult<u16> {
        Ok(u16::from_le_bytes(self.extract()?))
    }
    fn skip(&mut self, len: usize) -> CodecResult<()>;
}

impl<'a> ByteStream for &'a [u8] {
    fn extract<const N: usize>(&mut self) -> CodecResult<[u8; N]> {
        // SAFETY: slice has len of N
        Ok(extract_impl(self, N)?.try_into().unwrap())
    }

    fn extract_varint(&mut self) -> CodecResult<u64> {
        let tag = self.get(0).ok_or_else(|| CodecError::Needed(1))?;
        let data = extract_impl(self, varint::len(*tag))?;
        Ok(varint::decode(data))
    }

    fn skip(&mut self, len: usize) -> CodecResult<()> {
        *self = &self
            .get(len as usize..)
            .ok_or_else(|| CodecError::Needed(len - self.len()))?;
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

        buf.as_mut_slice().insert(val).unwrap();
        assert_eq!(buf, val);
        let expected = buf.as_slice().extract().unwrap();
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
        let mut buf = [0; 9];
        let val = u64::MAX;

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
