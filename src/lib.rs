use std::io::Read;

use msrf_io::error::CodecResult;

use crate::{
    codec::{
        AnySerialiser, RawSerialiser,
        constants::{HEADER_LEN, RECORD_META_MIN_LEN},
    },
    data::{Header, RecordMeta},
};

pub mod codec;
pub mod data;
pub mod reader;

const VERSION_TABLE: &'static [(u8, u8)] = &[(0, 0)];
const CURRENT_VERSION: (u8, u8) = (0, 0);

// TODO: Assert minor value?
#[derive(Debug)]
pub struct SerialiserBuilder {
    major_version: u8,
}

impl SerialiserBuilder {
    pub fn with_version(mut self, major: u8) -> Self {
        self.major_version = major;
        self
    }

    pub fn build_rdr<R: Read>(self, rdr: R) -> Option<MsrfReader<R>> {
        Some(MsrfReader::new(self.build_raw()?, rdr))
    }

    pub fn build_raw(self) -> Option<Serialiser> {
        Some(Serialiser {
            raw: match self.major_version {
                0 => AnySerialiser::V0_0(codec::v0_0::Serialiser),
                _ => return None,
            },
        })
    }
}

impl Default for SerialiserBuilder {
    fn default() -> Self {
        Self {
            major_version: CURRENT_VERSION.0,
        }
    }
}

pub struct Serialiser {
    raw: AnySerialiser,
}

impl Serialiser {
    pub fn builder() -> SerialiserBuilder {
        SerialiserBuilder::default()
    }
}

impl RawSerialiser for Serialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        self.raw.serialise_header(buf, header)
    }

    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        self.raw.serialise_record_meta(buf, meta)
    }

    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)> {
        self.raw.deserialise_header(buf)
    }

    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)> {
        self.raw.deserialise_record_meta(buf)
    }
}

mod private {
    use crate::{HasHeader, NeedHeader};

    pub trait Sealed {}

    impl Sealed for NeedHeader {}
    impl Sealed for HasHeader {}
}

pub trait SerialiserState: private::Sealed {}

pub struct NeedHeader;

impl SerialiserState for NeedHeader {}

pub struct HasHeader(Header);

impl SerialiserState for HasHeader {}

pub struct MsrfReader<R: Read, State: SerialiserState = NeedHeader> {
    raw: Serialiser,
    state: State,
    rdr: R,
    buf: Vec<u8>,
}

impl<R: Read> MsrfReader<R, NeedHeader> {
    fn new(raw: Serialiser, rdr: R) -> Self {
        MsrfReader {
            raw,
            state: NeedHeader,
            rdr,
            buf: Vec::new(),
        }
    }

    pub fn builder() -> SerialiserBuilder {
        SerialiserBuilder::default()
    }

    pub fn read_header(mut self) -> Result<MsrfReader<R, HasHeader>, ()> {
        // TODO: Not use codec constants as they could change
        // TODO: no unwrap
        let header_slice = self.buf.get_mut(..HEADER_LEN as usize).unwrap();
        // TODO: no unwrap
        self.rdr.read_exact(header_slice).unwrap();
        // TODO: no unwrap
        let (header, _) = self.raw.deserialise_header(header_slice).unwrap();

        Ok(MsrfReader {
            raw: self.raw,
            state: HasHeader(header),
            rdr: self.rdr,
            buf: self.buf,
        })
    }
}

// TODO: Partial reading e.g. read_value_chunked(len, || {...})
// TODO: Iterator
// TODO: Switch to driven architecture? (returns bytes needed to read when calling read_value, etc)
impl<R: Read> MsrfReader<R, HasHeader> {
    pub fn header(&self) -> &Header {
        &self.state.0
    }

    pub fn read_meta(&mut self) -> Result<Option<RecordMeta>, ()> {
        // TODO: Not use codec constants as they could change
        // TODO: RecordMeta could be EoS (1 byte)
        // TODO: no unwrap
        let meta_slice = self.buf.get_mut(..RECORD_META_MIN_LEN as usize).unwrap();
        // TODO: no unwrap
        self.rdr.read_exact(meta_slice).unwrap();
        // TODO: no unwrap
        let (meta, _) = self.raw.deserialise_record_meta(meta_slice).unwrap();

        Ok(Some(meta))
    }

    pub fn read_value<F, T>(&mut self, meta: &RecordMeta, decoder: F) -> Result<T, ()>
    where
        F: Fn(&[u8]) -> Result<T, ()>,
    {
        let len = meta.value_len() as usize + 1;
        // TODO: no unwrap
        let value = self.read_value_slice(len).unwrap();
        decoder(value)
    }

    pub fn read_value_with_type<F, T>(&mut self, meta: &RecordMeta, decoder: F) -> Result<T, ()>
    where
        F: Fn(u16, &[u8]) -> Result<T, ()>,
    {
        let len = meta.value_len() as usize + 1;
        // TODO: no unwrap
        let value = self.read_value_slice(len).unwrap();
        decoder(meta.type_id(), value)
    }

    pub fn skip_record(&mut self, meta: &RecordMeta) -> Result<(), ()> {
        std::io::copy(
            &mut self.rdr.by_ref().take(meta.value_len()),
            &mut std::io::sink(),
        )
        .unwrap();
        let mut guard = [0];
        self.rdr.read_exact(&mut guard).unwrap();
        if guard[0] != 0 { Err(()) } else { Ok(()) }
    }

    fn read_value_slice(&mut self, len: usize) -> Result<&[u8], ()> {
        self.buf.resize(len, 0);
        self.rdr.read_exact(self.buf.as_mut_slice()).unwrap();
        let (val, guard) = self.buf.as_slice().split_at(len - 1);
        if guard[0] != 0 { Err(()) } else { Ok(val) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialiser_invalid_major() {
        let invalid_major = CURRENT_VERSION.0 + 1;
        let res = Serialiser::builder()
            .with_version(invalid_major)
            .build_raw();
        assert!(res.is_none());
    }

    #[test]
    fn serialiser_build() {
        for version in VERSION_TABLE {
            let major = version.0;
            let res = Serialiser::builder().with_version(major).build_raw();
            assert!(
                res.is_some(),
                "expected serialiser for version `{}.{}`",
                version.0,
                version.1
            );
        }
    }
}
