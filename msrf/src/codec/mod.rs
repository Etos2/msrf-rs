pub mod v0_0;

use msrf_io::error::CodecResult;

use crate::{
    codec::v0_0::Serialiser as SerialiserV0_0,
    data::{Header, RecordMeta},
};

pub(crate) mod constants {
    pub(crate) const MAGIC_BYTES: [u8; 4] = *b"MSRF";
    pub(crate) const HEADER_LEN: usize = 8;
    pub(crate) const HEADER_CONTENTS: u64 = 3;
    pub(crate) const RECORD_META_MIN_LEN: u64 = 5;
    pub(crate) const RECORD_EOS: u64 = u64::MIN;
}

pub(crate) trait RawSerialiser {
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
