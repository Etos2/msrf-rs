use std::{
    error::Error,
    fmt::Display,
};

// TODO: Handle all cases (whatever they may be)
pub type DecodeResult<T> = Result<T, DecodeError>;

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