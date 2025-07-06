pub mod util;

use std::convert::Infallible;

// TODO: Use type to clarify?
// Usize = Bytes needed
pub type SerialiseError<E> = Result<usize, E>;
// Usize = Bytes written
pub type EncodeResult<E> = Result<usize, SerialiseError<E>>;
pub type DecodeResult<T, E> = Result<(usize, T), SerialiseError<E>>;
pub type EncodeExtResult<E> = Result<(), SerialiseError<E>>;
pub type DecodeExtResult<T, E> = Result<T, SerialiseError<E>>;

// TODO: Partial serialisation?
// pub trait SerialisableCtx
// where
//     Self: Sized + Default,
// {
//     type Err: std::error::Error;
//     fn encode_ctx(&self, buf: &mut [u8], prev_write: usize) -> EncodeResult<Self::Err>;
//     fn decode_ctx(self, buf: &[u8], prev_read: usize) -> DecodeResult<Self, Self::Err>;
// }

pub trait Serialisable<'a>
where
    Self: Sized,
{
    type Err: std::error::Error;
    fn encode_into(&self, buf: &mut [u8]) -> EncodeResult<Self::Err>;
    fn decode_from(buf: &'a [u8]) -> DecodeResult<Self, Self::Err>;
}

pub trait SerialisableVariable<'a>
where
    Self: Sized,
{
    type Err: std::error::Error;
    fn encode_into(&self, buf: &mut [u8], len: usize) -> EncodeResult<Self::Err>;
    fn decode_from(buf: &'a [u8], len: usize) -> DecodeResult<Self, Self::Err>;
}

impl<const N: usize> Serialisable<'_> for [u8; N]
where
    Self: Default,
{
    type Err = Infallible;

    fn encode_into(&self, buf: &mut [u8]) -> EncodeResult<Self::Err> {
        let dst = buf.get_mut(..N).ok_or_else(|| Ok(N))?;
        dst.copy_from_slice(&self[..N]);
        Ok(N)
    }

    fn decode_from(buf: &[u8]) -> DecodeResult<Self, Self::Err> {
        let mut out = Self::default();
        let src = buf.get(..N).ok_or_else(|| Ok(N))?;
        out[..N].copy_from_slice(src);
        Ok((N, out))
    }
}

impl<'a> SerialisableVariable<'a> for &'a [u8] {
    type Err = Infallible;

    fn encode_into(&self, buf: &mut [u8], len: usize) -> EncodeResult<Self::Err> {
        let dst = buf.get_mut(..len).ok_or_else(|| Ok(len))?;
        dst.copy_from_slice(&self[..len]);
        Ok(len)
    }

    fn decode_from(buf: &'a [u8], len: usize) -> DecodeResult<Self, Self::Err> {
        let src = buf.get(..len).ok_or_else(|| Ok(len))?;
        Ok((len, src))
    }
}

macro_rules! serialisable_impl {
    ($t:ident) => {
        impl Serialisable<'_> for $t {
            type Err = Infallible;

            fn encode_into(&self, buf: &mut [u8]) -> EncodeResult<Self::Err> {
                self.to_le_bytes().encode_into(buf)
            }

            fn decode_from(buf: &[u8]) -> DecodeResult<Self, Self::Err> {
                <[u8; size_of::<$t>()]>::decode_from(buf)
                    .map(|(written, val)| (written, <$t>::from_le_bytes(val)))
            }
        }
    };
}

serialisable_impl!(u8);
serialisable_impl!(u16);
serialisable_impl!(u32);
serialisable_impl!(u64);
serialisable_impl!(i8);
serialisable_impl!(i16);
serialisable_impl!(i32);
serialisable_impl!(i64);

