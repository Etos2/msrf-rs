use std::{ascii::Char as AsciiChar, borrow::Borrow};

use paste::paste;

use crate::error::{DecodeError, DecodeResult, EncodeError, EncodeResult};

pub trait EncodeSlice
where
    Self: Sized,
{
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> EncodeResult<&'a mut [u8]>;
}

pub trait EncodeSliceBounded: EncodeSlice
where
    Self: Sized,
{
    fn encode_len_into<'a>(&self, dst: &'a mut [u8], len: usize) -> EncodeResult<&'a mut [u8]> {
        let (dst, rem) = dst
            .split_at_mut_checked(len)
            .ok_or(EncodeError::Needed(len))?;
        self.encode_into(dst)?;
        Ok(rem)
    }
}

impl<const N: usize> EncodeSlice for [u8; N] {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
        // TODO: inaccurate (should be N - input.len())
        let (dst, rem) = dst.split_at_mut_checked(N).ok_or(EncodeError::Needed(N))?;
        dst.copy_from_slice(self);
        Ok(rem)
    }
}

impl EncodeSliceBounded for &[u8] {}

impl EncodeSlice for &[u8] {
    fn encode_into<'a>(&self, dst: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
        let dst_len = dst.len();
        match dst.split_at_mut_checked(self.len()) {
            Some((dst, rem)) => {
                dst.copy_from_slice(self);
                Ok(rem)
            }
            None => Err(EncodeError::Needed(self.len() - dst_len)),
        }
    }
}

impl EncodeSliceBounded for &[AsciiChar] {}

impl EncodeSlice for &[AsciiChar] {
    fn encode_into<'a>(&self, input: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
        self.as_bytes().encode_into(input)
    }
}

macro_rules! encode_impl {
    ($t:ident) => {
        impl EncodeSlice for $t {
            fn encode_into<'a>(&self, input: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
                self.to_le_bytes().encode_into(input)
            }
        }
    };
}

encode_impl!(u8);
encode_impl!(u16);
encode_impl!(u32);
encode_impl!(u64);
encode_impl!(i8);
encode_impl!(i16);
encode_impl!(i32);
encode_impl!(i64);

macro_rules! encode_tuple_impl {
    ($($T:tt)*) => {
        paste! {
            impl<$($T,)*> EncodeSlice for ($($T,)*)
            where
                $($T: EncodeSlice,)*
            {
                fn encode_into<'a>(&self, input: &'a mut [u8]) -> EncodeResult<&'a mut [u8]> {
                    let mut input = input;
                    let ($([<$T:lower 1>],)*) = self;
                    ($({input = [<$T:lower 1>].encode_into(std::mem::take(&mut input))?},)*);
                    Ok(input)
                }
            }
        }
    };
}

encode_tuple_impl!(A B C D E F G H);
encode_tuple_impl!(A B C D E F G);
encode_tuple_impl!(A B C D E F);
encode_tuple_impl!(A B C D E);
encode_tuple_impl!(A B C E);
encode_tuple_impl!(A B C);
encode_tuple_impl!(A B);
encode_tuple_impl!(A);

pub trait EncodeExt {
    fn encode<T>(&mut self, val: impl Borrow<T>) -> EncodeResult<()>
    where
        T: EncodeSlice;
    fn encode_len<T>(&mut self, val: impl Borrow<T>, len: usize) -> EncodeResult<()>
    where
        T: EncodeSlice + EncodeSliceBounded;
}

impl EncodeExt for &mut [u8] {
    fn encode<T>(&mut self, val: impl Borrow<T>) -> EncodeResult<()>
    where
        T: EncodeSlice,
    {
        let rem = val.borrow().encode_into(std::mem::take(self))?;
        *self = rem;
        Ok(())
    }

    fn encode_len<T>(&mut self, val: impl Borrow<T>, len: usize) -> EncodeResult<()>
    where
        T: EncodeSlice + EncodeSliceBounded,
    {
        let rem = val.borrow().encode_len_into(std::mem::take(self), len)?;
        *self = rem;
        Ok(())
    }
}

pub trait DecodeSlice<'a>
where
    Self: Sized,
{
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)>;
}

pub trait DecodeSliceBounded<'a>: DecodeSlice<'a>
where
    Self: Sized,
{
    fn decode_from_bounded(input: &'a [u8], len: usize) -> DecodeResult<(&'a [u8], Self)> {
        let (out, rem) = input
            .split_at_checked(len)
            .ok_or(DecodeError::Needed(len))?; // TODO: inaccurate (should be N - input.len())
        let (_, val) = Self::decode_from(out)?;
        Ok((rem, val))
    }
}

impl<'a, const N: usize> DecodeSlice<'a> for [u8; N] {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let (out, rem) = input.split_at_checked(N).ok_or(DecodeError::Needed(N))?; // TODO: inaccurate (should be N - input.len())
        Ok((rem, out.try_into().unwrap())) // SAFETY: "out" is always length N
    }
}

