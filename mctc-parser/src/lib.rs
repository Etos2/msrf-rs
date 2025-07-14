use crate::{
    codec::{v0_0::Deserialiser, AnySerialiser, RawSerialiser},
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

impl Serialiser {
    pub fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        self.raw.serialise_header(buf, header)
    }

    pub fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        self.raw.serialise_record_meta(buf, meta)
    }
}