use std::io::Read;

use msrf::{error::IoError, io::RecordChunk};

use crate::{AssignedId, Record, SourceAdd, SourceRemove, codec::RawDeserialiser, error::DesError};

pub struct MsrfExtReader<D> {
    // TODO: Add configuration (Options)
    // options: MagicalOptions
    des: D,
}

impl<D: RawDeserialiser> MsrfExtReader<D> {
    pub fn read_record<R: Read>(
        &self,
        id: u16,
        rdr: &mut RecordChunk<R>,
    ) -> Result<Record, IoError<DesError>> {
        match id {
            SourceAdd::TYPE_ID => self.des.read_source_add(rdr).map(Record::from),
            SourceRemove::TYPE_ID => self.des.read_source_remove(rdr).map(Record::from),
            unknown_id => Err(IoError::Parser(DesError::UnexpectedType(unknown_id))),
        }
    }

    pub fn read_source_add<R: Read>(
        &self,
        rdr: &mut RecordChunk<R>,
    ) -> Result<SourceAdd, IoError<DesError>> {
        self.des.read_source_add(rdr)
    }

    pub fn read_source_remove<R: Read>(
        &self,
        rdr: &mut RecordChunk<R>,
    ) -> Result<SourceRemove, IoError<DesError>> {
        self.des.read_source_remove(rdr)
    }
}
