use std::io::Write;

use msrf::{error::IoError, io::RecordSink};

use crate::{Record, SourceAdd, SourceRemove, codec::RawSerialiser, error::DesError};

pub struct MsrfExtWriter<S> {
    // TODO: Add configuration (Options)
    // options: MagicalOptions
    ser: S,
}

impl<S: RawSerialiser> MsrfExtWriter<S> {
    pub fn write_record<W: Write>(&self, wtr: &mut RecordSink<W>, record: Record) -> Result<(), IoError<DesError>> {
        match record {
            Record::SourceAdd(source_add) => self.ser.write_source_add(source_add, wtr),
            Record::SourceRemove(source_remove) => self.ser.write_source_remove(source_remove, wtr),
        }
    }

    pub fn write_source_add<W: Write>(&self, wtr: &mut RecordSink<W>, val: SourceAdd) -> Result<(), IoError<DesError>> {
        self.ser.write_source_add(val, wtr)
    }

    pub fn write_source_remove<W: Write>(&self, wtr: &mut RecordSink<W>, val: SourceRemove) -> Result<(), IoError<DesError>> {
        self.ser.write_source_remove(val, wtr)
    }
}
