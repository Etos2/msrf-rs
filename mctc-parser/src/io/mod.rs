pub mod util;

#[cfg(feature = "ascii")]
use std::ascii::Char as AsciiChar;
use std::borrow::Borrow;

use paste::paste;

use crate::error::{CodecError, CodecResult};

fn insert_bytes<'a>(buf: &'a mut [u8], input: &[u8]) -> Result<&'a mut [u8], usize> {
    if input.len() <= buf.len() {
        // SAFETY: mid <= self.len()
        let (dst, rem) = buf.split_at_mut(input.len());
        dst.copy_from_slice(input);
        Ok(rem)
    } else {
        Err(input.len() - buf.len())
    }
}

pub trait EncodeInto
where
    Self: Sized,
{
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]>;
}

pub trait EncodeIntoBounded: EncodeInto
where
    Self: Sized,
{
    fn encode_len_into<'a>(&self, dst: &'a mut [u8], len: usize) -> CodecResult<&'a mut [u8]>;
}

pub trait DecodeFrom<'a>
where
    Self: Sized,
{
    fn decode_from(input: &'a [u8]) -> CodecResult<(&'a [u8], Self)>;
}

pub trait DecodeIntoBounded<'a>: DecodeFrom<'a>
where
    Self: Sized,
{
    fn decode_from_bounded(input: &'a [u8], len: usize) -> CodecResult<(&'a [u8], Self)> {
        let (out, rem) = input
            .split_at_checked(len)
            .ok_or_else(|| CodecError::Needed(len - input.len()))?;
        let (_, val) = Self::decode_from(out)?;
        Ok((rem, val))
    }
}

impl<'a, const N: usize> DecodeFrom<'a> for [u8; N] {
    fn decode_from(input: &[u8]) -> CodecResult<(&[u8], Self)> {
        let (out, rem) = input
            .split_at_checked(N)
            .ok_or_else(|| CodecError::Needed(N - input.len()))?;
        Ok((rem, out.try_into().unwrap())) // SAFETY: "out" is always length N
    }
}

impl<const N: usize> EncodeInto for [u8; N] {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]> {
        insert_bytes(dst, self.as_slice()).map_err(|n| CodecError::Needed(n))
    }
}

impl EncodeInto for &[u8] {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]> {
        insert_bytes(dst, self).map_err(|n| CodecError::Needed(n))
    }
}

impl EncodeIntoBounded for &[u8] {
    fn encode_len_into<'a>(&self, dst: &'a mut [u8], len: usize) -> CodecResult<&'a mut [u8]> {
        insert_bytes(dst, &self[..len]).map_err(|n| CodecError::Needed(n))
    }
}

impl<'a> DecodeFrom<'a> for &'a [u8] {
    fn decode_from(input: &'a [u8]) -> CodecResult<(&'a [u8], Self)> {
        Ok((&[], input))
    }
}

impl<'a> DecodeIntoBounded<'a> for &'a [u8] {}

#[cfg(feature = "ascii")]
impl EncodeInto for &[AsciiChar] {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]> {
        insert_bytes(dst, self.as_bytes()).map_err(|n| CodecError::Needed(n))
    }
}

#[cfg(feature = "ascii")]
impl<'a> DecodeFrom<'a> for &'a [AsciiChar] {
    fn decode_from(input: &'a [u8]) -> CodecResult<(&'a [u8], Self)> {
        Ok((&[], input.as_ascii().ok_or(CodecError::Badness)?))
    }
}

#[cfg(feature = "ascii")]
impl EncodeIntoBounded for &[AsciiChar] {
    fn encode_len_into<'a>(&self, dst: &'a mut [u8], len: usize) -> CodecResult<&'a mut [u8]> {
        insert_bytes(dst, &self.as_bytes()[..len]).map_err(|n| CodecError::Needed(n))
    }
}
#[cfg(feature = "ascii")]
impl<'a> DecodeIntoBounded<'a> for &'a [AsciiChar] {}

macro_rules! codec_impl {
    ($t:ident) => {
        impl EncodeInto for $t {
            fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]> {
                insert_bytes(dst, &self.to_le_bytes()).map_err(|n| CodecError::Needed(n))
            }
        }

        impl<'a> DecodeFrom<'a> for $t {
            fn decode_from(input: &[u8]) -> CodecResult<(&[u8], Self)> {
                <[u8; size_of::<$t>()]>::decode_from(input)
                    .map(|(rem, bytes)| (rem, $t::from_le_bytes(bytes)))
            }
        }
    };
}

codec_impl!(u8);
codec_impl!(u16);
codec_impl!(u32);
codec_impl!(u64);
codec_impl!(i8);
codec_impl!(i16);
codec_impl!(i32);
codec_impl!(i64);

macro_rules! codec_tuple_impl {
    ($($T:tt)*) => {
        paste! {
            impl<$($T,)*> EncodeInto for ($($T,)*)
            where
                $($T: EncodeInto,)*
            {
                fn encode_into<'a>(&self, dst: &'a mut [u8]) -> CodecResult<&'a mut [u8]>
                {
                    let mut dst = dst;
                    let ($([<$T:lower 1>],)*) = self;
                    ($({dst = [<$T:lower 1>].encode_into(dst)?},)*);
                    Ok(dst)
                }
            }
        }

        impl<'a, $($T,)*> DecodeFrom<'a> for ($($T,)*)
        where
            $($T: DecodeFrom<'a>,)*
        {
            fn decode_from(input: &'a [u8]) -> CodecResult<(&'a [u8], Self)> {
                let mut input = input;
                let out = ($(<$T>::decode_from(input).map(|(rem, out)| {
                    input = rem;
                    out
                })?,)*);
                Ok((input, out))
            }
        }
    };
}

