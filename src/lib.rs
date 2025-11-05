pub mod error;
#[cfg(feature = "reader")]
pub mod codec;
#[cfg(feature = "reader")]
pub mod reader;

pub const RECORD_EOS: u16 = u16::MAX;
pub const CURRENT_VERSION: u16 = 0;
pub(crate) const TYPE_CONTAINER_MASK: u16 = 0b1000_0000_0000_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
}

impl Header {
    pub const fn new(version: u16) -> Self {
        Self { version }
    }

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

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordMeta {
    pub(crate) length: u64,
    pub(crate) source_id: u16,
    pub(crate) type_id: u16,
}

impl RecordMeta {
    pub const fn new(length: u64, source_id: u16, type_id: u16) -> Self {
        Self {
            length,
            source_id,
            type_id,
        }
    }

    pub fn new_eos() -> Self {
        Self::default()
    }

    pub const fn len(&self) -> u64 {
        self.length
    }

    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub const fn is_eos(&self) -> bool {
        self.source_id == RECORD_EOS
    }

    pub const fn source_id(&self) -> u16 {
        self.source_id
    }

    pub const fn type_id(&self) -> u16 {
        self.type_id & !TYPE_CONTAINER_MASK
    }

    pub const fn is_container(&self) -> bool {
        self.type_id & TYPE_CONTAINER_MASK == TYPE_CONTAINER_MASK
    }

    pub const fn value_len(&self) -> u64 {
        self.length
    }

    pub const fn into_ids(self) -> RecordId {
        RecordId::new(self.source_id, self.type_id)
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordId {
    pub(crate) source_id: u16,
    pub(crate) type_id: u16,
}

impl RecordId {
    pub const fn new(source_id: u16, type_id: u16) -> Self {
        Self { source_id, type_id }
    }

    pub fn new_eos() -> Self {
        Self::default()
    }

    pub const fn is_eos(&self) -> bool {
        self.source_id == RECORD_EOS
    }

    pub const fn source_id(&self) -> u16 {
        self.source_id
    }

    pub const fn type_id(&self) -> u16 {
        self.type_id & !TYPE_CONTAINER_MASK
    }

    pub const fn is_container(&self) -> bool {
        self.type_id & TYPE_CONTAINER_MASK == TYPE_CONTAINER_MASK
    }

    pub const fn into_meta(self, length: u64) -> RecordMeta {
        RecordMeta::new(length, self.source_id, self.type_id)
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
        let record_meta = RecordMeta::new(LEN, SOURCE, TYPE);
        let record_id = record_meta.into_ids();

        assert_eq!(record_meta, record_id.into_meta(LEN));
        assert_pair_eq(record_meta.is_container(), record_id.is_container(), false);
        assert_pair_eq(record_meta.is_eos(), record_id.is_eos(), false);
        assert_pair_eq(record_meta.source_id(), record_id.source_id(), SOURCE);
        assert_pair_eq(record_meta.type_id(), record_id.type_id(), TYPE);
    }

    #[test]
    fn record_interop_container() {
        const TYPE_WITH_CONTAINER: u16 = TYPE | TYPE_CONTAINER_MASK;
        let record_meta = RecordMeta::new(LEN, SOURCE, TYPE_WITH_CONTAINER);
        let record_id = record_meta.into_ids();

        assert_eq!(record_meta, record_id.into_meta(LEN));
        assert_pair_eq(record_meta.is_container(), record_id.is_container(), true);
        assert_pair_eq(record_meta.is_eos(), record_id.is_eos(), false);
        assert_pair_eq(record_meta.source_id(), record_id.source_id(), SOURCE);
        assert_pair_eq(record_meta.type_id(), record_id.type_id(), TYPE);
    }

    #[test]
    fn record_interop_eos() {
        let record_meta = RecordMeta::new(LEN, RECORD_EOS, TYPE);
        let record_id = record_meta.into_ids();

        assert_eq!(record_meta, record_id.into_meta(LEN));
        assert_pair_eq(record_meta.is_container(), record_id.is_container(), false);
        assert_pair_eq(record_meta.is_eos(), record_id.is_eos(), true);
        assert_pair_eq(record_meta.source_id(), record_id.source_id(), RECORD_EOS);
        assert_pair_eq(record_meta.type_id(), record_id.type_id(), TYPE);
    }
}
