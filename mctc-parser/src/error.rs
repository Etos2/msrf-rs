use std::{
    error::Error,
    fmt::Display,
    ops::{Bound, Range, RangeBounds},
    string::FromUtf8Error,
    sync::Arc,
};

pub type PResult<T> = Result<T, PError>;

#[derive(Debug, Clone)]
pub enum StringEncoding {
    Utf8,
}

impl Display for StringEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringEncoding::Utf8 => write!(f, "utf8"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PError {
    Io(Arc<std::io::Error>),
    InvalidString(StringEncoding),
    MismatchBytes { found: Vec<u8>, expected: Vec<u8> },
    OutsideRange { found: u64, range: Range<u64> },
    NoCodec(u16),
    DuplicateCodec(u64),
    InvalidGVE,
    UnexpectedEos,
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PError::Io(e) => e.fmt(f),
            PError::InvalidString(enc) => write!(f, "invalid string ({enc})"),
            PError::MismatchBytes { found, expected } => {
                write!(f, "found {found:?} expected {expected:?}")
            }
            PError::OutsideRange { found, range } => write!(
                f,
                "found {found:?} outside range {}..{}",
                range.start, range.end
            ),
            PError::NoCodec(id) => write!(f, "codec with id \'{id}\' does not exist"),
            PError::DuplicateCodec(id) => write!(f, "codec with id \'{id}\' already exists"),
            PError::InvalidGVE => write!(f, "invalid gve (maximum 8 bytes)"),
            PError::UnexpectedEos => write!(f, "unexpected eos before stream has ended"),
        }
    }
}

impl From<std::io::Error> for PError {
    fn from(value: std::io::Error) -> Self {
        PError::Io(Arc::new(value))
    }
}

impl From<FromUtf8Error> for PError {
    fn from(_value: FromUtf8Error) -> Self {
        PError::InvalidString(StringEncoding::Utf8)
    }
}
