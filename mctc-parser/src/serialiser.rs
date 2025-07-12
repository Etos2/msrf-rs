use std::{
    error::Error,
    fmt::{self, Display},
};

use crate::{
    data::{Header, RecordMeta},
    error::{CodecError, CodecResult},
    io::{
        Serialisable, SerialiseExt,
        util::{Guard, PVarint},
    },
};

pub(crate) const MAGIC_BYTES: [u8; 4] = *b"MCTC";
pub(crate) const HEADER_LEN: usize = 8;
pub(crate) const HEADER_CONTENTS: usize = 3;
pub(crate) const RECORD_META_MIN_LEN: usize = 4;
pub(crate) const RECORD_EOS: u64 = u64::MIN;

// TODO: Move into lib.rs? (make serialiser.rs internal decoding logic)
#[derive(Debug)]
pub struct Decoder {}

#[derive(Debug)]
pub enum MctcError {
    MagicByte([u8; 4]),
    Guard(u8),
    RecordLength(usize),
}

impl Error for MctcError {}

impl Display for MctcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MctcError::MagicByte(magic) => write!(
                f,
                "expected magic bytes \"{MAGIC_BYTES:?}\" found \"{magic:?}\""
            ),
            MctcError::Guard(b) => write!(f, "invalid guard \"{b:#X}\""),
            MctcError::RecordLength(len) => write!(
                f,
                "length \"{len}\" must be zero or greater than {RECORD_META_MIN_LEN}"
            ),
        }
    }
}

impl Serialisable<'_> for Header {
    type Err = MctcError;

    fn encode_into(&self, buf: &mut [u8]) -> CodecResult<usize> {
        let buf_len = buf.len();
        let mut dst = buf;

        if HEADER_LEN > dst.len() {
            return Err(CodecError::Needed(HEADER_LEN - dst.len()));
        }

        dst.encode::<[u8; 4]>(MAGIC_BYTES)?;
        dst.encode(PVarint::new(HEADER_CONTENTS as u64))?;
        dst.encode(self.version.0)?;
        dst.encode(self.version.1)?;
        dst.encode(Guard::from(HEADER_CONTENTS as u8))?;

        Ok(buf_len - dst.len())
    }

    fn decode_from(buf: &[u8]) -> CodecResult<(usize, Self)> {
        let mut src = buf;
        if HEADER_LEN > src.len() {
            return Err(CodecError::Needed(HEADER_LEN - src.len()));
        }

        let magic_bytes = src.decode::<[u8; 4]>()?;
        if magic_bytes != MAGIC_BYTES {
            return Err(CodecError::from_custom(MctcError::MagicByte(magic_bytes)));
        }

        let length = src.decode::<PVarint>()?.get();
        // TODO: handle versions (especially MAJOR version)
        let major = src.decode::<u8>()?;
        let minor = src.decode::<u8>()?;

        // Skip additional header data
        // TODO: Check if length is truncated when usize is  < 64bit
        if let Some(unknown) = (length as usize).checked_sub(HEADER_CONTENTS)
            && unknown != 0
        {
            src.skip(unknown)?;
        }

        let guard = src.decode::<u8>()?;
        if guard != Guard::generate(&length.to_le_bytes()) {
            return Err(CodecError::from_custom(MctcError::Guard(guard)));
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

    fn encode_into(&self, buf: &mut [u8]) -> CodecResult<usize> {
        let buf_len = buf.len();
        let mut dst = buf;
        let len = self.length as u64;

        dst.encode(PVarint::new(len))?;
        if len != RECORD_EOS {
            dst.encode(self.source_id)?;
            dst.encode(self.type_id)?;
        }

        Ok(buf_len - dst.len())
    }

    fn decode_from(buf: &[u8]) -> CodecResult<(usize, Self)> {
        let mut src = buf;

        let length = src.decode::<PVarint>()?.get() as usize;
        match length {
            // 0 = End Of Stream indicator
            0 => Ok((src.len(), RecordMeta::new_eos())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_MIN_LEN => Err(CodecError::from_custom(MctcError::RecordLength(length))),
            // Contains contents + zero/some data
            RECORD_META_MIN_LEN.. => {
                let source_id = src.decode::<u16>()?;
                let type_id = src.decode::<u16>()?;

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
