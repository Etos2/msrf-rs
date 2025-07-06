use std::{convert::Infallible, error::Error, fmt::Display};

pub type CodecResult<T> = Result<T, CodecError>;

// TODO: Handle all cases (whatever they may be)
#[derive(Debug)]
pub enum CodecError {
    Needed(usize),
    ExpectedGuard,
    Badness,
}

impl Error for CodecError {}

impl Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::Needed(n) => writeln!(f, "need {n} more bytes"),
            CodecError::ExpectedGuard => writeln!(f, "expected guard"),
            CodecError::Badness => writeln!(f, "bad!"),
        }
    }
}

impl From<Infallible> for CodecError {
    fn from(value: Infallible) -> Self {
        panic!("infallible value should not exist")
    }
}