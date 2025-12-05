use std::io::Read;

use msrf::{error::IoError, io::SizedRecord};

use crate::{
    SourceAdd, SourceRemove,
    codec::{RawDeserialiser, RawSerialiser},
    error::DesError,
};

const SOURCE_ADD_LEN: usize = 4;
const SOURCE_REMOVE_LEN: usize = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Serialiser;

impl RawSerialiser for Serialiser {
    fn write_source_add<W: std::io::Write>(
        &self,
        rec: &SourceAdd,
        mut wtr: W,
    ) -> Result<(), IoError<DesError>> {
        wtr.write_all(&rec.id.to_le_bytes())?;
        wtr.write_all(&rec.version.to_le_bytes())?;
        wtr.write_all(rec.name.as_bytes())?;

        Ok(())
    }

    fn write_source_remove<W: std::io::Write>(
        &self,
        rec: &SourceRemove,
        mut wtr: W,
    ) -> Result<(), IoError<DesError>> {
        wtr.write_all(&rec.id.to_le_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Deserialiser;

impl RawDeserialiser for Deserialiser {
    fn read_source_add<R: Read>(&self, mut rdr: R) -> Result<SourceAdd, IoError<DesError>> {
        let mut buf = [0; SOURCE_ADD_LEN];
        rdr.read_exact(&mut buf)?;
        let mut name_buf = String::new();
        rdr.read_to_string(&mut name_buf)?;

        Ok(SourceAdd {
            id: u16::from_le_bytes(*buf[..2].first_chunk().unwrap()), // SAFETY: [u8; 2] == size_of<u16>()
            version: u16::from_le_bytes(*buf[2..4].first_chunk().unwrap()), // SAFETY: [u8; 2] == size_of<u16>()
            name: name_buf,
        })
    }

    fn read_source_remove<R: Read>(&self, mut rdr: R) -> Result<SourceRemove, IoError<DesError>> {
        let mut buf = [0; SOURCE_REMOVE_LEN];
        rdr.read_exact(&mut buf)?;

        Ok(SourceRemove {
            id: u16::from_le_bytes(buf),
        })
    }
}

impl SizedRecord<Serialiser> for SourceAdd {
    fn encoded_len(&self, _ser: &Serialiser) -> usize {
        // ID: u16 + Version: u16 + Name: Variable
        size_of::<u16>() * 2 + self.name.len()
    }
}

impl SizedRecord<Serialiser> for SourceRemove{
    fn encoded_len(&self, _ser: &Serialiser) -> usize {
        // ID: u16
        size_of::<u16>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn serdes_source_add() {
        const REF_SOURCE_ADD_BYTES: &[u8; 14] = constcat::concat_bytes!(
            &u16::to_le_bytes(32),
            &u16::to_le_bytes(1),
            b"pxls.space".as_slice(),
        );
        let ref_source_add = SourceAdd {
            id: 32,
            version: 1,
            name: String::from("pxls.space"),
        };

        let ser = Serialiser;
        let des = Deserialiser;

        let mut buf = [0u8; 14];
        ser.write_source_add(&ref_source_add.clone(), buf.as_mut_slice())
            .expect("failed ser");
        assert_eq!(&buf, REF_SOURCE_ADD_BYTES);

        let mut rdr = Cursor::new(buf).take(14);
        let rec = des.read_source_add(&mut rdr).expect("failed des");
        assert_eq!(rec, ref_source_add);
        assert_eq!(rdr.into_inner().position(), 14);
    }

    #[test]
    fn serdes_source_remove() {
        const REF_SOURCE_REMOVE_BYTES: &[u8; 2] = &u16::to_le_bytes(32);
        const REF_SOURCE_REMOVE: SourceRemove = SourceRemove { id: 32 };

        let ser = Serialiser;
        let des = Deserialiser;

        let mut buf = [0u8; 2];
        ser.write_source_remove(&REF_SOURCE_REMOVE, buf.as_mut_slice())
            .expect("failed ser");
        assert_eq!(&buf, REF_SOURCE_REMOVE_BYTES);

        let mut rdr = Cursor::new(REF_SOURCE_REMOVE_BYTES).take(2);
        let rec = des.read_source_remove(&mut rdr).expect("failed des");
        assert_eq!(rec, REF_SOURCE_REMOVE);
        assert_eq!(rdr.into_inner().position(), 2);
    }
}
