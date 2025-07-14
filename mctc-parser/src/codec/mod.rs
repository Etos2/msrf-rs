mod util;
pub mod v0_0;

use crate::{
    codec::v0_0::Deserialiser as DeserialiserV0_0,
    codec::v0_0::Serialiser as SerialiserV0_0,
    data::{Header, RecordMeta},
    error::CodecResult,
};

pub(crate) mod constants {
    pub(crate) const MAGIC_BYTES: [u8; 4] = *b"MCTC";
    pub(crate) const HEADER_LEN: usize = 8;
    pub(crate) const HEADER_CONTENTS: u64 = 3;
    pub(crate) const RECORD_META_MIN_LEN: u64 = 5;
    pub(crate) const RECORD_EOS: u64 = u64::MIN;
}

pub(crate) trait RawSerialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize>;
    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize>;
}

pub(crate) trait RawDeserialiser {
    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)>;
    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)>;
}

#[non_exhaustive]
pub enum AnySerialiser {
    V0_0(SerialiserV0_0),
}

#[non_exhaustive]
pub enum AnyDeserialiser {
    V0_0(DeserialiserV0_0),
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
}

impl RawDeserialiser for AnyDeserialiser {
    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)> {
        match self {
            AnyDeserialiser::V0_0(raw) => raw.deserialise_header(buf),
        }
    }

    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)> {
        match self {
            AnyDeserialiser::V0_0(raw) => raw.deserialise_record_meta(buf),
        }
    }
}
