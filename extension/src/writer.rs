use std::io::Write;

use msrf::{IntoMetadata, RecordMeta, error::IoError, io::SizedRecord};

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
    pub fn record_len<T: SizedRecord<S>>(&self, record: &T) -> usize {
        record.encoded_len(&self.ser)
    }
}

impl<S: RawSerialiser> MsrfExtWriter<S> {
    fn new(ser: S) -> MsrfExtWriter<S> {
        MsrfExtWriter { ser }
    }

    pub fn write_record<W: Write>(
        &self,
        wtr: &mut W,
        record: &Record,
    ) -> Result<(), IoError<DesError>> {
        match record {
            Record::SourceAdd(source_add) => self.ser.write_source_add(source_add, wtr),
            Record::SourceRemove(source_remove) => self.ser.write_source_remove(source_remove, wtr),
        }
    }

    pub fn write_source_add<W: Write>(
        &self,
        wtr: &mut W,
        val: &SourceAdd,
    ) -> Result<(), IoError<DesError>> {
        self.ser.write_source_add(val, wtr)
    }

    pub fn write_source_remove<W: Write>(
        &self,
        wtr: &mut W,
        val: &SourceRemove,
    ) -> Result<(), IoError<DesError>> {
        self.ser.write_source_remove(val, wtr)
    }
}

impl<S: RawSerialiser> MsrfExtWriter<S>
where
    SourceAdd: IntoMetadata<S>,
    SourceRemove: IntoMetadata<S>,
{
    // TODO: Requires .clone() to use in practice, remove
    pub fn generate_meta(&self, source_id: u16, record: impl Into<Record>) -> RecordMeta {
        match record.into() {
            Record::SourceAdd(source_add) => source_add.meta(&self.ser, source_id),
            Record::SourceRemove(source_remove) => source_remove.meta(&self.ser, source_id),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use constcat::concat_bytes;

    use super::*;

    #[test]
    fn write_source_add() {
        const SOURCE_ADD_BYTES: &[u8; 12] = concat_bytes!(
            &u16::to_le_bytes(1),
            &u16::to_le_bytes(2),
            b"mrsf-ext".as_slice()
        );

        let dest = [0; 12];
        let mut cursor = Cursor::new(dest);
        let ser = MsrfExtWriterBuilder::default().build();
        let record = SourceAdd::new(1, 2, "mrsf-ext");
        ser.write_source_add(&mut cursor, &record).unwrap();
        assert_eq!(ser.record_len(&record), SOURCE_ADD_BYTES.len());
        assert_eq!(&cursor.into_inner(), SOURCE_ADD_BYTES);
    }

    #[test]
    fn write_source_remove() {
        const SOURCE_REMOVE_BYTES: &[u8; 2] = &u16::to_le_bytes(1);

        let dest = [0; 2];
        let mut cursor = Cursor::new(dest);
        let ser = MsrfExtWriterBuilder::default().build();
        let record = SourceRemove::new(1);
        ser.write_source_remove(&mut cursor, &record).unwrap();
        assert_eq!(ser.record_len(&record), SOURCE_REMOVE_BYTES.len());
        assert_eq!(&cursor.into_inner(), SOURCE_REMOVE_BYTES);
    }
}
