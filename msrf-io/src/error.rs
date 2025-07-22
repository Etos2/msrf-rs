use std::fmt::{self, Display};

pub type CodecResult<T> = Result<T, CodecError>;

#[derive(Debug)]
pub enum CodecError {
    Needed(usize),
    MagicByte([u8; 4]),
    Guard,
    Length(u64),
}

impl Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::Needed(bytes) => write!(f, "need {bytes} bytes"),
            CodecError::MagicByte(magic) => {
                write!(f, "invalid magic bytes \"{magic:?}\"")
            }
            CodecError::Guard => write!(f, "invalid guard"),
            CodecError::Length(len) => {
                write!(f, "invalid length \"{len}\"")
            }
        }
    }
}