codec_tuple_impl!(A B C D E F G H);
codec_tuple_impl!(A B C D E F G);
codec_tuple_impl!(A B C D E F);
codec_tuple_impl!(A B C D E);
codec_tuple_impl!(A B C E);
codec_tuple_impl!(A B C);
codec_tuple_impl!(A B);
codec_tuple_impl!(A);

pub trait EncodeExt {
    fn encode<T>(&mut self, val: impl Borrow<T>) -> CodecResult<()>
    where
        T: EncodeInto;
    fn encode_len<T>(&mut self, val: impl Borrow<T>, len: usize) -> CodecResult<()>
    where
        T: EncodeInto + EncodeIntoBounded;
    fn skip(&mut self, len: usize) -> CodecResult<()>;
}

impl EncodeExt for &mut [u8] {
    fn encode<T>(&mut self, val: impl Borrow<T>) -> CodecResult<()>
    where
        T: EncodeInto,
    {
        let rem = val.borrow().encode_into(std::mem::take(self))?;
        *self = rem;
        Ok(())
    }

    fn encode_len<T>(&mut self, val: impl Borrow<T>, len: usize) -> CodecResult<()>
    where
        T: EncodeInto + EncodeIntoBounded,
    {
        let rem = val.borrow().encode_len_into(std::mem::take(self), len)?;
        *self = rem;
        Ok(())
    }

    fn skip(&mut self, len: usize) -> CodecResult<()> {
        if self.len() >= len {
            let buf = std::mem::take(self);
            *self = &mut buf[len..];
            Ok(())
        } else {
            Err(CodecError::Needed(len - self.len()))
        }
    }
}

pub trait DecodeExt<'a> {
    fn decode<T>(&mut self) -> CodecResult<T>
    where
        T: DecodeFrom<'a>;
    fn decode_len<T>(&mut self, len: usize) -> CodecResult<T>
    where
        T: DecodeFrom<'a> + DecodeIntoBounded<'a>;
    fn decode_assert<T>(&mut self, cmp: T) -> CodecResult<Option<T>>
    where
        T: DecodeFrom<'a> + PartialEq;
    fn skip(&mut self, len: usize) -> CodecResult<()>;
}

impl<'a> DecodeExt<'a> for &'a [u8] {
    fn decode<T>(&mut self) -> CodecResult<T>
    where
        T: DecodeFrom<'a>,
    {
        let (rem, out) = T::decode_from(self)?;
        *self = rem;
        Ok(out)
    }

    fn decode_len<T>(&mut self, len: usize) -> CodecResult<T>
    where
        T: DecodeFrom<'a> + DecodeIntoBounded<'a>,
    {
        let (rem, out) = T::decode_from_bounded(self, len)?;
        *self = rem;
        Ok(out)
    }

    fn decode_assert<T>(&mut self, cmp: T) -> CodecResult<Option<T>>
    where
        T: DecodeFrom<'a> + PartialEq,
    {
        Ok((self.decode::<T>()? == cmp).then_some(cmp))
    }

    fn skip(&mut self, len: usize) -> CodecResult<()> {
        if self.len() >= len {
            let buf = std::mem::take(self);
            *self = &buf[len..];
            Ok(())
        } else {
            Err(CodecError::Needed(len - self.len()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

    pub(crate) fn codec_harness_sized<'a, T, const N: usize>(
        data: &'a [u8; N],
        expected_len: usize,
        expected: T,
    ) where
        T: EncodeInto + DecodeFrom<'a> + PartialEq + std::fmt::Debug,
    {
        let type_len = expected_len;
        let mut input = &data[..];

        // Decode: Assert expected value & remaining buf
        let value = input.decode::<T>().unwrap();
        assert_eq!(value, expected);
        assert_eq!(input, &data[type_len..], "type len = {type_len}");

        // Encode: Assert buffer contents == source contents & remaining buf untouched
        let mut buf = [0; N];
        let mut output = &mut buf[..];
        (output).encode(value).unwrap();
        assert_eq!(output.len(), buf.len() - type_len);
        assert_eq!(&buf[..type_len], &data[..type_len]);
    }

    // Decode/Encode one value from data (even if data has more bytes than needed)
    pub(crate) fn codec_harness<'a, T, const N: usize>(data: &'a [u8; N], expected: T)
    where
        T: EncodeInto + DecodeFrom<'a> + PartialEq + std::fmt::Debug,
    {
        codec_harness_sized(data, size_of_tuple!(T), expected)
    }

    #[test]
    fn codec_value() {
        codec_harness(&DATA, DATA);
        codec_harness(&DATA, 0x00u8);
        codec_harness(&DATA, 0x0100u16);
        codec_harness(&DATA, 0x03020100u32);
        codec_harness(&DATA, 0x0706050403020100u64);
        codec_harness(&DATA, 0x00i8);
        codec_harness(&DATA, 0x0100i16);
        codec_harness(&DATA, 0x03020100i32);
        codec_harness(&DATA, 0x0706050403020100i64);
    }

    #[test]
    fn codec_tuple() {
        codec_harness(&DATA, (0x00u8, 0x01u8, 0x02u8, 0x03u8));
        codec_harness(&DATA, (0x0100u16, 0x0302u16, 0x0504u16, 0x0706u16));
        codec_harness(&DATA, (0x03020100u32, 0x07060504u32, 0x0B0A0908u32));
        codec_harness(&DATA, (0x0706050403020100u64, 0x0F0E0D0C0B0A0908u64));
        codec_harness(&DATA, (0x00u8, 0x01u8, 0x0302u16, 0x07060504u32));
    }
}