pub trait SerialiseExt<'a> {
    fn encode<T>(self: &mut &'a mut Self, val: T) -> EncodeExtResult<T::Err>
    where
        T: Serialisable<'a>;
    fn encode_len<T>(self: &mut &'a mut Self, val: T, len: usize) -> EncodeExtResult<T::Err>
    where
        T: SerialisableVariable<'a>;
    fn decode<T>(self: &mut &'a Self) -> DecodeExtResult<T, T::Err>
    where
        T: Serialisable<'a>;
    fn decode_len<T>(self: &mut &'a Self, len: usize) -> DecodeExtResult<T, T::Err>
    where
        T: SerialisableVariable<'a>;
    fn skip(self: &mut &'a Self, len: usize) -> Result<(), Result<usize, Infallible>>;
}

impl<'a> SerialiseExt<'a> for [u8] {
    fn encode<T>(self: &mut &'a mut Self, val: T) -> EncodeExtResult<T::Err>
    where
        T: Serialisable<'a>,
    {
        let buf = std::mem::take(self);
        let written = val.encode_into(buf)?;
        *self = &mut buf[written..];
        Ok(())
    }

    fn encode_len<T>(self: &mut &'a mut Self, val: T, len: usize) -> EncodeExtResult<T::Err>
    where
        T: SerialisableVariable<'a>,
    {
        let buf = std::mem::take(self);
        let written = val.encode_into(buf, len)?;
        *self = &mut buf[written..];
        Ok(())
    }

    fn decode<T>(self: &mut &'a Self) -> DecodeExtResult<T, T::Err>
    where
        T: Serialisable<'a>,
    {
        let (read, val) = T::decode_from(self)?;
        *self = &self[read..];
        Ok(val)
    }

    fn decode_len<T>(self: &mut &'a Self, len: usize) -> DecodeExtResult<T, T::Err>
    where
        T: SerialisableVariable<'a>,
    {
        let (read, val) = T::decode_from(self, len)?;
        *self = &self[read..];
        Ok(val)
    }

    fn skip(self: &mut &'a Self, len: usize) -> Result<(), Result<usize, Infallible>> {
        let resized = self.get(len..).ok_or_else(|| Ok(self.len() - len))?;
        *self = resized;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use super::*;

    // fn serialiser_test_harness<T>(data: &[u8], expected: T)
    // where
    //     T: Serialisable + SerialisableCtx + PartialEq + Debug,
    // {
    //     let mid = data.len() / 2;

    //     // Decode
    //     let (rem, val) = <T>::decode(&data[..mid]).expect("decode error");
    //     assert_eq!(rem, mid);
    //     // assert_eq!(val, expected);

    //     // Decode remainder
    //     let (rem, val) = val.decode_ctx(&data[mid..], rem).expect("decode error");
    //     assert_eq!(rem, 0);
    //     assert_eq!(val, expected);

    //     // Encode
    //     let mut buf = vec![0; data.len()];
    //     let rem = val.encode(&mut buf[..mid]).expect("encode error");
    //     assert_eq!(rem, mid);
    //     assert_eq!(buf[..mid], data[..mid]);

    //     // Encode remainder
    //     let rem = val.encode_ctx(&mut buf[mid..], rem).expect("encode error");
    //     assert_eq!(rem, 0);
    //     assert_eq!(buf, *data);
    // }

    // #[test]
    // fn partial_serial_array() {
    //     let data = [0, 1, 2, 3, 4, 5, 6, 7];
    //     serialiser_test_harness(&data, data);
    //     serialiser_test_harness(&data[..2], i16::from_le_bytes(data[..2].try_into().unwrap()));
    //     serialiser_test_harness(&data[..2], u16::from_le_bytes(data[..2].try_into().unwrap()));
    //     serialiser_test_harness(&data[..4], i32::from_le_bytes(data[..4].try_into().unwrap()));
    //     serialiser_test_harness(&data[..4], u32::from_le_bytes(data[..4].try_into().unwrap()));
    //     serialiser_test_harness(&data[..8], i64::from_le_bytes(data[..8].try_into().unwrap()));
    //     serialiser_test_harness(&data[..8], u64::from_le_bytes(data[..8].try_into().unwrap()));
    // }

    const DATA: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D,
        0x1E, 0x1F,
    ];

    // TODO: Does not handle recursive tuples
    macro_rules! size_of_tuple {
        ($($T:tt)*) => {
            {
                let mut size = 0;
                ($(size += size_of::<$T>())*);
                size
            }

        };
    }

    // pub(crate) fn codec_harness_sized<'a, T, const N: usize>(
    //     data: &'a [u8; N],
    //     expected_len: usize,
    //     expected: T,
    // ) where
    //     T: Serialisable + PartialEq + std::fmt::Debug,
    // {
    //     let type_len = expected_len;
    //     let mut input = &data[..];

    //     // Decode: Assert expected value & remaining buf
    //     let value = input.decode::<T>().unwrap();
    //     assert_eq!(value, expected);
    //     assert_eq!(input, &data[type_len..], "type len = {type_len}");

    //     // Encode: Assert buffer contents == source contents & remaining buf untouched
    //     let mut buf = [0; N];
    //     let mut output = &mut buf[..];
    //     (output).encode(value).unwrap();
    //     assert_eq!(output.len(), buf.len() - type_len);
    //     assert_eq!(&buf[..type_len], &data[..type_len]);
    // }

    // // Decode/Encode one value from data (even if data has more bytes than needed)
    // pub(crate) fn codec_harness<'a, T, const N: usize>(data: &'a [u8; N], expected: T)
    // where
    //     T: Serialisable + PartialEq + std::fmt::Debug,
    // {
    //     codec_harness_sized(data, size_of_tuple!(T), expected)
    // }

    // #[test]
    // fn codec_value() {
    //     codec_harness(&DATA, DATA);
    //     codec_harness(&DATA, 0x00u8);
    //     codec_harness(&DATA, 0x0100u16);
    //     codec_harness(&DATA, 0x03020100u32);
    //     codec_harness(&DATA, 0x0706050403020100u64);
    //     codec_harness(&DATA, 0x00i8);
    //     codec_harness(&DATA, 0x0100i16);
    //     codec_harness(&DATA, 0x03020100i32);
    //     codec_harness(&DATA, 0x0706050403020100i64);
    // }

    //     #[test]
    //     fn codec_tuple() {
    //         codec_harness(&DATA, (0x00u8, 0x01u8, 0x02u8, 0x03u8));
    //         codec_harness(&DATA, (0x0100u16, 0x0302u16, 0x0504u16, 0x0706u16));
    //         codec_harness(&DATA, (0x03020100u32, 0x07060504u32, 0x0B0A0908u32));
    //         codec_harness(&DATA, (0x0706050403020100u64, 0x0F0E0D0C0B0A0908u64));
    //         codec_harness(&DATA, (0x00u8, 0x01u8, 0x0302u16, 0x07060504u32));
    //     }
}
