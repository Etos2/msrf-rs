#![feature(ascii_char)]

use std::{io::Write, ops::RangeTo};

use crate::{
    data::{Header, RecordMeta},
    error::CodecResult,
    io::Serialisable,
};

pub mod data;
pub mod error;
#[cfg(feature = "io")]
pub mod io;
#[cfg(not(feature = "io"))]
mod io;
pub mod serialiser;

const CURRENT_VERSION: (u8, u8) = (0, 0);

trait RawSerialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize>;
    fn serialise_record_meta(&self, buf: &mut [u8], header: &RecordMeta) -> CodecResult<usize>;
}

pub struct Serialiser {
    raw: Box<dyn RawSerialiser>
}

impl Serialiser {
    pub fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        self.raw.serialise_header(buf, header)
    }

    pub fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        self.raw.serialise_record_meta(buf, meta)
    }

    pub fn serialise_record_eos(&self, buf: &mut [u8]) -> CodecResult<usize> {
        0u8.encode_into(buf)
    }
}