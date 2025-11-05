use std::io::Read;

use msrf::error::IoError;

use crate::{SourceAdd, SourceRemove, codec::RawDeserialiser, error::DesError};

pub struct Deserialiser;

const SOURCE_ADD_MIN: usize = 4;
const SOURCE_REMOVE_MIN: usize = 2;

impl RawDeserialiser for Deserialiser {
    fn read_source_add<R: Read>(&self, rdr: &mut R) -> Result<SourceAdd, IoError<DesError>> {
        let mut buf = [0; SOURCE_ADD_MIN];
        rdr.read_exact(&mut buf)?;
        let mut name_buf = String::new();
        rdr.read_to_string(&mut name_buf)?;

        Ok(SourceAdd {
            id: u16::from_le_bytes(buf[..2].try_into().unwrap()), // SAFETY: buf[..2].len() == size_of<u16>()
            version: u16::from_le_bytes(buf[2..].try_into().unwrap()), // SAFETY: buf[2..].len() == size_of<u16>()
            name: name_buf,
        })
    }

    fn read_source_remove<R: Read>(&self, rdr: &mut R) -> Result<SourceRemove, IoError<DesError>> {
        let mut buf = [0; SOURCE_REMOVE_MIN];
        rdr.read_exact(&mut buf)?;

        Ok(SourceRemove {
            id: u16::from_le_bytes(buf),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn des_source_add() {
        const REF_SOURCE_ADD_BYTES: &[u8; 14] = constcat::concat_bytes!(
            &u16::to_le_bytes(32),
            &u16::to_le_bytes(1),
            b"pxls.space".as_slice(),
        );

        let mut rdr = Cursor::new(REF_SOURCE_ADD_BYTES).take(14);
        let des = Deserialiser;

        assert_eq!(
            des.read_source_add(&mut rdr).expect("failed des"),
            SourceAdd {
                id: 32,
                version: 1,
                name: String::from("pxls.space"),
            }
        );

        let mut dump = [0; 1];
        assert_eq!(rdr.into_inner().read(&mut dump).expect("failed IO"), 0)
    }

    #[test]
    fn des_source_remove() {
        const REF_SOURCE_REMOVE_BYTES: &[u8; 2] = &u16::to_le_bytes(32);

        let mut rdr = Cursor::new(REF_SOURCE_REMOVE_BYTES).take(2);
        let des = Deserialiser;

        assert_eq!(
            des.read_source_remove(&mut rdr).expect("failed des"),
            SourceRemove { id: 32 }
        );

        let mut dump = [0; 1];
        assert_eq!(rdr.into_inner().read(&mut dump).expect("failed IO"), 0)
    }
}
