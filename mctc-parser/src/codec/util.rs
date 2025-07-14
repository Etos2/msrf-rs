use crate::error::{CodecError, CodecResult};

pub(crate) trait MutByteStream {
    fn insert<const N: usize>(&mut self, data: [u8; N]) -> CodecResult<()>;
    fn insert_slice(&mut self, data: &[u8]) -> CodecResult<()>;
    fn insert_u8(&mut self, data: u8) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_i8(&mut self, data: i8) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_u16(&mut self, data: u16) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_i16(&mut self, data: i16) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_u32(&mut self, data: u32) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_i32(&mut self, data: i32) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_u64(&mut self, data: u64) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
    fn insert_i64(&mut self, data: i64) -> CodecResult<()> {
        self.insert(data.to_le_bytes())
    }
}

impl<'a> MutByteStream for &'a mut [u8] {
    fn insert<const N: usize>(&mut self, data: [u8; N]) -> CodecResult<()> {
        let (dst, rem) = std::mem::take(self)
            .split_at_mut_checked(N)
            .ok_or_else(|| CodecError::Needed(N - self.len()))?;
        dst.copy_from_slice(&data);
        *self = rem;
        Ok(())
    }

    fn insert_slice(&mut self, data: &[u8]) -> CodecResult<()> {
        let (dst, rem) = std::mem::take(self)
            .split_at_mut_checked(data.len())
            .ok_or_else(|| CodecError::Needed(data.len() - self.len()))?;
        dst.copy_from_slice(data);
        *self = rem;
        Ok(())
    }
}

pub(crate) trait ByteStream {
    fn extract<const N: usize>(&mut self) -> CodecResult<[u8; N]>;
    fn extract_slice(&mut self, len: usize) -> CodecResult<&[u8]>;
    fn extract_u8(&mut self) -> CodecResult<u8> {
        Ok(u8::from_le_bytes(self.extract()?))
    }
    fn extract_i8(&mut self) -> CodecResult<i8> {
        Ok(i8::from_le_bytes(self.extract()?))
    }
    fn extract_u16(&mut self) -> CodecResult<u16> {
        Ok(u16::from_le_bytes(self.extract()?))
    }
    fn extract_i16(&mut self) -> CodecResult<i16> {
        Ok(i16::from_le_bytes(self.extract()?))
    }
    fn extract_u32(&mut self) -> CodecResult<u32> {
        Ok(u32::from_le_bytes(self.extract()?))
    }
    fn extract_i32(&mut self) -> CodecResult<i32> {
        Ok(i32::from_le_bytes(self.extract()?))
    }
    fn extract_u64(&mut self) -> CodecResult<u64> {
        Ok(u64::from_le_bytes(self.extract()?))
    }
    fn extract_i64(&mut self) -> CodecResult<i64> {
        Ok(i64::from_le_bytes(self.extract()?))
    }
    fn skip(&mut self, len: usize) -> CodecResult<()>;
}

impl<'a> ByteStream for &'a [u8] {
    fn extract<const N: usize>(&mut self) -> CodecResult<[u8; N]> {
        let (out, rem) = self
            .split_at_checked(N)
            .ok_or_else(|| CodecError::Needed(N - self.len()))?;
        *self = rem;
        Ok(out.try_into().unwrap()) // SAFETY: out has len of N
    }

    fn extract_slice(&mut self, len: usize) -> CodecResult<&[u8]> {
        let (out, rem) = self
            .split_at_checked(len)
            .ok_or_else(|| CodecError::Needed(len - self.len()))?;
        *self = rem;
        Ok(out)
    }

    fn skip(&mut self, len: usize) -> CodecResult<()> {
        *self = &self
            .get(len as usize..)
            .ok_or_else(|| CodecError::Needed(len - self.len()))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct PVarint([u8; 9]);

impl PVarint {
    pub fn new(val: u64) -> Self {
        PVarint(Self::encode(val))
    }

    pub fn get(&self) -> u64 {
        Self::decode_impl(self.as_slice())
    }

    pub fn as_slice(&self) -> &[u8] {
        let len = self.0[0].trailing_zeros() as usize;
        &self.0[..=len]
    }

    pub fn from_slice(raw: &[u8]) -> Option<Self> {
        if PVarint::len_from_tag(*raw.get(0)?) > raw.len() {
            return None;
        }
        let mut dst = [0; 9];
        dst[..raw.len()].copy_from_slice(raw);
        Some(PVarint(dst))
    }

    pub fn len(&self) -> usize {
        Self::len_from_tag(self.0[0])
    }

    pub fn len_from_tag(tag: u8) -> usize {
        tag.trailing_zeros() as usize + 1
    }

    pub fn encode(val: u64) -> [u8; 9] {
        let mut out = [0u8; 9];
        let zeros = val.leading_zeros();

        // Catch empty u64
        if zeros == 64 {
            out
        // Catch full u64
        } else if zeros == 0 {
            out[1..].copy_from_slice(&val.to_le_bytes());
            out
        // Catch var u64
        } else {
            let bytes = 8 - ((zeros - 1) / 7) as usize;
            let data = val << bytes + 1;
            out[..=bytes].copy_from_slice(&data.to_le_bytes()[..=bytes]);
            out[0] |= if bytes >= 8 { 0 } else { 0x01 << bytes };
            out
        }
    }

    pub fn decode(val: &[u8]) -> Option<u64> {
        let len = val.get(0)?.trailing_zeros() as usize;
        Some(Self::decode_impl(val.get(..=len)?))
    }

    fn decode_impl(val: &[u8]) -> u64 {
        let tag = val[0];
        let data = &val[1..];
        let len = val.len() - 1;
        let mut out = [0; 8];

        out[..len].copy_from_slice(&data);
        let data = u64::from_le_bytes(out);

        let out = if len < 7 {
            // Catch tag w/data (0bXXXXXXX1...0bX100000)
            let remainder = tag >> (len + 1); // Remove guard bit
            (data << (7 - len)) + remainder as u64
        } else {
            // Catch tag w/o data (0b1000000 + 0b00000000)
            data
        };

        out
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    #[test]
    fn pvarint_api() {
        fn harness(val: u64) {
            let enc = PVarint::encode(val);
            let dec = PVarint::decode(enc.as_slice()).expect("invalid input");
            assert_eq!(val, dec, "failed to manually encode/decode");

            let pv = PVarint::new(val);
            assert_eq!(val, pv.get(), "failed to implicitly encode/decode");
            assert_eq!(val, dec, "failed to compare manual/implicit");
        }

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
