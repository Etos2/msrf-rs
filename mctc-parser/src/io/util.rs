use crate::{
    error::{CodecResult},
    io::{DecodeExt, DecodeFrom, EncodeInto},
};

pub struct Guard(u8);

impl Guard {
    pub fn new(bytes: &[u8]) -> Guard {
        Guard(Self::generate(bytes))
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn generate(bytes: &[u8]) -> u8 {
        !(bytes.iter().fold(0u8, |b, acc| acc ^ b))
    }
}

macro_rules! guard_impl {
    ($t:ident) => {
        impl From<$t> for Guard {
            fn from(value: $t) -> Self {
                Guard::new(&value.to_le_bytes())
            }
        }
    };
}

guard_impl!(u64);
guard_impl!(u32);
guard_impl!(u16);
guard_impl!(u8);

impl EncodeInto for Guard {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]> {
        self.0.encode_into(dst)
    }
}

impl<'a> DecodeFrom<'a> for Guard {
    fn decode_from(input: &'a [u8]) -> CodecResult<(&'a [u8], Self)> {
        u8::decode_from(input).map(|(rem, val)| (rem, Guard(val)))
    }
}

// TODO: Treat PVarint like Guard (creates value upon insertion PVarint([u8; 9]))
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PVarint(u64);

impl From<u64> for PVarint {
    fn from(value: u64) -> Self {
        PVarint(value)
    }
}

impl From<PVarint> for u64 {
    fn from(value: PVarint) -> Self {
        value.0
    }
}

impl PVarint {
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl EncodeInto for PVarint {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]> {
        let mut buf = [0u8; 9];
        let value = self.get();
        let zeros = value.leading_zeros();

        // Catch empty u64
        if zeros == 64 {
            0x01u8.encode_into(dst)
        // Catch full u64
        } else if zeros == 0 {
            buf[1..].copy_from_slice(&value.to_le_bytes());
            buf.encode_into(dst)
        // Catch var u64
        } else {
            let bytes = 8 - ((zeros - 1) / 7) as usize;
            let data = value << bytes + 1;
            buf[..=bytes].copy_from_slice(&data.to_le_bytes()[..=bytes]);
            buf[0] |= if bytes >= 8 { 0 } else { 0x01 << bytes };
            (&buf[..=bytes]).encode_into(dst)
        }
    }
}

impl<'a> DecodeFrom<'a> for PVarint {
    fn decode_from(input: &'a [u8]) -> CodecResult<(&'a [u8], Self)> {
        let mut input = input;
        let tag = input.decode::<u8>()?;
        let len = tag.trailing_zeros() as usize;
        let data_slice = input.decode_len::<&[u8]>(len)?;

        let mut data = [0; 8];
        data[..len].copy_from_slice(&data_slice[..len]);
        let data = u64::from_le_bytes(data);
        let out = if len < 7 {
            // Catch tag w/data (0bXXXXXXX1...0bX100000)
            let remainder = tag >> (len + 1); // Remove guard bit
            (data << (7 - len)) + remainder as u64
        } else {
            // Catch tag w/o data (0b1000000 + 0b00000000)
            data
        };

        Ok((input, PVarint(out)))
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::io::test::codec_harness_sized;

    #[test]
    fn codec_pvarint() {
        fn prefixed_array<const N: usize>(prefix: u8) -> [u8; N] {
            assert!(N > 0 && N <= 9, "invalid range");
            let mut buf = [0xFF; N];
            buf[0] = prefix;
            buf
        }
        codec_harness_sized(&prefixed_array::<1>(0xFF), 1, PVarint(127)); // 2^7 - 1
        codec_harness_sized(&prefixed_array::<2>(0xFE), 2, PVarint(16383)); // 2^14 - 1
        codec_harness_sized(&prefixed_array::<3>(0xFC), 3, PVarint(2097151)); // 2^21 - 1
        codec_harness_sized(&prefixed_array::<4>(0xF8), 4, PVarint(268435455)); // 2^28 - 1
        codec_harness_sized(&prefixed_array::<5>(0xF0), 5, PVarint(34359738367)); // 2^35 - 1
        codec_harness_sized(&prefixed_array::<6>(0xE0), 6, PVarint(4398046511103)); // 2^42 - 1
        codec_harness_sized(&prefixed_array::<7>(0xC0), 7, PVarint(562949953421311)); // 2^49 - 1
        codec_harness_sized(&prefixed_array::<8>(0x80), 8, PVarint(72057594037927935)); // 2^56 - 1
        codec_harness_sized(&prefixed_array::<9>(0x00), 9, PVarint(18446744073709551615));
        // 2^64
    }
}
