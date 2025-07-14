use crate::error::{CodecError, CodecResult};

#[inline]
pub(crate) fn insert(buf: &mut &mut [u8], data: &[u8]) -> CodecResult<()> {
    let len = data.len();
    let (dst, rem) = std::mem::take(buf)
        .split_at_mut_checked(len)
        .ok_or_else(|| CodecError::Needed(len - data.len()))?;
    dst.copy_from_slice(data);
    *buf = rem;
    Ok(())
}

#[inline]
pub(crate) fn extract<'a>(buf: &mut &'a [u8], len: usize) -> CodecResult<&'a [u8]> {
    let (out, rem) = buf
        .split_at_checked(len)
        .ok_or_else(|| CodecError::Needed(len - buf.len()))?;
    *buf = rem;
    Ok(out)
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
