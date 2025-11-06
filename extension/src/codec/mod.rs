use std::io::{Read, Write};

use msrf::error::IoError;

use crate::{SourceAdd, SourceRemove, error::DesError};

mod v0;

// TODO: Accept a shared configuration type (Options)
pub(crate) trait RawSerialiser {
    fn write_source_add<W: Write>(&self, rec: SourceAdd, wtr: W) -> Result<(), IoError<DesError>>;
    fn write_source_remove<W: Write>(&self, rec: SourceRemove, wtr: W) -> Result<(), IoError<DesError>>;
}

// TODO: Accept a shared configuration type (Options)
pub(crate) trait RawDeserialiser {
    fn read_source_add<R: Read>(&self, rdr: R) -> Result<SourceAdd, IoError<DesError>>;
    fn read_source_remove<R: Read>(&self, rdr: R) -> Result<SourceRemove, IoError<DesError>>;
}