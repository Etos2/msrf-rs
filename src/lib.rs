use std::fmt::Debug;

use crate::io::SizedRecord;

#[cfg(any(feature = "reader", feature = "writer"))]
pub mod codec;
pub mod error;
pub mod io;
#[cfg(feature = "reader")]
pub mod reader;
#[cfg(feature = "writer")]
pub mod writer;

pub const RECORD_EOS: u16 = u16::MAX;
pub const CURRENT_VERSION: u16 = 0;
pub(crate) const TYPE_CONTAINER_MASK: u16 = 0x8000;

pub trait ConstAssignedId {
    const TYPE_ID: u16;
}

pub trait AssignedId {
    fn typ_id(&self) -> u16;
}

impl<T> AssignedId for T
where
    T: ConstAssignedId,
{
    fn typ_id(&self) -> u16 {
        T::TYPE_ID
    }
}

pub trait IntoMetadata<S>: AssignedId + SizedRecord<S> {
    fn meta(&self, ser: &S, source_id: u16) -> RecordMeta {
        RecordMeta::new(source_id, self.typ_id(), self.encoded_len(ser) as u64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
}

impl Header {
    #[must_use] 
    pub const fn new(version: u16) -> Self {
        Self { version }
    }

    #[must_use] 
    pub const fn version(&self) -> u16 {
        self.version
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordMeta {
    pub(crate) source_id: u16,
    pub(crate) type_id: u16,
    pub(crate) length: u64,
    pub(crate) contained: Option<u16>,
}

impl RecordMeta {
    #[must_use] 
    pub const fn new(source_id: u16, type_id: u16, length: u64) -> Self {
        Self {
            length,
            contained: None,
            source_id,
            type_id,
        }
    }

    #[must_use] 
    pub const fn new_container(source_id: u16, type_id: u16, length: u64, contained: u16) -> Self {
        Self {
            source_id,
            type_id,
            length,
            contained: Some(contained),
        }
    }

    #[must_use] 
    pub fn new_eos() -> Self {
        Self {
            length: 0,
            contained: None,
            source_id: RECORD_EOS,
            type_id: 0,
        }
    }

    #[must_use] 
    pub const fn len(&self) -> u64 {
        self.length
    }

    #[must_use] 
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[must_use] 
    pub const fn is_eos(&self) -> bool {
        self.source_id == RECORD_EOS
    }

    #[must_use] 
    pub const fn source_id(&self) -> u16 {
        self.source_id
    }

    #[must_use] 
    pub const fn type_id(&self) -> u16 {
        self.type_id
    }

    #[must_use] 
    pub const fn is_container(&self) -> bool {
        self.contained.is_some()
    }

    #[must_use] 
    pub const fn value_len(&self) -> u64 {
        self.length
    }

    #[must_use] 
    pub const fn contained(&self) -> Option<u16> {
        self.contained
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordId {
    pub(crate) source_id: u16,
    pub(crate) type_id: u16,
}

impl RecordId {
    #[must_use] 
    pub const fn new(source_id: u16, type_id: u16) -> Self {
        Self { source_id, type_id }
    }

    #[must_use] 
    pub fn new_eos() -> Self {
        Self {
            source_id: RECORD_EOS,
            type_id: 0,
        }
    }

    #[must_use] 
    pub const fn is_eos(&self) -> bool {
        self.source_id == RECORD_EOS
    }

    #[must_use]
    pub const fn source_id(&self) -> u16 {
        self.source_id
    }

    #[must_use]
    pub const fn type_id(&self) -> u16 {
        self.type_id & !TYPE_CONTAINER_MASK
    }

    #[must_use]
    pub const fn into_meta(self, len: u64) -> RecordMeta {
        RecordMeta::new(self.source_id, self.type_id, len)
    }
}

impl From<RecordMeta> for RecordId {
    fn from(value: RecordMeta) -> Self {
        RecordId::new(value.source_id, value.type_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    const LEN: u64 = 64;
    const SOURCE: u16 = 1;
    const TYPE: u16 = 2;

    fn assert_pair_eq<T: Debug + PartialEq>(first: T, second: T, expected: T) {
        assert_eq!(first, expected);
        assert_eq!(second, expected);
    }

    #[test]
    fn record_interop() {
        let record_meta = RecordMeta::new(SOURCE, TYPE, LEN);
        let record_id = RecordId::from(record_meta);

        assert_eq!(record_meta, record_id.into_meta(LEN));
        assert_pair_eq(record_meta.is_eos(), record_id.is_eos(), false);
        assert_pair_eq(record_meta.source_id(), record_id.source_id(), SOURCE);
        assert_pair_eq(record_meta.type_id(), record_id.type_id(), TYPE);
    }

    #[test]
    fn record_interop_container() {
        const COUNT: u16 = 5;
        let record_meta = RecordMeta::new_container(SOURCE, TYPE, LEN, COUNT);
        let record_id = RecordId::from(record_meta);

        assert_ne!(record_meta, record_id.into_meta(LEN));
        assert_pair_eq(record_meta.is_eos(), record_id.is_eos(), false);
        assert_pair_eq(record_meta.source_id(), record_id.source_id(), SOURCE);
        assert_pair_eq(record_meta.type_id(), record_id.type_id(), TYPE);
    }

    #[test]
    fn record_interop_eos() {
        let record_meta = RecordMeta::new(RECORD_EOS, TYPE, LEN);
        let record_id = RecordId::from(record_meta);

        assert_eq!(record_meta, record_id.into_meta(LEN));
        assert_pair_eq(record_meta.is_eos(), record_id.is_eos(), true);
        assert_pair_eq(record_meta.source_id(), record_id.source_id(), RECORD_EOS);
        assert_pair_eq(record_meta.type_id(), record_id.type_id(), TYPE);
    }
}
