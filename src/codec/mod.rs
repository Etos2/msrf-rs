pub mod v0_0;

use msrf_io::{ByteStream, error::CodecResult};

use crate::{
    codec::v0_0::Serialiser as SerialiserV0_0,
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

pub trait RawSerialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize>;
    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize>;
    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)>;
    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)>;
}

#[non_exhaustive]
pub enum AnySerialiser {
    V0_0(SerialiserV0_0),
}

impl RawSerialiser for AnySerialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        match self {
            AnySerialiser::V0_0(raw) => raw.serialise_header(buf, header),
        }
    }

    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        match self {
            AnySerialiser::V0_0(raw) => raw.serialise_record_meta(buf, meta),
        }
    }

    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)> {
        match self {
            AnySerialiser::V0_0(raw) => raw.deserialise_header(buf),
        }
    }

    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)> {
        match self {
            AnySerialiser::V0_0(raw) => raw.deserialise_record_meta(buf),
        }
    }
}

pub fn default_deserialise_header(buf: &[u8]) -> DesResult<Header> {
    let len = buf.len();
    let mut buf = buf;

    if constants::HEADER_LEN - 1 > len {
        return Err(ParserError::Need(constants::HEADER_LEN - 1 - len));
    }

    // SAFETY: [u8; 4].len() == 4
    let magic_bytes = buf
        .extract(4)
        .map_err(ParserError::Need)?
        .try_into()
        .unwrap();
    if magic_bytes != constants::MAGIC_BYTES {
        return Err(ParserError::MagicBytes(magic_bytes));
    }

    let length = buf.extract_varint().map_err(ParserError::Need)?;
    let major = buf.extract_u8().map_err(ParserError::Need)?;
    let minor = buf.extract_u8().map_err(ParserError::Need)?;

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
