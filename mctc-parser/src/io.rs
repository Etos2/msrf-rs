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
/// Trait for decoding values from slices.
pub trait Decodable
where
    Self: Sized,
{
    /// Length of slice needed to decode ```Self```.
    /// None if length cannot yet be determined.
    fn bytes_needed(src: &[u8]) -> Option<usize>;
    /// Decode Self from slice.
    ///
    /// # Panics
    /// Will panic if slice is not the same length as ```Self::decode_len(src)```
    fn decode_from(src: &[u8]) -> Self;
    /// Decode ```Self``` from slice.
    ///
    /// Returns ```None``` if ```src.len() != Self::decode_len(src)```
    fn decode_checked(src: &[u8]) -> Option<Self> {
        (src.len() == Self::bytes_needed(src)?).then(|| Self::decode_from(src))
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

// TODO: differentiate non-checked and checked?
pub trait DecodeExt<T: Decodable> {
    fn read_decode(&mut self) -> Option<T>;
    fn read_decode_checked(&mut self) -> Option<T>;
}

impl<T: Decodable> DecodeExt<T> for &[u8] {
    fn read_decode(&mut self) -> Option<T> {
        let len = T::bytes_needed(self)?;
        let (a, b) = std::mem::take(self).split_at(len);
        *self = b;
        Some(T::decode_from(a))
    }

    fn read_decode_checked(&mut self) -> Option<T> {
        let len = T::bytes_needed(self)?;
        let (a, b) = std::mem::take(self).split_at_checked(len)?;
        *self = b;
        Some(T::decode_from(a))
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

        impl Decodable for $t {
            fn bytes_needed(_src: &[u8]) -> Option<usize> {
                Some(std::mem::size_of::<$t>())
            }

            fn decode_from(src: &[u8]) -> Self {
                assert_eq!(src.len(), Self::bytes_needed(src).unwrap());
                <$t>::from_le_bytes(src.try_into().unwrap())
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

impl<const N: usize> Decodable for [u8; N] {
    fn bytes_needed(_src: &[u8]) -> Option<usize> {
        Some(std::mem::size_of::<[u8; N]>())
    }

    fn decode_from(src: &[u8]) -> Self {
        src.try_into().unwrap()
    }
}

// TODO: Finish API

// TODO: Handle all cases (whatever they may be)
pub enum DecodeError {
    Needed(usize),
    InvalidGuard,
    Badness,
}

impl DecodeError {
    fn need<T>() -> DecodeError {
        DecodeError::Needed(size_of::<T>())
    }
}

pub type FromByteResult<'a, T> = Result<(&'a [u8], T), DecodeError>;
pub type DecodeResult<T> = Result<T, DecodeError>;

/// Trait for decoding values from slices.
pub trait FromByteSlice<'a>
where
    Self: Sized,
{
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self>;
    fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self>;
}

/// Trait for decoding values from slices.
pub trait FromByteSliceBounded<'a>: FromByteSlice<'a>
where
    Self: Sized,
{
    fn from_bytes_bounded(input: &'a [u8], len: usize) -> FromByteResult<'a, Self>;
    fn from_bytes_bounded_checked(input: &'a [u8], len: usize) -> FromByteResult<'a, Self>;
}

// TODO: Rewrite to take mut and replace "self" with "rem" (allows `let val = src.decode::<T>()?;`)
pub trait DecodeExt2<'a> {
    fn decode<T: FromByteSlice<'a>>(&mut self) -> DecodeResult<T>;
    fn decode_len<T: FromByteSlice<'a> + FromByteSliceBounded<'a>>(
        &mut self,
        len: usize,
    ) -> DecodeResult<T>;
    fn decode_assert<T: FromByteSlice<'a> + PartialEq>(
        &mut self,
        cmp: T,
    ) -> DecodeResult<Option<T>>;
}

impl<'a> DecodeExt2<'a> for &'a [u8] {
    fn decode<T: FromByteSlice<'a>>(&mut self) -> DecodeResult<T> {
        let (rem, out) = T::from_bytes_checked(self)?;
        *self = rem;
        Ok(out)
    }

    fn decode_len<T: FromByteSlice<'a> + FromByteSliceBounded<'a>>(
        &mut self,
        len: usize,
    ) -> DecodeResult<T> {
        let (rem, out) = T::from_bytes_bounded_checked(self, len)?;
        *self = rem;
        Ok(out)
    }

    fn decode_assert<T: FromByteSlice<'a> + PartialEq>(
        &mut self,
        cmp: T,
    ) -> DecodeResult<Option<T>> {
        Ok((self.decode::<T>()? == cmp).then_some(cmp))
    }
}

impl<'a, const N: usize> FromByteSlice<'a> for [u8; N] {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
        let (out, rem) = input.split_at(N); // PANIC: Panics if N > input.len()
        Ok((rem, out.try_into().unwrap())) // SAFETY: "out" is always length N
    }

    fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self> {
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

            fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self> {
                <[u8; size_of::<$t>()]>::from_bytes_checked(input)
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

    fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self> {
        Self::from_bytes(input)
    }
}

impl<'a> FromByteSliceBounded<'a> for &'a [u8] {
    fn from_bytes_bounded(input: &'a [u8], len: usize) -> FromByteResult<'a, Self> {
        let (out, rem) = input.split_at(len); // PANIC: Panics if len > input.len()
        Ok((out, rem))
    }

    fn from_bytes_bounded_checked(input: &'a [u8], len: usize) -> FromByteResult<'a, Self> {
        let (out, rem) = input
            .split_at_checked(len)
            .ok_or(DecodeError::Needed(len))?;
        Ok((out, rem))
    }
}

impl<'a> FromByteSlice<'a> for &'a str {
    fn from_bytes(input: &'a [u8]) -> FromByteResult<'a, Self> {
        Ok((&[], str::from_utf8(input).unwrap())) // PANIC: Panics if input contains invalid UTF8
    }

    fn from_bytes_checked(input: &'a [u8]) -> FromByteResult<'a, Self> {
        Ok((
            &[],
            str::from_utf8(input).map_err(|_| DecodeError::Badness)?,
        ))
    }
}

impl<'a> FromByteSliceBounded<'a> for &'a str {
    fn from_bytes_bounded(input: &'a [u8], len: usize) -> FromByteResult<'a, Self> {
        let (out, rem) = input.split_at(len); // PANIC: Panics if len > input.len()
        Ok((rem, str::from_utf8(out).unwrap())) // PANIC: Panics if input contains invalid UTF8
    }

    fn from_bytes_bounded_checked(input: &'a [u8], len: usize) -> FromByteResult<'a, Self> {
        let (out, rem) = input
            .split_at_checked(len)
            .ok_or(DecodeError::Needed(len))?;
        Ok((rem, str::from_utf8(out).map_err(|_| DecodeError::Badness)?))
    }
}
