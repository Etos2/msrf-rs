use std::{cmp::Ordering, error::Error, fmt::Display};

use crate::{
    codec::{
        RawDeserialiser, constants::HEADER_LEN,
    },
    data::{Header, RecordMeta},
};

pub type DeserialiseResult<T> = Result<(T, usize), Result<usize, ParserError>>;

// TODO: Re-evaluate variant nessicity (e.g. length?)
#[derive(PartialEq, Debug, Clone)]
pub enum ParserError {
    Need(usize),
    Unsupported((u8, u8)),
    Guard(u8),
    MagicBytes([u8; 4]),
    Length(u64),
    ContainerOverflow(u64),
    ContainerUnderflow(u64),
    UnexpectedEos,
}

impl Error for ParserError {}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::Need(n) => write!(f, "need {n} more bytes to continue"),
            ParserError::Unsupported((maj, min)) => write!(f, "unsupported version ({maj}.{min})"),
            ParserError::Guard(g) => write!(f, "expected guard ({g})"),
            ParserError::MagicBytes(b) => write!(f, "invalid magic bytes ({b:?})"),
            ParserError::Length(l) => write!(f, "invalid length ({l})"),
            ParserError::UnexpectedEos => write!(f, "unexpected eos"),
            ParserError::ContainerOverflow(n) => {
                write!(
                    f,
                    "container overflow (record is {n} bytes longer than it's container)"
                )
            }
            ParserError::ContainerUnderflow(n) => {
                write!(f, "container underflow (expected {n} more bytes)")
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct ParserBuilder {}

impl ParserBuilder {
    pub fn build_raw<D: RawDeserialiser + Clone + Default + std::fmt::Debug>() -> Reader<D> {
        Reader::new()
    }
}

impl<D: RawDeserialiser> ParserState<D> {
    fn is_eos(&self) -> bool {
        matches!(self, ParserState::Eos)
    }
}

#[derive(Debug, PartialEq)]
pub struct DataChunk<'a> {
    data: &'a [u8],
    complete: bool,
}

impl<'a> DataChunk<'a> {
    fn new(data: &'a [u8]) -> Self {
        DataChunk {
            data,
            complete: true,
        }
    }

    fn new_incomplete(data: &'a [u8]) -> Self {
        DataChunk {
            data,
            complete: false,
        }
    }

    fn new_with_len(data: &'a [u8], expected_len: usize) -> Self {
        if data.len() >= expected_len {
            Self::new(&data[..expected_len])
        } else {
            Self::new_incomplete(data)
        }
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn take(self) -> &'a [u8] {
        self.data
    }
}

