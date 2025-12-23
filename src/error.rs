use std::{error::Error, fmt::Display};

// TODO: Re-evaluate variant nessicity (e.g. length?)
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ParserError {
    Need(usize), // TODO: Remove
    Unsupported(u16),
    Guard(u8),
    MagicBytes([u8; 4]),
    Length(u64),
    UnexpectedEos,
    IsEos,
}

impl Error for ParserError {}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Need(n) => write!(f, "need {n} more bytes to continue"),
            Self::Unsupported(ver) => {
                write!(f, "unsupported version (v{ver})")
            }
            Self::Guard(g) => write!(f, "expected guard ({g})"),
            Self::MagicBytes(b) => write!(f, "invalid magic bytes ({b:?})"),
            Self::Length(l) => write!(f, "invalid length ({l})"),
            Self::UnexpectedEos => write!(f, "unexpected eos"),
            Self::IsEos => write!(f, "already recieved eos"),
        }
    }
}

#[derive(Debug)]
pub enum IoError<E> {
    Parser(E),
    Io(std::io::Error),
}

impl<E: Error> Error for IoError<E> {}

impl<E: Error> Display for IoError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::Parser(e) => Display::fmt(&e, f),
            IoError::Io(e) => e.fmt(f),
        }
    }
}

// TODO: Generic impl? Requires specialisation for io::Error
impl From<ParserError> for IoError<ParserError> {
    fn from(value: ParserError) -> Self {
        Self::Parser(value)
    }
}

impl<E: Error> From<std::io::Error> for IoError<E> {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
