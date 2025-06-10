use bitflags::bitflags;

use crate::CODEC_ID_EOS;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Header {
    pub(crate) version: (u8, u8),
}

impl Header {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(&self) -> (u8, u8) {
        self.version
    }
}

bitflags! {
    #[derive(Default, Debug, Clone, PartialEq)]
    pub struct RecordFlags: u8 {
        const TYPE_INHERIT = 0b00000001;
        const SOURCE_INHERIT = 0b0000010;
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RecordMeta2 {
    pub(crate) length: usize,
    pub(crate) source_id: u16,
    pub(crate) type_id: u16,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RecordMeta {
    pub(crate) codec_id: u64,
    pub(crate) type_id: u64,
    pub(crate) length: usize,
}

impl RecordMeta {
    pub fn new_eos() -> Self {
        RecordMeta {
            codec_id: CODEC_ID_EOS,
            type_id: 0,
            length: 9,
        }
    }

    pub fn is_eos(&self) -> bool {
        self.codec_id == CODEC_ID_EOS
    }

    pub fn codec_id(&self) -> u64 {
        self.codec_id
    }

    pub fn type_id(&self) -> u64 {
        self.type_id
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

pub trait RecordExt {
    fn codec_id(&self) -> u64;
    fn type_id(&self) -> u64;
    fn len(&self) -> usize;
    fn is_eos(&self) -> bool;
    fn value(&self) -> Option<&[u8]>;
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Record<'a> {
    pub(crate) codec_id: u64,
    pub(crate) type_id: u64,
    pub(crate) val: Option<&'a [u8]>,
}

impl<'a> Record<'a> {
    pub fn new(codec_id: u64, type_id: u64, val: &'a [u8]) -> Self {
        Record {
            codec_id,
            type_id,
            val: Some(val),
        }
    }

    pub fn new_empty(codec_id: u64, type_id: u64) -> Self {
        Record {
            codec_id,
            type_id,
            val: None,
        }
    }

    pub fn new_eos() -> Self {
        Record {
            codec_id: CODEC_ID_EOS,
            type_id: 0,
            val: None,
        }
    }
}

impl<'a> RecordExt for Record<'a> {
    fn codec_id(&self) -> u64 {
        self.codec_id
    }

    fn type_id(&self) -> u64 {
        self.type_id
    }

    fn len(&self) -> usize {
        self.val.as_ref().map(|data| data.len()).unwrap_or(0)
    }

    fn is_eos(&self) -> bool {
        self.codec_id == CODEC_ID_EOS
    }

    fn value(&self) -> Option<&[u8]> {
        self.val.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordOwned {
    pub(crate) codec_id: u64,
    pub(crate) type_id: u64,
    pub(crate) val: Option<Box<[u8]>>,
}

impl RecordOwned {
    pub fn new(codec_id: u64, type_id: u64, val: &[u8]) -> Self {
        RecordOwned {
            codec_id,
            type_id,
            val: Some(Box::from(val)),
        }
    }

    pub fn new_from_box(codec_id: u64, type_id: u64, val: Box<[u8]>) -> Self {
        RecordOwned {
            codec_id,
            type_id,
            val: Some(val),
        }
    }

    pub fn new_eos() -> Self {
        RecordOwned {
            codec_id: CODEC_ID_EOS,
            type_id: 0,
            val: None,
        }
    }
}

impl RecordExt for RecordOwned {
    fn codec_id(&self) -> u64 {
        self.codec_id
    }

    fn type_id(&self) -> u64 {
        self.type_id
    }

    fn len(&self) -> usize {
        self.val.as_ref().map(|data| data.len()).unwrap_or(0)
    }

    fn is_eos(&self) -> bool {
        self.codec_id == CODEC_ID_EOS
    }

    fn value(&self) -> Option<&[u8]> {
        self.val.as_deref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO
}
