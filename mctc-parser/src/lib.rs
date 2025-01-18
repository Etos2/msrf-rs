use std::{error::Error, io::{Read, Write}, ops::RangeInclusive};

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

pub trait Record {
    fn type_id(&self) -> u64;
    fn length(&self) -> usize;
}

pub trait WriteRecord<E: Error>: Record {
    fn write_into(&self, wtr: impl Write) -> Result<(), E>;
}

pub trait ReadRecord<E: Error>: Record {
    fn read_from(rdr: impl Read) -> Result<Self, E> where Self: Sized;
}

pub trait Codec {
    const NAME: &'static str;
    type Err: Error;
    type Rec: Record + WriteRecord<Self::Err> + ReadRecord<Self::Err>;

    fn codec_id(&self) -> u64;
    fn write_record(&mut self, wtr: impl Write, rec: &Self::Rec) -> Result<(), Self::Err>;
    fn read_record(&mut self, rdr: impl Read) -> Result<Self::Rec, Self::Err>;
}
