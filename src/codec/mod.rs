pub mod v0;
mod varint;

use std::io::{Read, Write};

use crate::{
    CURRENT_VERSION, Header, RecordMeta,
    codec::constants::{HEADER_LEN, MAGIC_BYTES},
    error::{IoError, ParserError},
};

pub(crate) mod constants {
    pub const MAGIC_BYTES: [u8; 4] = *b"MSRF";
    pub const HEADER_LEN: usize = 7;
}

// TODO: Add options
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesOptions;

// TODO: Add options
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SerOptions;

pub trait RawDeserialiser {
    fn read_meta(&self, rdr: impl Read) -> Result<RecordMeta, IoError<ParserError>>;
}

pub trait RawSerialiser {
    fn write_meta(&self, meta: RecordMeta, wtr: impl Write) -> Result<(), IoError<ParserError>>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownSerdes;

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

impl RawDeserialiser for AnyDeserialiser {
    fn read_meta(&self, rdr: impl Read) -> Result<RecordMeta, IoError<ParserError>> {
        match self {
            AnyDeserialiser::V0(des) => des.read_meta(rdr),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AnySerialiser {
    V0(v0::Serialiser),
}

impl AnySerialiser {
    pub fn new(version: u16, options: SerOptions) -> Option<Self> {
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
            Some(Self::new_impl(version, SerOptions))
        }
    }

    fn new_impl(version: u16, options: SerOptions) -> Self {
        match version {
            0 => Self::V0(options.into()),
            _ => todo!("version control"),
        }
    }
}

impl RawSerialiser for AnySerialiser {
    fn write_meta(&self, meta: RecordMeta, wtr: impl Write) -> Result<(), IoError<ParserError>> {
        match self {
            AnySerialiser::V0(ser) => ser.write_meta(meta, wtr),
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

pub fn write_header<W: Write>(mut wtr: W, header: Header) -> Result<(), IoError<ParserError>> {
    wtr.write_all(&MAGIC_BYTES)?;
    wtr.write_all(&header.version().to_le_bytes())?;
    wtr.write_all(&[0x00])?;
    Ok(())
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
