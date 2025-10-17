use std::io::{self, Read};

use crate::{
    codec::{self, constants::HEADER_LEN, AnyDeserialiser, RawDeserialiser}, error::{IoError, ParserError}, RecordId
};

pub type DeserialiseResult<T> = Result<(T, usize), Result<usize, ParserError>>;

pub struct Unknown;

pub struct MsrfReader<D, R> {
    rdr: R,
    des: D,
}

impl<R: Read> MsrfReader<Unknown, R> {
    pub fn init(mut self) -> Result<MsrfReader<AnyDeserialiser, R>, IoError<ParserError>> {
        let mut buf = [0; HEADER_LEN];
        self.rdr.read_exact(&mut buf)?;

        let header = codec::read_header(&buf)?;
        let des = AnyDeserialiser::new_default(header.version)
            .ok_or(ParserError::Unsupported(header.version))?;

        Ok(MsrfReader { rdr: self.rdr, des })
    }
}

impl<D: RawDeserialiser, R: Read> MsrfReader<D, R> {
    pub fn read_record<'a>(&'a mut self) -> Result<(RecordId, RecordChunk<'a, R>), IoError<ParserError>> {
        let record = self.des.read_record(&mut self.rdr)?;
        let ref_rdr = RecordChunk::new(&mut self.rdr, record.length);
        Ok((record.into_ids(), ref_rdr))
    }
}

pub struct RecordChunk<'a, R: Read>(io::Take<&'a mut R>);

impl<'a, R: Read> RecordChunk<'a, R> {
    fn new(rdr: &'a mut R, limit: u64) -> Self {
        Self(rdr.take(limit))
    }

    pub fn len(&self) -> u64 {
        self.0.limit()
    }

    pub fn is_empty(&self) -> bool {
        self.0.limit() == 0
    }
}

impl<'a, R: Read> Read for RecordChunk<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<'a, R: Read> Drop for RecordChunk<'a, R> {
    fn drop(&mut self) {
        if self.0.limit() > 0 {
            // BufWriter<W> drop impl also performs IO (flushing) on drop, we shall pretend this is normal
            let _res = io::copy(&mut self.0, &mut io::sink());
        }
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
        reader::{MsrfReader, Unknown},
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
        let reader = MsrfReader {
            rdr: internal_rdr,
            des: Unknown,
        };

        let reader = reader.init().expect("failed to find deserialiser");
        assert_eq!(reader.des, AnyDeserialiser::V0(v0::Deserialiser::default()))
    }

    #[test]
    fn read_record() {
        let user_data = [1, 2, 3, 4, 5, 6];
        let mut data = REF_RECORD_META_BYTES.to_vec();
        data.extend_from_slice(&user_data); // User data
        data.extend_from_slice(&[0]); // Guard

        let internal_rdr = Cursor::new(data);
        let mut reader = MsrfReader {
            rdr: internal_rdr,
            des: v0::Deserialiser::default(),
        };

        let (meta, mut user_rdr) = reader.read_record().expect("failed to parse record");
        assert_eq!(meta, REF_RECORD_META.into_ids());
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
