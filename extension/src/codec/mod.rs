use std::io::Read;

use msrf::error::IoError;

use crate::{SourceAdd, SourceRemove, error::DesError};

mod v0_0;

// TODO: Accept a shared configuration type (Options)
pub trait RawDeserialiser {
    fn read_source_add<R: Read>(&self, rdr: &mut R) -> Result<SourceAdd, IoError<DesError>>;
    fn read_source_remove<R: Read>(&self, rdr: &mut R) -> Result<SourceRemove, IoError<DesError>>;
}
