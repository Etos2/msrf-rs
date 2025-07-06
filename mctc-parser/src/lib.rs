#![feature(ascii_char)]

pub mod data;
pub mod serialiser;
pub mod error;
#[cfg(feature = "io")]
pub mod io;
#[cfg(not(feature = "io"))]
mod io;

const CURRENT_VERSION: (u8, u8) = (0, 0);

// TODO: Impl options
// pub struct Options {}

// impl Default for Options {
//     fn default() -> Self {
//         Self {}
//     }
// }

// pub trait RecordImpl {
//     fn type_id(&self) -> u64;
//     fn length(&self) -> usize;
// }

// // TODO: Isolate API from IO (remove impl Write)
// pub trait WriteRecord<E: Error>: RecordImpl {
//     fn write_into(&self, wtr: impl Write) -> Result<(), E>;
// }

// // TODO: Isolate API from IO (remove impl Read)
// pub trait ReadRecord<E: Error>: RecordImpl {
//     fn read_from(rdr: impl Read, meta: RecordMeta) -> Result<Self, E>
//     where
//         Self: Sized;
// }

// // TODO: Isolate API from IO (remove impl Read + Write)
// // TODO: Remove `ascii::Char` from pub api
// pub trait Codec {
//     const NAME: &'static [ascii::Char];
//     const VERSION: u16;
//     type Err: Error;
//     type Rec;

//     fn type_id(&self, rec: &Self::Rec) -> u64;
//     fn size(&self, rec: &Self::Rec) -> usize;
//     fn write_value(&mut self, wtr: impl Write, rec: &Self::Rec) -> Result<(), Self::Err>;
//     fn read_value(&mut self, rdr: impl Read, meta: RecordMeta) -> Result<Self::Rec, Self::Err>;
// }
