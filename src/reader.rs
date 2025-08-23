use std::{error::Error, fmt::Display};

use crate::{
    codec::v0_0::{self, Deserialiser},
    data::{Header, RecordMeta},
};

pub type DeserialiseResult<T> = Result<(T, usize), Result<usize, ParserError>>;

fn deserialise_header(input: &[u8]) -> DeserialiseResult<Header> {
    todo!()
}

pub(crate) trait RawDeserialiser {
    fn deserialise_record_head(&self, input: &[u8]) -> DeserialiseResult<RecordMeta>;
    fn deserialise_record_tail(&self, input: &[u8]) -> DeserialiseResult<()>;
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum AnyDeserialiser {
    V0_0(v0_0::Deserialiser),
}

impl AnyDeserialiser {
    pub fn new() -> Self {
        Self::V0_0(Deserialiser)
    }

    pub fn with_version(version: (u8, u8)) -> Option<Self> {
        match version {
            (0, 0) => Some(Self::V0_0(Deserialiser)),
            _ => None,
        }
    }
}

impl RawDeserialiser for AnyDeserialiser {
    fn deserialise_record_head(&self, input: &[u8]) -> DeserialiseResult<RecordMeta> {
        match self {
            AnyDeserialiser::V0_0(des) => des.deserialise_record_head(input),
        }
    }

    fn deserialise_record_tail(&self, input: &[u8]) -> DeserialiseResult<()> {
        match self {
            AnyDeserialiser::V0_0(des) => des.deserialise_record_tail(input),
        }
    }
}

// pub struct Uninit;

// pub struct Init {
//     version: (u8, u8),
//     ser: Box<dyn Serialiser>,
// }

#[derive(PartialEq, Debug, Clone)]
pub enum ParserError {
    Unsupported((u8, u8)),
    Guard(u8),
    MagicBytes([u8; 4]),
    Length(u64),
    Eos,
}

impl Error for ParserError {}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::Unsupported((maj, min)) => write!(f, "unsupported version ({maj}.{min})"),
            ParserError::Guard(g) => write!(f, "expected guard ({g})"),
            ParserError::MagicBytes(b) => write!(f, "invalid magic bytes ({b:?})"),
            ParserError::Length(l) => write!(f, "invalid length ({l})"),
            ParserError::Eos => write!(f, "parser is finished (reached eos)"),
        }
    }
}

pub enum ParserResult {
    HeaderData(usize),
    RecordData(RecordMeta),
    NeedsData(usize),
    Eos,
}

#[derive(Debug, Default)]
enum State {
    #[default]
    ReadHeader,
    ReadRecordMeta(AnyDeserialiser),
    ReadRecordGuard(AnyDeserialiser),
    Eos,
}

#[derive(Default, Debug)]
pub struct ParserBuilder {}

impl ParserBuilder {
    pub fn build_raw() -> RawParser {
        RawParser::new()
    }
}

// TODO: Config (strictness, left-over header bytes, etc)
pub struct RawParser {
    state: State,
}

impl RawParser {
    fn new() -> Self {
        Self {
            state: State::ReadHeader,
        }
    }

    pub fn builder() -> ParserBuilder {
        ParserBuilder::default()
    }

    pub fn reset(&mut self) {
        self.state = State::ReadHeader
    }

    pub fn is_eos(&self) -> bool {
        matches!(self.state, State::Eos)
    }

    pub fn process(&mut self, input: &[u8]) -> Result<(usize, ParserResult), ParserError> {
        let mut read = 0;
        let (state, res) = match std::mem::take(&mut self.state) {
            State::ReadHeader => Self::process_header(input, &mut read),
            State::ReadRecordMeta(des) => Self::process_record_meta(input, &mut read, des),
            State::ReadRecordGuard(des) => Self::process_record_guard(input, &mut read, des),
            State::Eos => Err(ParserError::Eos),
        }?;
        self.state = state;
        Ok((read, res))
    }

    fn process_header(
        input: &[u8],
        bytes_read: &mut usize,
    ) -> Result<(State, ParserResult), ParserError> {
        match deserialise_header(input) {
            Ok((header, read)) => {
                *bytes_read += read;
                let version = header.version();
                match AnyDeserialiser::with_version(version) {
                    Some(des) => {
                        if header.remainder > 0 {
                            Ok((
                                State::ReadRecordMeta(des),
                                ParserResult::HeaderData(header.remainder),
                            ))
                        } else {
                            Self::process_record_meta(&input[read..], bytes_read, des)
                        }
                    }
                    None => Err(ParserError::Unsupported(version)),
                }
            }
            Err(e) => e.map(|need| (State::ReadHeader, ParserResult::NeedsData(need))),
        }
    }

    fn process_record_meta(
        input: &[u8],
        bytes_read: &mut usize,
        des: AnyDeserialiser,
    ) -> Result<(State, ParserResult), ParserError> {
        match des.deserialise_record_head(input) {
            Ok((meta, read)) => {
                *bytes_read += read;
                if meta.is_eos() {
                    Ok((State::Eos, ParserResult::Eos))
                } else {
                    Ok((State::ReadRecordGuard(des), ParserResult::RecordData(meta)))
                }
            }
            Err(e) => e.map(|need| (State::ReadRecordMeta(des), ParserResult::NeedsData(need))),
        }
    }

    fn process_record_guard(
        input: &[u8],
        bytes_read: &mut usize,
        des: AnyDeserialiser,
    ) -> Result<(State, ParserResult), ParserError> {
        match des.deserialise_record_tail(input) {
            Ok((_, read)) => {
                *bytes_read += read;
                let input = &input[..*bytes_read];
                Self::process_record_meta(input, bytes_read, des)
            }
            Err(e) => e.map(|need| (State::ReadRecordGuard(des), ParserResult::NeedsData(need))),
        }
    }
}
