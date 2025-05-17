use std::{ascii::Char as AsciiChar, borrow::Borrow};

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
