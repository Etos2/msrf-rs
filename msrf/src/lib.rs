use msrf_io::error::CodecResult;

use crate::{
    codec::{AnySerialiser, RawSerialiser},
    data::{Header, RecordMeta},
};

pub mod codec;
pub mod data;

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

    pub fn build(self) -> Option<Serialiser> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialiser_invalid_major() {
        let invalid_major = CURRENT_VERSION.0 + 1;
        let res = Serialiser::builder().with_version(invalid_major).build();
        assert!(res.is_none());
    }

    #[test]
    fn serialiser_build() {
        for version in VERSION_TABLE {
            let major = version.0;
            let res = Serialiser::builder().with_version(major).build();
            assert!(res.is_some(), "expected serialiser for version `{}.{}`", version.0, version.1);
        }
    }
}
