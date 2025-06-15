use crate::CURRENT_VERSION;

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub(crate) version: (u8, u8),
}

impl Header {
    pub fn new() -> Self {
        Header {
            version: CURRENT_VERSION,
        }
    }

    pub fn new_with_version(major: u8, minor: u8) -> Self {
        Header {
            version: (major, minor),
        }
    }

    pub fn version(&self) -> (u8, u8) {
        self.version
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RecordMeta {
    pub(crate) length: usize,
    pub(crate) source_id: u16,
    pub(crate) type_id: u16,
}

impl RecordMeta {
    pub fn new_eos() -> Self {
        RecordMeta::default()
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn is_eos(&self) -> bool {
        self.length() == 0
    }

    pub fn source_id(&self) -> u16 {
        self.source_id
    }

    pub fn type_id(&self) -> u16 {
        self.type_id
    }
}
