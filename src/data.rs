use crate::{
    CURRENT_VERSION,
    codec::constants::{HEADER_LEN, RECORD_META_MIN_LEN},
};

pub(crate) const TYPE_ID_CONTAINER_MASK: u16 = 0b1000_0000_0000_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub(crate) length: u64,
    pub(crate) version: (u8, u8),
}

impl Header {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn new_with_version(major: u8, minor: u8) -> Self {
        Self {
            length: HEADER_LEN as u64,
            version: (major, minor),
        }
    }

    pub const fn version(&self) -> (u8, u8) {
        self.version
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            length: HEADER_LEN as u64,
            version: CURRENT_VERSION,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RecordMeta {
    pub(crate) length: u64,
    pub(crate) source_id: u16,
    pub(crate) type_id: u16,
}

impl RecordMeta {
    pub const fn new(value_len: u64, source_id: u16, type_id: u16) -> Self {
        Self {
            length: value_len + RECORD_META_MIN_LEN, // TODO: Remove dependence on const
            source_id,
            type_id,
        }
    }

    pub fn new_eos() -> Self {
        Self::default()
    }

    pub const fn length(&self) -> u64 {
        self.length
    }

    pub const fn is_eos(&self) -> bool {
        self.length() == 0
    }

    pub const fn source_id(&self) -> u16 {
        self.source_id
    }

    pub const fn type_id(&self) -> u16 {
        self.type_id & !TYPE_ID_CONTAINER_MASK
    }

    pub const fn is_container(&self) -> bool {
        self.type_id & TYPE_ID_CONTAINER_MASK == TYPE_ID_CONTAINER_MASK
    }

    pub const fn value_len(&self) -> u64 {
        if self.is_eos() {
            0
        } else {
            // TODO: Remove dependence on const
            self.length
                .checked_sub(RECORD_META_MIN_LEN)
                .expect("length should always be >= 5 || 0")
        }
    }
}
