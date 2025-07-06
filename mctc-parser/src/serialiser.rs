use crate::{
    data::{Header, RecordMeta},
    error::CodecError,
    io::{
        util::{infallible, Guard, PVarint},
        DecodeResult, EncodeResult, Serialisable, SerialiseExt,
    },
};

pub(crate) const MAGIC_BYTES: [u8; 4] = *b"MCTC";
pub(crate) const HEADER_LEN: usize = 8;
pub(crate) const HEADER_CONTENTS: usize = 3;
pub(crate) const RECORD_META_LEN: usize = 4;
pub(crate) const RECORD_EOS: u64 = u64::MIN;

// TODO: Move into lib.rs? (make decode.rs internal decoding logic)
#[derive(Debug)]
pub struct Decoder {}

impl Serialisable<'_> for Header {
    type Err = CodecError;

    fn encode_into(&self, buf: &mut [u8]) -> EncodeResult<Self::Err> {
        let mut buf = buf;
        if HEADER_LEN > buf.len() {
            return Err(Ok(HEADER_LEN - buf.len()));
        }

        // TODO: Correct length (assumes 1 byte despite variable header size)
        let content_len = (HEADER_LEN - HEADER_CONTENTS) as u64; // Exclude MagicBytes & Length
        buf.encode::<[u8; 4]>(MAGIC_BYTES).map_err(infallible)?;
        buf.encode(PVarint::from(content_len)).map_err(infallible)?;
        buf.encode(self.version.0).map_err(infallible)?;
        buf.encode(self.version.1).map_err(infallible)?;
        buf.encode(Guard::from(content_len)).map_err(infallible)?;

        Ok(buf.len())
    }

    fn decode_from(buf: &[u8]) -> DecodeResult<Self, Self::Err> {
        let mut buf = buf;
        if HEADER_LEN > buf.len() {
            return Err(Ok(HEADER_LEN - buf.len()));
        }

        let magic_bytes = buf.decode::<[u8; 4]>().map_err(infallible)?;
        if magic_bytes != MAGIC_BYTES {
            return Err(Err(CodecError::Badness));
        }

        let length = buf.decode::<PVarint>().map_err(infallible)?.get();
        // TODO: handle versions (especially MAJOR version)
        let major = buf.decode::<u8>().map_err(infallible)?;
        let minor = buf.decode::<u8>().map_err(infallible)?;

        // Skip additional header data
        // TODO: Check if length is truncated when usize is  < 64bit
        if let Some(unknown) = (length as usize).checked_sub(HEADER_CONTENTS) {
            buf.skip(unknown).map_err(infallible)?;
        }

        let guard = buf.decode::<u8>().map_err(infallible)?;
        if guard != Guard::generate(&length.to_le_bytes()) {
            return Err(Err(CodecError::Badness));
        }

        Ok((
            buf.len(),
            Header {
                version: (major, minor),
            },
        ))
    }
}

impl Serialisable<'_> for RecordMeta {
    type Err = CodecError;

    fn encode_into(&self, buf: &mut [u8]) -> crate::io::EncodeResult<Self::Err> {
        let mut buf = buf;
        let len = self.length as u64;

        buf.encode(PVarint::from(len)).map_err(infallible)?;
        if len != RECORD_EOS {
            buf.encode(self.source_id).map_err(infallible)?;
            buf.encode(self.type_id).map_err(infallible)?;
        }

        Ok(buf.len())
    }

    fn decode_from(buf: &[u8]) -> crate::io::DecodeResult<Self, Self::Err> {
        let mut buf = buf;

        let length = buf.decode::<PVarint>().map_err(infallible)?.get() as usize;
        match length {
            // 0 = End Of Stream indicator
            0 => Ok((buf.len(), RecordMeta::new_eos())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_LEN => Err(Err(CodecError::Badness)),
            // Contains contents + zero/some data
            RECORD_META_LEN.. => {
                let source_id = buf.decode::<u16>().map_err(infallible)?;
                let type_id = buf.decode::<u16>().map_err(infallible)?;

                Ok((
                    buf.len(),
                    RecordMeta {
                        length,
                        source_id,
                        type_id,
                    },
                ))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    pub fn ref_header() -> Header {
        Header { version: (1, 2) }
    }

    pub fn ref_header_bytes() -> Vec<u8> {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(&MAGIC_BYTES); // Magic bytes
        data.extend_from_slice(&0b111_u8.to_le_bytes()); // Length Pvarint(3)
        data.extend_from_slice(&1_u8.to_le_bytes()); // Version (Major)
        data.extend_from_slice(&2_u8.to_le_bytes()); // Version (Minor)
        data.extend_from_slice(&Guard::generate(&3u8.to_le_bytes()).to_le_bytes()); // Guard

        data
    }

    #[test]
    fn decode_header() {
        let header = ref_header();
        let header_bytes = ref_header_bytes();

        assert_eq!(header_bytes.len(), 8);
        let mut data = header_bytes.as_slice();
        let output = data.decode::<Header>().unwrap();

        // Verify decode
        assert_eq!(header, output);

        // Verify all data is read
        assert_eq!(data.len(), 0);
    }
}
