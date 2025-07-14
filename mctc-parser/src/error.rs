use std::fmt::{self, Display};

use crate::codec::constants::{MAGIC_BYTES, RECORD_META_MIN_LEN};

pub type CodecResult<T> = Result<T, CodecError>;

#[derive(Debug)]
pub enum CodecError {
    Needed(usize),
    MagicByte([u8; 4]),
    Guard(u8),
    // TODO: Generic invalid length (length value could simply be incorrect, rather than record meta invariance)
    RecordLength(u64),
}

impl Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::Needed(bytes) => write!(f, "need {bytes} bytes"),
            CodecError::MagicByte(magic) => write!(
                f,
                "expected magic bytes \"{MAGIC_BYTES:?}\" found \"{magic:?}\""
            ),
            CodecError::Guard(b) => write!(f, "invalid guard \"{b:#X}\""),
            CodecError::RecordLength(len) => write!(
                f,
                "length \"{len}\" must be zero or equal to or greater than {RECORD_META_MIN_LEN}"
            ),
        }
    }
}