impl<'a> AsRef<[u8]> for DataChunk<'a> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'a> From<&'a [u8]> for DataChunk<'a> {
    fn from(value: &'a [u8]) -> Self {
        DataChunk::new(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParserEvent<'a> {
    Header(Header),
    HeaderUnknown(DataChunk<'a>),
    RecordMeta(RecordMeta, usize),
    RecordValue(DataChunk<'a>),
    NeedData(usize),
    Eos,
}

impl ParserEvent<'_> {
    pub fn is_eos(&self) -> bool {
        matches!(self, ParserEvent::Eos)
    }
}

#[derive(Debug, Default, Clone)]
enum ParserState<D: RawDeserialiser> {
    Header(Option<D>),
    HeaderUnknown(D, u64),
    RecordMeta(D),
    RecordValue(D, u64),
    Guard(D),
    #[default]
    Eos,
}

// TODO: Config (strictness, left-over header bytes, etc)
#[derive(Debug, Default)]
pub struct Reader<D: RawDeserialiser + Clone> {
    state: ParserState<D>,
    layers: Vec<u64>,
    bytes_read: usize,
}

impl<D: RawDeserialiser + Clone + Default + std::fmt::Debug> Reader<D> {
    fn new() -> Self {
        Reader {
            state: ParserState::Header(None),
            ..Self::default()
        }
    }

    fn new_with(des: D) -> Self {
        Reader {
            state: ParserState::Header(Some(des)),
            ..Self::default()
        }
    }

    fn get_data_chunk<'a>(buf: &mut &'a [u8], len: usize) -> Result<DataChunk<'a>, ParserError> {
        let data = DataChunk::new_with_len(buf, len);
        let data_len = data.as_ref().len();
        if data_len == 0 {
            Err(ParserError::Need(len))
        } else {
            Ok(data)
        }
    }

    fn process<'a>(&mut self, buf: &mut &'a [u8]) -> Result<ParserEvent<'a>, ParserError> {
        loop {
            eprintln!("{:?}", self.state);
            match self.impl_process(buf) {
                Ok((maybe_event, read)) => {
                    self.bytes_read += read;
                    *buf = &buf[read..];
                    if let Some(event) = maybe_event {
                        return Ok(event);
                    }
                }
                Err(ParserError::Need(n)) => return Ok(ParserEvent::NeedData(n)),
                Err(e) => return Err(e),
            }
        }
    }

    fn impl_process<'a>(
        &mut self,
        buf: &mut &'a [u8],
    ) -> Result<(Option<ParserEvent<'a>>, usize), ParserError> {
        match self.state.clone() {
            ParserState::Header(None) => {
                // TODO: Change <D> to AnyDeserialiser
                todo!()

                // let (header, read) = default_deserialise_header(buf)?;
                // let version = header.version();
                // let des = AnyDeserialiser::with_version(version)
                //     .ok_or(ParserError::Unsupported(version))?;
            }
            ParserState::Header(Some(des)) => {
                let (header, read) = des.deserialise_header(buf)?;

                self.state = match header.length.checked_sub(HEADER_LEN as u64) {
                    Some(rem) => ParserState::HeaderUnknown(des, rem),
                    None => ParserState::Guard(des),
                };

                Ok((Some(ParserEvent::Header(header)), read))
            }
            ParserState::HeaderUnknown(des, rem) => {
                let data = Self::get_data_chunk(buf, rem as usize)?;
                let data_len = data.as_ref().len();

                self.state = if data.is_complete() {
                    ParserState::Guard(des)
                } else {
                    ParserState::HeaderUnknown(des, rem - data_len as u64)
                };

                Ok((Some(ParserEvent::HeaderUnknown(data)), data_len))
            }
            ParserState::RecordMeta(des) => {
                let (meta, read) = des.deserialise_record_meta(buf)?;
                if meta.is_eos() {
                    self.state = ParserState::Eos;
                    Ok((Some(ParserEvent::Eos), read))
                } else {
                    self.state = if meta.is_container() {
                        // TODO: Assert validity
                        self.layers.push(self.bytes_read as u64 + meta.length);
                        ParserState::RecordMeta(des)
                    } else {
                        // TODO: Validity, cannot be < 5: Length(1 bytes) + Source(2 bytes) + Type(2 bytes) + Guard(1 bytes) == 5 minimum
                        // TODO: Length(1 bytes) is NOT TRUE, assert consumed value correctly (make meta store length of contents only, shift during serdes)
                        if meta.length() <= 6 {
                            ParserState::Guard(des)
                        } else {
                            ParserState::RecordValue(des, meta.length() - 6)
                        }
                    };
                    Ok((Some(ParserEvent::RecordMeta(meta, self.layers.len())), read))
                }
            }
            ParserState::RecordValue(des, rem) => {
                let data = Self::get_data_chunk(buf, rem as usize)?;
                let data_len = data.as_ref().len();

                self.state = if data.is_complete() {
                    ParserState::Guard(des)
                } else {
                    ParserState::RecordValue(des, rem - data_len as u64)
                };

                Ok((Some(ParserEvent::RecordValue(data)), data_len))
            }
            ParserState::Guard(des) => {
                let (_, read) = des.deserialise_guard(buf)?;
                if let Some(layer) = self.layers.last() {
                    self.state = match layer.cmp(&(self.bytes_read as u64 + 1)) {
                        Ordering::Less => {
                            return Err(ParserError::ContainerUnderflow(
                                self.bytes_read as u64 - layer,
                            ));
                        }
                        Ordering::Equal => {
                            self.layers.pop();
                            ParserState::Guard(des)
                        }
                        Ordering::Greater => ParserState::RecordMeta(des),
                    }
                } else {
                    self.state = ParserState::RecordMeta(des);
                }

                Ok((None, read))
            }
            ParserState::Eos => Err(ParserError::UnexpectedEos),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        codec::v0_0::{
            self,
            test::{REF_HEADER, REF_HEADER_BYTES, REF_RECORD_META, REF_RECORD_META_BYTES},
        },
        data::TYPE_ID_CONTAINER_MASK,
    };

    use super::*;

    #[test]
    fn parser_no_data() {
        let mut raw = Reader::new_with(v0_0::Deserialiser);
        let buf: [u8; 0] = [];
        let mut_buf = &mut buf.as_slice();

        // TODO: `ParserState::ReadHeader(None)` is incomplete
        // raw.state = ParserState::ReadHeader(None);
        // assert_eq!(raw.process(mut_buf).unwrap(), ParserEvent::NeedData(7));

        raw.state = ParserState::Header(Some(v0_0::Deserialiser));
        assert_eq!(raw.process(mut_buf).unwrap(), ParserEvent::NeedData(7));

        raw.state = ParserState::HeaderUnknown(v0_0::Deserialiser, 64);
        assert_eq!(raw.process(mut_buf).unwrap(), ParserEvent::NeedData(64));

        raw.state = ParserState::Guard(v0_0::Deserialiser);
        assert_eq!(raw.process(mut_buf).unwrap(), ParserEvent::NeedData(1));

        raw.state = ParserState::RecordValue(v0_0::Deserialiser, 64);
        assert_eq!(raw.process(mut_buf).unwrap(), ParserEvent::NeedData(64));

        // TODO: 1 is minimum to progress, but not ideal (minimum 5 required)
        raw.state = ParserState::RecordMeta(v0_0::Deserialiser);
        assert_eq!(raw.process(mut_buf).unwrap(), ParserEvent::NeedData(1));

        raw.state = ParserState::Eos;
        assert_eq!(
            raw.process(mut_buf).unwrap_err(),
            ParserError::UnexpectedEos
        );
    }

    #[test]
    fn parser_with_data() {
        const REF_DATA: &[u8; 27] = constcat::concat_bytes!(
            REF_HEADER_BYTES,
            REF_RECORD_META_BYTES,                           // Record 1
            &[0x00],                                         // Record 1: Guard
            &[0b10111_u8],                                   // Record 2: Length (11)
            &16_u16.to_le_bytes(),                           // Record 2: Source ID
            &(1_u16 | TYPE_ID_CONTAINER_MASK).to_le_bytes(), // Record 2: Type ID
            &[0b1101_u8],                                    // Record 3: Length (6)
            &33_u16.to_le_bytes(),                           // Record 3: Source ID
            &1_u16.to_le_bytes(),                            // Record 3: Type ID
            &[0x00],                                         // Record 3: Guard
            &[0x00],                                         // Record 2: Guard
            &[0b1_u8],                                         // Record 4: Length (EoS)
        );
        let mut reader = Reader::new_with(v0_0::Deserialiser);
        let mut_buf = &mut REF_DATA.as_slice();

        assert_eq!(
            reader.process(mut_buf).unwrap(),
            ParserEvent::Header(REF_HEADER)
        );
        assert_eq!(mut_buf.len(), 20);

        // Record 1
        assert_eq!(
            reader.process(mut_buf).unwrap(),
            ParserEvent::RecordMeta(REF_RECORD_META, 0)
        );
        assert_eq!(mut_buf.len(), 14);

        // Record 2
        assert_eq!(
            reader.process(mut_buf).unwrap(),
            ParserEvent::RecordMeta(
                RecordMeta {
                    length: 11,
                    source_id: 16,
                    type_id: 1 | TYPE_ID_CONTAINER_MASK
                },
                1
            )
        );
        assert_eq!(mut_buf.len(), 8);

        // Record 3
        assert_eq!(
            reader.process(mut_buf).unwrap(),
            ParserEvent::RecordMeta(
                RecordMeta {
                    length: 6,
                    source_id: 33,
                    type_id: 1
                },
                1
            )
        );
        assert_eq!(mut_buf.len(), 3);

        // Record 3
        assert_eq!(reader.process(mut_buf).unwrap(), ParserEvent::Eos);
        assert_eq!(mut_buf.len(), 0);
    }
}
