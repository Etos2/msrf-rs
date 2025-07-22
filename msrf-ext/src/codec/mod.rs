mod v0_0;

use msrf_io::error::CodecResult;

use crate::data::Record;

// TODO: Use different error type
pub trait RawSerialiser {
    fn serialise(&self, buf: &mut [u8], record: &Record) -> CodecResult<usize>;
    fn deserialise(&self, buf: &[u8]) -> CodecResult<(Record, usize)>;
}