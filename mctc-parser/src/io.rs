use crate::error::{DecodeError, DecodeResult};

/// Trait for decoding values into slices.
pub trait Encodable {
    /// Length of ```Self``` when encoded.
    fn len_needed(&self) -> usize;
    /// Encode ```Self``` into mut slice.
    ///
    /// # Panics
    /// Will panic if slice is not the same length as ```Self::encode_len()```
    fn encode_into(&self, dst: &mut [u8]);
    /// Encode ```Self``` into mut slice.
    ///
    /// Returns ```None``` if ```dst.len() != Self::encode_len()```
    fn encode_checked(&self, dst: &mut [u8]) -> Option<()> {
        (dst.len() == self.len_needed()).then(|| self.encode_into(dst))
    }
}

// TODO: Error handling (validity)
pub trait EncodeExt<T: Encodable> {
    fn write_encodable(&mut self, val: T);
    fn write_encodable_checked(&mut self, val: T) -> Option<()>;
}

impl<T: Encodable> EncodeExt<T> for &mut [u8] {
    fn write_encodable(&mut self, val: T) {
        let (a, b) = std::mem::take(self).split_at_mut(val.len_needed());
        val.encode_into(a);
        *self = b;
    }

    fn write_encodable_checked(&mut self, val: T) -> Option<()> {
        let (a, b) = std::mem::take(self).split_at_mut_checked(val.len_needed())?;
        val.encode_into(a);
        *self = b;
        Some(())
    }
}

macro_rules! encode_decode_impl {
    ($t:ident) => {
        impl Encodable for $t {
            fn len_needed(&self) -> usize {
                std::mem::size_of::<$t>()
            }

            fn encode_into(&self, dst: &mut [u8]) {
                assert_eq!(dst.len(), self.len_needed());
                dst.copy_from_slice(&self.to_le_bytes());
            }
        }
    };
}

encode_decode_impl!(u8);
encode_decode_impl!(u16);
encode_decode_impl!(u32);
encode_decode_impl!(u64);
encode_decode_impl!(i8);
encode_decode_impl!(i16);
encode_decode_impl!(i32);
encode_decode_impl!(i64);

impl<T: Encodable> Encodable for &T {
    fn len_needed(&self) -> usize {
        (*self).len_needed()
    }

    fn encode_into(&self, dst: &mut [u8]) {
        (*self).encode_into(dst);
    }
}

impl<const N: usize> Encodable for [u8; N] {
    fn len_needed(&self) -> usize {
        std::mem::size_of::<[u8; N]>()
    }

    fn encode_into(&self, dst: &mut [u8]) {
        assert_eq!(dst.len(), self.len_needed());
        dst.copy_from_slice(self);
    }
}

impl Encodable for &[u8] {
    fn len_needed(&self) -> usize {
        std::mem::size_of_val(*self)
    }

    fn encode_into(&self, dst: &mut [u8]) {
        assert_eq!(dst.len(), self.len_needed());
        dst.copy_from_slice(self);
    }
}

impl Encodable for &str {
    fn len_needed(&self) -> usize {
        std::mem::size_of_val(*self)
    }

    fn encode_into(&self, dst: &mut [u8]) {
        assert_eq!(dst.len(), self.len_needed());
        dst.copy_from_slice(self.as_bytes());
    }
}

// TODO: Finish API

pub type FromByteResult<'a, T> = Result<(&'a [u8], T), DecodeError>;

/// Trait for decoding values from slices.
pub trait FromByteSlice<'a>
where
    Self: Sized,
{
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self>;
}

/// Trait for decoding values from slices.
pub trait FromByteSliceBounded<'a>: FromByteSlice<'a>
where
    Self: Sized,
{
    fn from_bytes_bounded(input: &'a [u8], len: usize) -> FromByteResult<'a, Self>;
}

impl<'a, const N: usize> FromByteSlice<'a> for [u8; N] {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
        let (out, rem) = input.split_at_checked(N).ok_or(DecodeError::Needed(N))?;
        Ok((rem, out.try_into().unwrap())) // SAFETY: "out" is always length N
    }
}

macro_rules! from_bytes_impl {
    ($t:ident) => {
        impl<'a> FromByteSlice<'a> for $t {
            fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
                <[u8; size_of::<$t>()]>::from_bytes(input)
                    .map(|(rem, input)| (rem, $t::from_le_bytes(input)))
            }
        }
    };
}

from_bytes_impl!(u8);
from_bytes_impl!(u16);
from_bytes_impl!(u32);
from_bytes_impl!(u64);
from_bytes_impl!(i8);
from_bytes_impl!(i16);
from_bytes_impl!(i32);
from_bytes_impl!(i64);

impl<'a> FromByteSlice<'a> for &'a [u8] {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
        Ok((input, &[]))
    }
}

impl<'a> FromByteSliceBounded<'a> for &'a [u8] {
    fn from_bytes_bounded(input: &'a [u8], len: usize) -> FromByteResult<'a, Self> {
        let (out, rem) = input
            .split_at_checked(len)
            .ok_or(DecodeError::Needed(len))?;
        Ok((out, rem))
    }
}

impl<'a> FromByteSlice<'a> for &'a str {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
        Ok((
            &[],
            str::from_utf8(input).map_err(|_| DecodeError::Badness)?,
        ))
    }
}

impl<'a> FromByteSliceBounded<'a> for &'a str {
    fn from_bytes_bounded(input: &'a [u8], len: usize) -> FromByteResult<'a, Self> {
        let (out, rem) = input
            .split_at_checked(len)
            .ok_or(DecodeError::Needed(len))?;
        Ok((rem, str::from_utf8(out).map_err(|_| DecodeError::Badness)?))
    }
}

pub trait DecodeExt<'a> {
    fn decode<T>(&mut self) -> DecodeResult<T>
    where
        T: FromByteSlice<'a>;
    fn decode_peek<T>(&self) -> DecodeResult<T>
    where
        T: FromByteSlice<'a>;
    fn decode_len<T>(&mut self, len: usize) -> DecodeResult<T>
    where
        T: FromByteSlice<'a> + FromByteSliceBounded<'a>;
    fn decode_assert<T>(&mut self, cmp: T) -> DecodeResult<Option<T>>
    where
        T: FromByteSlice<'a> + PartialEq;
}

impl<'a> DecodeExt<'a> for &'a [u8] {
    fn decode<T>(&mut self) -> DecodeResult<T>
    where
        T: FromByteSlice<'a>,
    {
        let (rem, out) = T::from_bytes(self)?;
        *self = rem;
        Ok(out)
    }

    fn decode_peek<T>(&self) -> DecodeResult<T>
    where
        T: FromByteSlice<'a>,
    {
        let (_, out) = T::from_bytes(self)?;
        Ok(out)
    }

    fn decode_len<T>(&mut self, len: usize) -> DecodeResult<T>
    where
        T: FromByteSlice<'a> + FromByteSliceBounded<'a>,
    {
        let (rem, out) = T::from_bytes_bounded(self, len)?;
        *self = rem;
        Ok(out)
    }

    fn decode_assert<T>(&mut self, cmp: T) -> DecodeResult<Option<T>>
    where
        T: FromByteSlice<'a> + PartialEq,
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

impl<'a> FromByteSlice<'a> for PVarint {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
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
