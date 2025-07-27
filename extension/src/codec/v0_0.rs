use msrf_io::{ByteStream, MutByteStream, RecordSerialise};

use crate::data::Record;
use crate::data::{ID_SOURCE_ADD, ID_SOURCE_REMOVE, SourceAdd, SourceRemove};
use crate::error::Error;
pub struct Serialiser;

impl RecordSerialise for Serialiser {
    type Err = Error;
    type Record = Record;

    fn deserialise_record(&self, id: u16, value: &[u8]) -> Result<Self::Record, Self::Err> {
        match id.try_into().map_err(|_| Error::UnexpectedType(id))? {
            ID_SOURCE_ADD => deserialise_source_add(value).map(Record::from),
            ID_SOURCE_REMOVE => deserialise_source_remove(value).map(Record::from),
            id => return Err(Error::UnexpectedType(id as u16)),
        }
        .map_err(|_| Error::InvalidValueLength)
    }

    fn serialise_record(
        &self,
        value: &mut [u8],
        record: &Self::Record,
    ) -> Result<usize, Self::Err> {
        match record {
            Record::SourceAdd(source_add) => serialise_source_add(value, source_add),
            Record::SourceRemove(source_remove) => serialise_source_remove(value, source_remove),
        }
        .map_err(|_| Error::InvalidValueLength)
    }
}

fn serialise_source_add(buf: &mut [u8], data: &SourceAdd) -> Result<usize, usize> {
    let len = buf.len();
    let mut buf = buf;

    buf.insert_u64(data.id)?;
    buf.insert_u8(data.version.0)?;
    buf.insert_u8(data.version.1)?;
    buf.insert(data.name.as_bytes())?;

    Ok(len - buf.len())
}

fn deserialise_source_add(buf: &[u8]) -> Result<SourceAdd, usize> {
    let mut buf = buf;

    let id = buf.extract_u64()?;
    let major = buf.extract_u8()?;
    let minor = buf.extract_u8()?;
    let name = buf.extract(buf.len())?;
    let name = str::from_utf8(name).unwrap().to_string();

    Ok(SourceAdd {
        id,
        version: (major, minor),
        name,
    })
}

fn serialise_source_remove(buf: &mut [u8], data: &SourceRemove) -> Result<usize, usize> {
    let len = buf.len();
    let mut buf = buf;
    buf.insert_u64(data.id)?;
    Ok(len - buf.len())
}

fn deserialise_source_remove(buf: &[u8]) -> Result<SourceRemove, usize> {
    let mut buf = buf;
    let id = buf.extract_u64()?;
    Ok(SourceRemove { id })
}

#[cfg(test)]
mod test {
    use super::*;

    fn serialise_record_harness(sample: Record, expected: &[u8]) {
        let serialiser = Serialiser;
        let record = serialiser
            .deserialise_record(sample.type_id(), expected)
            .expect("failed deserialise");

        assert_eq!(record, sample);

        let mut buf = vec![0; expected.len()];
        let written = serialiser
            .serialise_record(buf.as_mut_slice(), &record)
            .expect("failed serialise");

        assert_eq!(written, buf.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn serialise_source_add() {
        serialise_record_harness(
            Record::SourceAdd(SourceAdd {
                id: 21,
                version: (0, 1),
                name: String::from("test"),
            }),
            constcat::concat_bytes!(
                &21u64.to_le_bytes(),
                &0u8.to_le_bytes(),
                &1u8.to_le_bytes(),
                b"test".as_slice()
            ),
        );
    }

    #[test]
    fn serialise_source_remove() {
        serialise_record_harness(
            Record::SourceRemove(SourceRemove { id: 21 }),
            &21u64.to_le_bytes(),
        );
    }

    #[test]
    fn serialise_invalid_id() {
        const INVALID_ID: u16 = 0xFFFF;
        let serialiser = Serialiser;
        let res = serialiser.deserialise_record(INVALID_ID, &[]);
        assert_eq!(res, Err(Error::UnexpectedType(INVALID_ID)));
    }

    #[test]
    fn serialise_invalid_len() {
        fn harness(id: u16) {
            const INVALID_BYTES: [u8; 3] = [1, 2, 3];
            let serialiser = Serialiser;
            let res = serialiser.deserialise_record(id, &INVALID_BYTES);
            assert_eq!(res, Err(Error::InvalidValueLength));
        }

        harness(ID_SOURCE_ADD);
        harness(ID_SOURCE_REMOVE);
    }
}
