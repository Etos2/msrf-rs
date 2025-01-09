use std::{io::Write, ops::RangeInclusive};

pub mod data;
pub mod error;
pub mod reader;
pub(crate) mod util;
pub mod writer;

const CURRENT_VERSION: u16 = 0;
const MAGIC_BYTES: [u8; 4] = *b"MCTC";
const CODEC_ID_EOS: u64 = u64::MAX;
const CODEC_NAME_BOUNDS: RangeInclusive<u64> = 4..=64;
const CODEC_ENTRY_LENGTH_BOUNDS: RangeInclusive<u64> = 6..=66;

// TODO: Impl options
pub struct DefaultOptions {}

impl Default for DefaultOptions {
    fn default() -> Self {
        Self {}
    }
}

pub trait WriteRecord<E: From<std::io::Error>> {
    fn type_id(&self) -> u64;
    fn length(&self) -> usize;
    fn write(&self, wtr: impl Write) -> Result<(), E>;
}

pub trait FromRecord {}

pub trait Codec {
    const NAME: &'static str;
}