impl<'a> DecodeSliceBounded<'a> for &'a [u8] {}

impl<'a> DecodeSlice<'a> for &'a [u8] {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        Ok((&[], input))
    }
}

impl<'a> DecodeSliceBounded<'a> for &'a [AsciiChar] {}

impl<'a> DecodeSlice<'a> for &'a [AsciiChar] {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        Ok((&[], input.as_ascii().ok_or(DecodeError::Badness)?))
    }
}

macro_rules! decode_impl {
    ($t:ident) => {
        impl<'a> DecodeSlice<'a> for $t {
            fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
                <[u8; size_of::<$t>()]>::decode_from(input)
                    .map(|(rem, bytes)| (rem, $t::from_le_bytes(bytes)))
            }
        }
    };
}

decode_impl!(u8);
decode_impl!(u16);
decode_impl!(u32);
decode_impl!(u64);
decode_impl!(i8);
decode_impl!(i16);
decode_impl!(i32);
decode_impl!(i64);

macro_rules! decode_tuple_impl {
    ($($T:tt)*) => {
        impl<'a, $($T,)*> DecodeSlice<'a> for ($($T,)*)
        where
            $($T: DecodeSlice<'a>,)*
        {
            fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
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

decode_tuple_impl!(A B C D E F G H);
decode_tuple_impl!(A B C D E F G);
decode_tuple_impl!(A B C D E F);
decode_tuple_impl!(A B C D E);
decode_tuple_impl!(A B C D);
decode_tuple_impl!(A B C);
decode_tuple_impl!(A B);
decode_tuple_impl!(A);

pub trait DecodeExt<'a> {
    fn decode<T>(&mut self) -> DecodeResult<T>
    where
        T: DecodeSlice<'a>;
    fn decode_len<T>(&mut self, len: usize) -> DecodeResult<T>
    where
        T: DecodeSlice<'a> + DecodeSliceBounded<'a>;
    fn decode_assert<T>(&mut self, cmp: T) -> DecodeResult<Option<T>>
    where
        T: DecodeSlice<'a> + PartialEq;
}

impl<'a> DecodeExt<'a> for &'a [u8] {
    fn decode<T>(&mut self) -> DecodeResult<T>
    where
        T: DecodeSlice<'a>,
    {
        let (rem, out) = T::decode_from(self)?;
        *self = rem;
        Ok(out)
    }

    fn decode_len<T>(&mut self, len: usize) -> DecodeResult<T>
    where
        T: DecodeSlice<'a> + DecodeSliceBounded<'a>,
    {
        let (rem, out) = T::decode_from_bounded(self, len)?;
        *self = rem;
        Ok(out)
    }

    fn decode_assert<T>(&mut self, cmp: T) -> DecodeResult<Option<T>>
    where
        T: DecodeSlice<'a> + PartialEq,
    {
        Ok((self.decode::<T>()? == cmp).then_some(cmp))
    }
}

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

impl<'a> DecodeSlice<'a> for PVarint {
    fn decode_from(input: &'a [u8]) -> DecodeResult<(&'a [u8], Self)> {
        let mut input = input;
        let tag = input.decode::<u8>()?;
        let len = tag.trailing_zeros() as usize;
        let data_slice = input.decode_len::<&[u8]>(len)?;

        let mut data = [0; 8];
        data.copy_from_slice(data_slice);
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
mod test {
    use std::fmt::Debug;

    use super::*;

    const DATA: [u8; 32] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F];

    // Decode/Encode one value from data (even if data has more bytes than needed)
    fn codec_harness<'a, T, const N: usize>(data: &'a [u8; N], expected: T)
    where
        T: EncodeSlice + DecodeSlice<'a> + PartialEq + Debug,
    {
        let type_len = size_of::<T>();
        let mut input = &data[..];

        // Decode: Assert expected value & remaining buf
        let value = input.decode::<T>().unwrap();
        assert_eq!(value, expected);
        assert_eq!(input, &data[type_len..]);

        // Encode: Assert buffer contents == source contents & remaining buf untouched
        let mut buf = [0; N];
        let mut output = &mut buf[..];
        (output).encode(value).unwrap();
        assert_eq!(output.len(), buf.len() - type_len);
        assert_eq!(&buf[..type_len], &data[..type_len]);
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
        codec_harness(&DATA, (0x00u8, 0x01u8, 0x02u8, 0x03u8, 0x04u8, 0x05u8, 0x06u8, 0x07u8));
        codec_harness(&DATA, (0x0100u16, 0x0302u16, 0x0504u16, 0x0706u16));
        codec_harness(&DATA, (0x03020100u32, 0x07060504u32, 0x0B0A0908u32, 0x0F0E0D0Cu32));
        codec_harness(&DATA, (0x0706050403020100u64, 0x0F0E0D0C0B0A0908u64));
    }
}
