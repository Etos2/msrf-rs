use std::{
    error::Error,
    fmt::Display,
    ops::{Bound, Range, RangeBounds},
    string::FromUtf8Error,
};

pub type PResult<T> = Result<T, PError>;

#[derive(Debug)]
pub enum PError {
    Io(std::io::Error),
    MismatchBytes { found: Vec<u8>, expected: Vec<u8> },
    OutsideRange { found: u64, range: Range<u64> },
    Utf8(FromUtf8Error),
    NoCodec(u8),
}

impl PError {
    pub(crate) fn new_range<T: Into<u64> + Copy>(found: T, range: impl RangeBounds<T>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(s) => (*s).into(),
            Bound::Excluded(s) => (*s).into() + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(s) => (*s).into() + 1,
            Bound::Excluded(s) => (*s).into(),
            Bound::Unbounded => u64::MAX,
        };
        PError::OutsideRange {
            found: found.into(),
            range: start..end,
        }
    }
}

impl Error for PError {}

impl Display for PError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<std::io::Error> for PError {
    fn from(value: std::io::Error) -> Self {
        PError::Io(value)
    }
}

impl From<FromUtf8Error> for PError {
    fn from(value: FromUtf8Error) -> Self {
        PError::Utf8(value)
    }
}
