use std::io::Write;

use crate::{
    RecordMeta,
    codec::RawSerialiser,
    error::{IoError, ParserError},
    io::RecordSink,
};

// TODO: Builder
// TODO: Config
pub struct MsrfWriter<S, W> {
    is_finished: bool,
    wtr: W,
    ser: S,
}

impl<S: RawSerialiser, W: Write> MsrfWriter<S, W> {
    pub fn write_record<'a>(
        &'a mut self,
        meta: RecordMeta,
    ) -> Result<RecordSink<'a, W>, IoError<ParserError>> {
        if self.is_finished {
            return Err(IoError::Parser(ParserError::IsEos));
        }

        // Guard Byte is written later (specifically when RecordSink is dropped)
        self.ser.write_meta(meta, &mut self.wtr)?;
        Ok(RecordSink::new(self.wtr.by_ref(), meta.length))
    }

    pub fn finish(&mut self) -> Result<(), IoError<ParserError>> {
        if self.is_finished {
            return Err(IoError::Parser(ParserError::IsEos));
        }

        self.is_finished = true;
        self.write_record(RecordMeta::new_eos())?;
        Ok(())
    }
}
