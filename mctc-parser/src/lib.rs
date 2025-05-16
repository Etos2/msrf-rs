#![feature(ascii_char)]

use std::{
    ascii, error::Error, io::{Read, Write}, ops::RangeInclusive
};

use data::RecordMeta;

pub mod data;
pub mod decoder;
pub mod encoder;
pub mod error;
#[cfg(feature = "io")]
pub mod io;
#[cfg(not(feature = "io"))]
mod io;
mod util;

const CURRENT_VERSION: u16 = 0;
const MAGIC_BYTES: [u8; 4] = *b"MCTC";
const CODEC_ID_EOS: u64 = u64::MAX;
const CODEC_NAME_BOUNDS: RangeInclusive<u64> = 4..=64;
const CODEC_ENTRY_LENGTH_BOUNDS: RangeInclusive<u64> = 6..=66;

// TODO: Impl options
pub struct Options {}

impl Default for Options {
    fn default() -> Self {
        Self {}
    }
}

pub trait RecordImpl {
    fn type_id(&self) -> u64;
    fn length(&self) -> usize;
}

// TODO: Isolate API from IO (remove impl Write)
pub trait WriteRecord<E: Error>: RecordImpl {
    fn write_into(&self, wtr: impl Write) -> Result<(), E>;
}

// TODO: Isolate API from IO (remove impl Read)
pub trait ReadRecord<E: Error>: RecordImpl {
    fn read_from(rdr: impl Read, meta: RecordMeta) -> Result<Self, E>
    where
        Self: Sized;
}

// TODO: Isolate API from IO (remove impl Read + Write)
// TODO: Remove `ascii::Char` from pub api
pub trait Codec {
    const NAME: &'static [ascii::Char];
    const VERSION: u16;
    type Err: Error;
    type Rec;

    fn type_id(&self, rec: &Self::Rec) -> u64;
    fn size(&self, rec: &Self::Rec) -> usize;
    fn write_value(&mut self, wtr: impl Write, rec: &Self::Rec) -> Result<(), Self::Err>;
    fn read_value(&mut self, rdr: impl Read, meta: RecordMeta) -> Result<Self::Rec, Self::Err>;
}
