pub mod v0;

use std::{convert::Infallible, io::Read};

use crate::{
    CURRENT_VERSION,
    codec::constants::{HEADER_LEN, MAGIC_BYTES},
    data::{Header, RecordMeta},
    reader::{IoParserError, ParserError},
};

pub(crate) mod constants {
    pub const MAGIC_BYTES: [u8; 4] = *b"MSRF";
    pub const HEADER_LEN: usize = 7;
    pub const HEADER_CONTENTS: u64 = 3;
    pub const RECORD_META_MIN_LEN: u64 = 5;
    pub const RECORD_EOS: u64 = u64::MIN;
}

pub type DesResult<T> = Result<(T, usize), ParserError>;

// TODO: Add options
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesOptions;

pub trait RawDeserialiser {
    const VERSION: usize;
    fn read_record(&self, rdr: impl Read) -> Result<RecordMeta, IoParserError>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownDeserialiser;

#[derive(Debug, PartialEq, Eq)]
pub enum AnyDeserialiser {
    V0(v0::Deserialiser),
}

impl AnyDeserialiser {
    pub fn new(version: u16, options: DesOptions) -> Option<Self> {
        if version > CURRENT_VERSION {
            None
        } else {
            Some(Self::new_impl(version, options))
        }
    }

    pub fn new_default(version: u16) -> Option<Self> {
        if version > CURRENT_VERSION {
            None
        } else {
            Some(Self::new_impl(version, DesOptions::default()))
        }
    }

    fn new_impl(version: u16, options: DesOptions) -> Self {
        match version {
            0 => Self::V0(options.into()),
            _ => unreachable!(),
        }
    }
}

// TODO: Consts for byte indexes
pub fn read_header(input: &[u8; HEADER_LEN]) -> Result<Header, ParserError> {
    // SAFETY: input[4..6].len() == 2
    let magic_bytes = input[..4].try_into().unwrap();
    if magic_bytes != MAGIC_BYTES {
        return Err(ParserError::MagicBytes(magic_bytes));
    } else if input[6] != 0 {
        return Err(ParserError::Guard(input[6]));
    }

    // SAFETY: input[4..6].len() == 2
    let version = u16::from_le_bytes(input[4..6].try_into().unwrap());

    Ok(Header { version })
}
