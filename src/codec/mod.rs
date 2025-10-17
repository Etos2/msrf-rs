pub mod v0;
mod varint;

use std::io::Read;

use crate::{
    CURRENT_VERSION, Header, RecordMeta,
    codec::constants::{HEADER_LEN, MAGIC_BYTES},
    error::{IoError, ParserError},
};

pub(crate) mod constants {
    pub const MAGIC_BYTES: [u8; 4] = *b"MSRF";
    pub const HEADER_LEN: usize = 7;
    pub const RECORD_EOS: u16 = u16::MAX;
}

// TODO: Add options
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesOptions;

pub trait RawDeserialiser {
    const VERSION: usize;
    fn read_record(&self, rdr: impl Read) -> Result<RecordMeta, IoError<ParserError>>;
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
            Some(Self::new_impl(version, DesOptions))
        }
    }

    fn new_impl(version: u16, options: DesOptions) -> Self {
        match version {
            0 => Self::V0(options.into()),
            _ => unreachable!(),
        }
    }
}

// TODO: Reader impl
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Header, codec::constants::MAGIC_BYTES};

    pub(crate) const REF_HEADER: Header = Header { version: 3 };

    pub(crate) const REF_HEADER_BYTES: &[u8; HEADER_LEN] = constcat::concat_bytes!(
        &MAGIC_BYTES,  // Magic bytes
        &[3_u8, 0_u8], // Version
        &[0x00]        // Guard
    );

    #[test]
    fn des_header() {
        let header = read_header(REF_HEADER_BYTES).expect("failed parse");
        assert_eq!(header, REF_HEADER);
    }

    #[test]
    fn des_header_invalid_magic() {
        let mut invalid_bytes = REF_HEADER_BYTES.clone();
        let invalid_magic = b"BAD!";
        invalid_bytes[..4].copy_from_slice(invalid_magic);

        let header = read_header(&invalid_bytes).expect_err("succeeded parse");
        assert_eq!(header, ParserError::MagicBytes(*invalid_magic));
    }

    #[test]
    fn des_header_invalid_guard() {
        let mut invalid_bytes = REF_HEADER_BYTES.clone();
        let invalid_guard = 42;
        invalid_bytes[6] = invalid_guard;

        let header = read_header(&invalid_bytes).expect_err("succeeded parse");
        assert_eq!(header, ParserError::Guard(invalid_guard));
    }
}
