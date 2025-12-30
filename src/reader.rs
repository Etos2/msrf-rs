use std::io::Read;

use crate::{
    CURRENT_VERSION, RecordId, RecordMeta,
    codec::{self, AnyDeserialiser, RawDeserialiser, UnknownSerdes, constants::HEADER_LEN},
    error::{IoError, ParserError},
    io::RecordChunk,
};

pub type DeserialiseResult<T> = Result<(T, usize), Result<usize, ParserError>>;

#[derive(Debug, Default, Clone)]
pub struct MsrfReaderBuilder {
    version: Option<u16>,
}

impl MsrfReaderBuilder {
    #[must_use]
    pub fn new() -> Self {
        MsrfReaderBuilder::default()
    }

    #[must_use]
    pub fn version(mut self, version: u16) -> MsrfReaderBuilder {
        self.version = Some(version);
        self
    }

    // TODO: Error
    pub fn build<R: Read>(self, wtr: R) -> Option<MsrfReader<AnyDeserialiser, R>> {
        let version = self.version.unwrap_or(CURRENT_VERSION);
        let des = AnyDeserialiser::new_default(version)?;
        Some(MsrfReader::new(wtr, des))
    }

    pub fn build_with_unknown<R: Read>(self, wtr: R) -> MsrfReader<UnknownSerdes, R> {
        MsrfReader::new_unknown(wtr)
    }

    pub fn build_with<R: Read, D: RawDeserialiser>(self, wtr: R, des: D) -> MsrfReader<D, R> {
        MsrfReader::new(wtr, des)
    }
}

// TODO: Builder
// TODO: Config
pub struct MsrfReader<D, R> {
    is_finished: bool,
    rdr: R,
    des: D,
    depth: Vec<(u16, RecordId)>,
}

impl<R: Read> MsrfReader<UnknownSerdes, R> {
    pub fn new_unknown(rdr: R) -> MsrfReader<UnknownSerdes, R> {
        MsrfReader {
            is_finished: false,
            rdr,
            des: UnknownSerdes,
            depth: Vec::new(),
        }
    }

    pub fn initialise(mut self) -> Result<MsrfReader<AnyDeserialiser, R>, IoError<ParserError>> {
        let mut buf = [0; HEADER_LEN];
        self.rdr.read_exact(&mut buf)?;

        let header = codec::read_header(&buf)?;
        let des = AnyDeserialiser::new_default(header.version)
            .ok_or(ParserError::Unsupported(header.version))?;

        Ok(MsrfReader {
            is_finished: false,
            rdr: self.rdr,
            des,
            depth: Vec::new(),
        })
    }
}

impl<D: RawDeserialiser, R: Read> MsrfReader<D, R> {
    pub fn new(rdr: R, des: D) -> MsrfReader<D, R> {
        MsrfReader {
            is_finished: false,
            rdr,
            des,
            depth: Vec::new(),
        }
    }

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

    // TODO: Return Err(ParserError::IsEos) on EoS byte rather than Some(None)?
    pub fn read_record(
        &mut self,
    ) -> Result<Option<(RecordId, RecordChunk<'_, R>)>, IoError<ParserError>> {
        let record = self.des.read_meta(&mut self.rdr)?;
        if record.is_eos() {
            return if self.is_finished {
                Err(IoError::Parser(ParserError::IsEos))
            } else {
                self.is_finished = true;
                Ok(None)
            };
        }

        self.update(&record);
        let ref_rdr = RecordChunk::new(&mut self.rdr, record.length);
        Ok(Some((record.into(), ref_rdr)))
    }

    pub fn current_parent(&self) -> Option<RecordId> {
        self.depth.last().map(|(_, id)| id).copied()
    }

    // Top down
    pub fn parents(&self) -> impl DoubleEndedIterator<Item = RecordId> {
        self.depth.iter().rev().map(|(_, id)| id).copied()
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read};

    use crate::{
        codec::{
            AnyDeserialiser,
            constants::MAGIC_BYTES,
            v0::{
                self,
                test::{REF_RECORD_META, REF_RECORD_META_BYTES},
            },
        },
        reader::MsrfReader,
    };

    const REF_HEADER_BYTES: &[u8; 7] = constcat::concat_bytes!(
        &MAGIC_BYTES,
        &[0x00, 0x00], // Version: u16(0)
        &[0x00]        // Guard: u8(0x00)
    );

    #[test]
    fn find_version() {
        let data = REF_HEADER_BYTES;
        let internal_rdr = Cursor::new(data);
        let reader = MsrfReader::new_unknown(internal_rdr);

        let reader = reader.initialise().expect("failed to find deserialiser");
        assert_eq!(reader.des, AnyDeserialiser::V0(v0::Deserialiser::default()))
    }

    #[test]
    fn read_record() {
        let user_data = [1, 2, 3, 4, 5, 6];
        let mut data = REF_RECORD_META_BYTES.to_vec();
        data.extend_from_slice(&user_data); // User data
        data.extend_from_slice(&[0]); // Guard

        let internal_rdr = Cursor::new(data);
        let mut reader = MsrfReader::new(internal_rdr, v0::Deserialiser::default());

        let res = reader.read_record().expect("failed to parse record");
        let (id, mut user_rdr) = res.expect("unexpected eos");
        assert_eq!(id, REF_RECORD_META.into());
        assert_eq!(user_rdr.len(), REF_RECORD_META.len());

        let mut user_buf = Vec::new();
        assert_eq!(
            user_rdr.len() as usize,
            user_rdr.read_to_end(&mut user_buf).expect("io fail")
        );
        assert_eq!(user_rdr.len(), 0);
        assert_eq!(user_buf.as_slice(), user_data);

        drop(user_rdr);

        let mut guard_buf = Vec::new();
        assert_eq!(1, reader.rdr.read_to_end(&mut guard_buf).expect("io fail"));
        assert_eq!(guard_buf.as_slice(), &[0]);
    }
}
