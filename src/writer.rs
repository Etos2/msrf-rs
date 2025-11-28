use std::{io::Write, marker::PhantomData};

use crate::{
    CURRENT_VERSION, Header, RecordMeta,
    codec::{self, AnySerialiser, RawSerialiser},
    error::{IoError, ParserError},
    io::RecordSink,
};

#[derive(Debug, Clone)]
pub struct MsrfWriterBuilder {
    version: u16,
}

impl Default for MsrfWriterBuilder {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
        }
    }
}

// TODO: Smart version handling (track statically if valid)
impl MsrfWriterBuilder {
    pub fn new() -> Self {
        MsrfWriterBuilder::default()
    }

    pub fn version(mut self, version: u16) -> Option<MsrfWriterBuilder> {
        if version > CURRENT_VERSION {
            return None;
        }
        self.version = version;
        Some(self)
    }

    // TODO: Error
    pub fn build<W: Write>(self, wtr: W) -> Result<MsrfWriter<AnySerialiser, W, HeaderUninit>, ()> {
        let ser = AnySerialiser::new_default(self.version).ok_or(())?;
        Ok(MsrfWriter::new(wtr, ser))
    }

    pub fn build_with<W: Write, S: RawSerialiser>(self, wtr: W, ser: S) -> MsrfWriter<S, W, HeaderUninit> {
        MsrfWriter::new(wtr, ser)
    }
}

// TODO: Remove typestate?
pub struct HeaderInit;
// TODO: Remove typestate?
pub struct HeaderUninit;

// TODO: Config
pub struct MsrfWriter<S, W, H> {
    is_finished: bool,
    wtr: W,
    ser: S,
    header_state: PhantomData<H>,
}

impl<S, W, H> MsrfWriter<S, W, H> {
    pub fn builder() -> MsrfWriterBuilder {
        MsrfWriterBuilder::new()
    }
}

impl<S: RawSerialiser, W: Write> MsrfWriter<S, W, HeaderUninit> {
    fn new(wtr: W, ser: S) -> MsrfWriter<S, W, HeaderUninit> {
        MsrfWriter {
            is_finished: false,
            wtr,
            ser,
            header_state: PhantomData,
        }
    }

    pub fn initialise(
        mut self,
    ) -> Result<MsrfWriter<S, W, HeaderInit>, IoError<ParserError>> {
        let header = Header::new(CURRENT_VERSION);
        codec::write_header(&mut self.wtr, header)?;
        Ok(MsrfWriter {
            is_finished: self.is_finished,
            wtr: self.wtr,
            ser: self.ser,
            header_state: PhantomData,
        })
    }
}

impl<S: RawSerialiser, W: Write> MsrfWriter<S, W, HeaderInit> {
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
