pub mod v0_0;

use msrf_io::{TakeExt, error::CodecResult, varint};

use crate::{
    data::{Header, RecordMeta},
    reader::ParserError,
};

pub(crate) mod constants {
    pub(crate) const MAGIC_BYTES: [u8; 4] = *b"MSRF";
    pub(crate) const HEADER_LEN: usize = 8;
    pub(crate) const HEADER_CONTENTS: u64 = 3;
    pub(crate) const RECORD_META_MIN_LEN: u64 = 5;
    pub(crate) const RECORD_EOS: u64 = u64::MIN;
}

pub type DesResult<T> = Result<(T, usize), ParserError>;

pub trait RawDeserialiser {
    fn deserialise_header(&self, input: &[u8]) -> DesResult<Header> {
        default_deserialise_header(input)
    }
    fn deserialise_record_meta(&self, input: &[u8]) -> DesResult<RecordMeta>;
    fn deserialise_guard(&self, input: &[u8]) -> DesResult<()>;
}

pub fn default_deserialise_header(buf: &[u8]) -> DesResult<Header> {
    let len = buf.len();
    let mut buf = buf;

    if constants::HEADER_LEN - 1 > len {
        return Err(ParserError::Need(constants::HEADER_LEN - 1 - len));
    }

    // SAFETY: [u8; 4].len() == 4
    let magic_bytes = buf.take_chunk().unwrap();
    if magic_bytes != constants::MAGIC_BYTES {
        return Err(ParserError::MagicBytes(magic_bytes));
    }

    // TODO: Assert if byte exists?
    let length_len = varint::len(buf[0]);
    let length = varint::from_le_bytes(
        buf.take_slice(length_len)
            .ok_or_else(|| ParserError::Need(todo!()))?,
    );
    let major = u8::from_le_bytes(buf.take_chunk().ok_or_else(|| ParserError::Need(todo!()))?);
    let minor = u8::from_le_bytes(buf.take_chunk().ok_or_else(|| ParserError::Need(todo!()))?);
    Ok((
        Header {
            length,
            version: (major, minor),
        },
        len - buf.len(),
    ))
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum AnyDeserialiser {
    V0_0(v0_0::Deserialiser),
}

impl AnyDeserialiser {
    pub fn new() -> Self {
        Self::V0_0(v0_0::Deserialiser)
    }

    pub fn with_version(version: (u8, u8)) -> Option<Self> {
        match version {
            (0, 0) => Some(Self::V0_0(v0_0::Deserialiser)),
            _ => None,
        }
    }
}

impl RawDeserialiser for AnyDeserialiser {
    fn deserialise_header(&self, input: &[u8]) -> DesResult<Header> {
        match self {
            AnyDeserialiser::V0_0(des) => des.deserialise_header(input),
        }
    }

    fn deserialise_record_meta(&self, input: &[u8]) -> DesResult<RecordMeta> {
        match self {
            AnyDeserialiser::V0_0(des) => des.deserialise_record_meta(input),
        }
    }

    fn deserialise_guard(&self, input: &[u8]) -> DesResult<()> {
        match self {
            AnyDeserialiser::V0_0(des) => des.deserialise_guard(input),
        }
    }
}
