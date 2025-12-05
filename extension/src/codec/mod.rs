use std::io::{Read, Write};

use msrf::{error::IoError, io::SizedRecord};

use crate::{
    MSRF_EXT_VERSION, SourceAdd, SourceRemove,
    codec::v0::{Deserialiser, Serialiser},
    error::DesError,
};

mod v0;

// TODO: Accept a shared configuration type (Options)
pub trait RawSerialiser {
    fn write_source_add<W: Write>(&self, rec: &SourceAdd, wtr: W) -> Result<(), IoError<DesError>>;
    fn write_source_remove<W: Write>(
        &self,
        rec: &SourceRemove,
        wtr: W,
    ) -> Result<(), IoError<DesError>>;
}

// TODO: Accept a shared configuration type (Options)
pub trait RawDeserialiser {
    fn read_source_add<R: Read>(&self, rdr: R) -> Result<SourceAdd, IoError<DesError>>;
    fn read_source_remove<R: Read>(&self, rdr: R) -> Result<SourceRemove, IoError<DesError>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Version(u16);

impl Version {
    pub const fn new(version: u16) -> Option<Version> {
        if version > MSRF_EXT_VERSION {
            None
        } else {
            Some(Version(version))
        }
    }

    pub const fn get(&self) -> u16 {
        self.0
    }

    pub const fn current() -> Version {
        Version(MSRF_EXT_VERSION)
    }
}

#[derive(Debug)]
pub enum AnySerialiser {
    V0(v0::Serialiser),
}

impl RawSerialiser for AnySerialiser {
    fn write_source_add<W: Write>(&self, rec: &SourceAdd, wtr: W) -> Result<(), IoError<DesError>> {
        match self {
            AnySerialiser::V0(ser) => ser.write_source_add(rec, wtr),
        }
    }

    fn write_source_remove<W: Write>(
        &self,
        rec: &SourceRemove,
        wtr: W,
    ) -> Result<(), IoError<DesError>> {
        match self {
            AnySerialiser::V0(ser) => ser.write_source_remove(rec, wtr),
        }
    }
}

impl AnySerialiser {
    pub fn new(version: Version) -> AnySerialiser {
        assert_eq!(0, MSRF_EXT_VERSION);
        match version.get() {
            0 => AnySerialiser::V0(Serialiser),
            _ => unreachable!(),
        }
    }
}

impl SizedRecord<AnySerialiser> for SourceAdd {
    fn encoded_len(&self, ser: &AnySerialiser) -> usize {
        match ser {
            AnySerialiser::V0(ser) => self.encoded_len(ser),
        }
    }
}

impl SizedRecord<AnySerialiser> for SourceRemove {
    fn encoded_len(&self, ser: &AnySerialiser) -> usize {
        match ser {
            AnySerialiser::V0(ser) => self.encoded_len(ser),
        }
    }
}

#[derive(Debug)]
pub enum AnyDeserialiser {
    V0(v0::Deserialiser),
}

impl RawDeserialiser for AnyDeserialiser {
    fn read_source_add<R: Read>(&self, rdr: R) -> Result<SourceAdd, IoError<DesError>> {
        match self {
            AnyDeserialiser::V0(des) => des.read_source_add(rdr),
        }
    }

    fn read_source_remove<R: Read>(&self, rdr: R) -> Result<SourceRemove, IoError<DesError>> {
        match self {
            AnyDeserialiser::V0(des) => des.read_source_remove(rdr),
        }
    }
}

impl AnyDeserialiser {
    pub fn new(version: Version) -> AnyDeserialiser {
        assert_eq!(0, MSRF_EXT_VERSION);
        match version.get() {
            0 => AnyDeserialiser::V0(Deserialiser),
            _ => unreachable!(),
        }
    }
}
