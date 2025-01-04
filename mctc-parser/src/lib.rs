use std::ops::RangeInclusive;

pub mod data;
pub mod error;
pub mod reader;
pub mod writer;

const MAGIC_BYTES: [u8; 4] = *b"MCTC";
const CODEC_ID_EOS: u64 = u64::MAX;
const CODEC_NAME_BOUNDS: RangeInclusive<u64> = 4..=64;

// TODO: Impl options
pub struct DefaultOptions {}

impl Default for DefaultOptions {
    fn default() -> Self {
        Self {}
    }
}
