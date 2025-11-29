use std::io::Write;

use msrf::error::IoError;

use crate::{
    Record, SourceAdd, SourceRemove,
    codec::{AnySerialiser, RawSerialiser, Version},
    error::DesError,
};

#[derive(Debug, Clone)]
pub struct MsrfExtWriterBuilder {
    version: Version,
}

impl Default for MsrfExtWriterBuilder {
    fn default() -> Self {
        Self {
            version: Version::current(),
        }
    }
}

// TODO: Impl version control
impl MsrfExtWriterBuilder {
    pub fn build(self) -> MsrfExtWriter<AnySerialiser> {
        MsrfExtWriter::new(AnySerialiser::new(self.version))
    }

    pub fn build_with<S: RawSerialiser>(self, ser: S) -> MsrfExtWriter<S> {
        MsrfExtWriter::new(ser)
    }
}

pub struct MsrfExtWriter<S> {
    // TODO: Add configuration (Options)
    // options: MagicalOptions
    ser: S,
}

impl<S: RawSerialiser> MsrfExtWriter<S> {
    fn new(ser: S) -> MsrfExtWriter<S> {
        MsrfExtWriter { ser }
    }

    pub fn write_record<W: Write>(
        &self,
        wtr: &mut W,
        record: Record,
    ) -> Result<(), IoError<DesError>> {
        match record {
            Record::SourceAdd(source_add) => self.ser.write_source_add(source_add, wtr),
            Record::SourceRemove(source_remove) => self.ser.write_source_remove(source_remove, wtr),
        }
    }

    pub fn write_source_add<W: Write>(
        &self,
        wtr: &mut W,
        val: SourceAdd,
    ) -> Result<(), IoError<DesError>> {
        self.ser.write_source_add(val, wtr)
    }

    pub fn write_source_remove<W: Write>(
        &self,
        wtr: &mut W,
        val: SourceRemove,
    ) -> Result<(), IoError<DesError>> {
        self.ser.write_source_remove(val, wtr)
    }
}
