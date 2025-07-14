use crate::{
    codec::{AnySerialiser, RawSerialiser},
    data::{Header, RecordMeta},
    error::CodecResult,
};

pub(crate) mod codec;
pub mod data;
pub mod error;

const CURRENT_VERSION: (u8, u8) = (0, 0);

pub struct Serialiser {
    raw: AnySerialiser,
}

impl RawSerialiser for Serialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        self.raw.serialise_header(buf, header)
    }

    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        self.raw.serialise_record_meta(buf, meta)
    }

    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)> {
        self.raw.deserialise_header(buf)
    }

    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)> {
        self.raw.deserialise_record_meta(buf)
    }
}
