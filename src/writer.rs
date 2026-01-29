use std::{fmt::Debug, io::Write, marker::PhantomData};

use crate::{
    CURRENT_VERSION, Header, IntoMetadata, RecordId, RecordMeta,
    codec::{self, AnySerialiser, IntoData, RawSerialiser},
    error::{IoError, ParserError},
    io::SizedValue,
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
    #[must_use]
    pub fn new() -> Self {
        MsrfWriterBuilder::default()
    }

    #[must_use]
    pub fn version(mut self, version: u16) -> Option<MsrfWriterBuilder> {
        if version > CURRENT_VERSION {
            return None;
        }
        self.version = version;
        Some(self)
    }

    // TODO: Error
    pub fn build<W: Write>(self, wtr: W) -> Option<MsrfWriter<AnySerialiser, W, HeaderUninit>> {
        let ser = AnySerialiser::new_default(self.version)?;
        Some(MsrfWriter::new(wtr, ser))
    }

    pub fn build_with<W: Write, S: RawSerialiser>(
        self,
        wtr: W,
        ser: S,
    ) -> MsrfWriter<S, W, HeaderUninit> {
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
    depth: Vec<(u16, RecordId)>,
}

impl<S, W, H> MsrfWriter<S, W, H> {
    #[must_use]
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
            depth: Vec::new(),
        }
    }

    pub fn initialise(mut self) -> Result<MsrfWriter<S, W, HeaderInit>, IoError<ParserError>> {
        let header = Header::new(CURRENT_VERSION);
        codec::write_header(&mut self.wtr, &header)?;
        Ok(MsrfWriter {
            is_finished: self.is_finished,
            wtr: self.wtr,
            ser: self.ser,
            header_state: PhantomData,
            depth: Vec::new(),
        })
    }
}

impl<S: RawSerialiser, W: Write> MsrfWriter<S, W, HeaderInit> {
    fn update(&mut self, meta: &RecordMeta) {
        if let Some(count) = meta.contained()
            && count > 0
        {
            self.depth.push((count, (*meta).into()));
        } else {
            while let Some(cur_count) = self.depth.last_mut() {
                cur_count.0 -= 1;
                if cur_count.0 == 0 {
                    let _ = self.depth.pop();
                } else {
                    break;
                }
            }
        }
    }

    fn write_record_impl(
        &mut self,
        user_data: impl IntoData<S, W>,
        meta: RecordMeta,
    ) -> Result<(), IoError<ParserError>> {
        self.update(&meta);

        self.ser.write_meta(meta, &mut self.wtr)?;
        user_data.encode_into(&mut self.wtr, &self.ser, meta.source_id())?;
        self.wtr.write_all(&[0u8])?;

        Ok(())
    }

    pub fn write_record(
        &mut self,
        user_data: impl IntoData<S, W> + IntoMetadata<S>,
        source_id: u16,
    ) -> Result<(), IoError<ParserError>> {
        let meta = user_data.meta(&self.ser, source_id);

        if self.is_finished {
            return Err(IoError::Parser(ParserError::IsEos));
        } else if meta.is_eos() {
            // TODO: Better handling of EoS RecordMeta
            return Err(IoError::Parser(ParserError::UnexpectedEos));
        }

        self.write_record_impl(user_data, meta)
    }

    pub fn write_record_with(
        &mut self,
        user_data: impl IntoData<S, W> + SizedValue<S>,
        id: RecordId,
    ) -> Result<(), IoError<ParserError>> {
        if self.is_finished {
            return Err(IoError::Parser(ParserError::IsEos));
        }

        let meta = id.into_meta(user_data.encoded_len(&self.ser) as u64);
        self.write_record_impl(user_data, meta)
    }

    pub fn write_container(&mut self, user_data: impl IntoData<S, W> + IntoMetadata<S>, source_id: u16, length: u16) -> Result<(), IoError<ParserError>> {
        let mut meta = user_data.meta(&self.ser, source_id);
        meta.contained = Some(length);

        if self.is_finished {
            return Err(IoError::Parser(ParserError::IsEos));
        } else if meta.is_eos() {
            // TODO: Better handling of EoS RecordMeta
            return Err(IoError::Parser(ParserError::UnexpectedEos));
        }

        self.write_record_impl(user_data, meta)
    }

    // TODO: Call on drop()
    // TODO: Make impossible to be `self.is_finished` early (consume self)
    pub fn finish(&mut self) -> Result<(), IoError<ParserError>> {
        if self.is_finished {
            return Err(IoError::Parser(ParserError::IsEos));
        }

        self.ser.write_meta(RecordMeta::new_eos(), &mut self.wtr)?;
        Ok(())
    }

    pub fn current_parent(&self) -> Option<RecordId> {
        self.depth.last().map(|(_, id)| id).copied()
    }

    // Top down
    pub fn parents(&self) -> impl DoubleEndedIterator<Item = RecordId> {
        self.depth.iter().rev().map(|(_, id)| id).copied()
    }

    // pub fn write_record_2<Ser, V>(
    //     &mut self,
    //     source: &mut Ser,
    //     value: &V,
    // ) -> Result<(), IoError<ParserError>>
    // where
    //     Ser: Serialiser + AssignedId,
    //     V: Serialisable<Ser, W> + AssignedId + SizedValue,
    // {
    //     if source.id() == RECORD_EOS {
    //         return Err(IoError::Parser(ParserError::UnexpectedEos));
    //     }

    //     let len = value.encoded_len(&source) as u64;
    //     let meta = RecordMeta::new(source.id(), value.id(), len);

    //     self.ser.write_meta(meta, self.wtr)?;
    //     source.write_value(value)?;
    //     self.wtr.write_all(&[0u8])?;

    //     Ok(())
    // }
}

//
// let registrar = SourceRegistrar::new();
// let msrf_ext_id = registrar.register_root(MsrfExtWriter::name());
// let custom_id = registrar.register_root(CustomWriter::name());
//
// let msrf_wtr = MsrfWriter::new();
// let msrf_ext_wtr = MsrfExtWriter::new(msrf_ext_id);
// let custom_wtr = CustomWriter::new(custom_id);
//
// msrf_wtr.write_record(msrf_ext_wtr, record)?;
// msrf_wtr.write_record(msrf_ext_wtr, records[..])?;
// msrf_wtr.write_container(custom_wtr, record, records.iter().length())?;
// msrf_wtr.write_record_from_iter(custom_wtr, records.iter())?;
