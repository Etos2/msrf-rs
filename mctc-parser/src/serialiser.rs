use crate::{
    data::{Header, RecordMeta},
    error::CodecError,
    io::{
        DecodeResult, EncodeResult, Serialisable, SerialiseExt,
        util::{Guard, PVarint, infallible},
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
        let buf_len = buf.len();
        let mut dst = buf;

        if HEADER_LEN > dst.len() {
            return Err(Ok(HEADER_LEN - dst.len()));
        }

        dst.encode::<[u8; 4]>(MAGIC_BYTES).map_err(infallible)?;
        dst.encode(PVarint::new(HEADER_CONTENTS as u64)).map_err(infallible)?;
        dst.encode(self.version.0).map_err(infallible)?;
        dst.encode(self.version.1).map_err(infallible)?;
        dst.encode(Guard::from(HEADER_CONTENTS as u8)).map_err(infallible)?;

        Ok(buf_len - dst.len())
    }

    fn decode_from(buf: &[u8]) -> DecodeResult<Self, Self::Err> {
        let mut src = buf;
        if HEADER_LEN > src.len() {
            return Err(Ok(HEADER_LEN - src.len()));
        }

        let magic_bytes = src.decode::<[u8; 4]>().map_err(infallible)?;
        if magic_bytes != MAGIC_BYTES {
            return Err(Err(CodecError::Badness));
        }

        let length = src.decode::<PVarint>().map_err(infallible)?.get();
        // TODO: handle versions (especially MAJOR version)
        let major = src.decode::<u8>().map_err(infallible)?;
        let minor = src.decode::<u8>().map_err(infallible)?;

        // Skip additional header data
        // TODO: Check if length is truncated when usize is  < 64bit
        if let Some(unknown) = (length as usize).checked_sub(HEADER_CONTENTS)
            && unknown != 0
        {
            src.skip(unknown).map_err(infallible)?;
        }

        let guard = src.decode::<u8>().map_err(infallible)?;
        if guard != Guard::generate(&length.to_le_bytes()) {
            return Err(Err(CodecError::Badness));
        }

        Ok((
            buf.len() - src.len(),
            Header {
                version: (major, minor),
            },
        ))
    }
}

impl Serialisable<'_> for RecordMeta {
    type Err = CodecError;

    fn encode_into(&self, buf: &mut [u8]) -> crate::io::EncodeResult<Self::Err> {
        let buf_len = buf.len();
        let mut dst = buf;
        let len = self.length as u64;

        dst.encode(PVarint::new(len)).map_err(infallible)?;
        if len != RECORD_EOS {
            dst.encode(self.source_id).map_err(infallible)?;
            dst.encode(self.type_id).map_err(infallible)?;
        }

        Ok(buf_len - dst.len())
    }

    fn decode_from(buf: &[u8]) -> crate::io::DecodeResult<Self, Self::Err> {
        let mut src = buf;

        let length = src.decode::<PVarint>().map_err(infallible)?.get() as usize;
        match length {
            // 0 = End Of Stream indicator
            0 => Ok((src.len(), RecordMeta::new_eos())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_LEN => Err(Err(CodecError::Badness)),
            // Contains contents + zero/some data
            RECORD_META_LEN.. => {
                let source_id = src.decode::<u16>().map_err(infallible)?;
                let type_id = src.decode::<u16>().map_err(infallible)?;

                Ok((
                    buf.len() - src.len(),
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

    const REF_HEADER: Header = Header { version: (1, 2) };
    const REF_HEADER_BYTES: &[u8; 8] = constcat::concat_bytes!(
        &MAGIC_BYTES,                           // Magic bytes
        &[0b111_u8],                            // Length Pvarint(3)
        &[1_u8, 2_u8],                          // Version (Major, Minor)
        &[Guard::generate(&3u8.to_le_bytes())]  // Guard
    );

    #[test]
    fn decode_header() {
        let mut data = REF_HEADER_BYTES.as_slice();
        let output = data.decode::<Header>().unwrap();

        // Verify decode
        assert_eq!(REF_HEADER, output);
        // Verify all data is read
        assert_eq!(data.len(), 0);
    }
}
