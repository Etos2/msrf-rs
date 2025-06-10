use std::{error::Error, fmt::Display};

pub type DecodeResult<T> = Result<T, DecodeError>;
pub type EncodeResult<T> = Result<T, EncodeError>;

// TODO: Merge with EncodeError?
// TODO: Handle all cases (whatever they may be)
#[derive(Debug)]
pub enum DecodeError {
    Needed(usize),
    ExpectedGuard,
    Badness,
}

impl DecodeError {
    fn need<T>() -> DecodeError {
        DecodeError::Needed(size_of::<T>())
    }
}

impl Error for DecodeError {}

impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::Needed(n) => writeln!(f, "need {n} more bytes"),
            DecodeError::ExpectedGuard => writeln!(f, "expected guard"),
            DecodeError::Badness => writeln!(f, "bad!"),
        }
    }
}

// TODO: Merge with DecodeError?
// TODO: Handle all cases (whatever they may be)
#[derive(Debug)]
pub enum EncodeError {
    Needed(usize),
    Truncation,
    Badness,
}

impl Error for EncodeError {}

impl Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::Needed(n) => writeln!(f, "need {n} more bytes"),
            EncodeError::Truncation => writeln!(f, "value was too large"),
            EncodeError::Badness => writeln!(f, "bad!"),
        }
    }
}

