use crate::error::{CodecError, CodecResult};

pub(crate) mod varint {
    const TAG_WITH_DATA_LEN: usize = 7;

    pub fn len(tag: u8) -> usize {
        tag.trailing_zeros() as usize + 1
    }

    pub fn encode(buf: &mut [u8; 9], val: u64) -> usize {
        let zeros = val.leading_zeros();

        // Catch empty u64
        if zeros == 64 {
            buf[0] = 0x01;
            1
        // Catch full u64
        } else if zeros == 0 {
            buf[1..].copy_from_slice(&val.to_le_bytes());
            9
        // Catch var u64
        } else {
            let bytes = 8 - ((zeros - 1) / TAG_WITH_DATA_LEN as u32) as usize;
            let data = val << bytes + 1;
            buf[..=bytes].copy_from_slice(&data.to_le_bytes()[..=bytes]);
            buf[0] |= if bytes >= 8 { 0 } else { 0x01 << bytes };
            bytes + 1
        }
    }

    // PANIC: Will panic when data.is_empty()
    pub fn decode(data: &[u8]) -> u64 {
        let mut out = [0; 8];
        let len = len(data[0]);

        if len <= TAG_WITH_DATA_LEN {
            out[..len].copy_from_slice(&data[..len]);
            u64::from_le_bytes(out) >> len
        } else {
            out[..len - 1].copy_from_slice(&data[1..len]);
            u64::from_le_bytes(out)
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use super::*;

        #[test]
        fn serialise_pvarint() {
            fn harness(val: u64) {
                let mut buf = [0; 9];
                let len = encode(&mut buf, val);
                let dec = decode(&buf[..len]);
                assert_eq!(
                    val, dec,
                    "failed to manually encode/decode {val:X} != {dec:X}"
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
}

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

pub(crate) trait MutByteStream {
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

pub(crate) trait ByteStream {
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