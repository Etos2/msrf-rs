use bitflags::bitflags;
use std::collections::HashMap;

use crate::CODEC_ID_EOS;

pub type CodecTable = HashMap<u64, CodecEntry>;

// TODO: Impl CodecOwned vs CodecRef?
#[derive(Debug, Clone, PartialEq)]
pub struct Codec {
    pub(crate) id: u64,
    pub(crate) entry: CodecEntry,
}

impl From<(u64, CodecEntry)> for Codec {
    fn from(val: (u64, CodecEntry)) -> Self {
        Codec {
            id: val.0,
            entry: val.1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecEntry {
    pub(crate) version: u16,
    pub(crate) name: String,
}

impl From<Codec> for CodecEntry {
    fn from(val: Codec) -> Self {
        CodecEntry {
            version: val.entry.version,
            name: val.entry.name,
        }
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct HeaderFlags: u16 {
        // TODO
    }
}

impl From<u16> for HeaderFlags {
    fn from(val: u16) -> Self {
        HeaderFlags(val.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Header<'a> {
    pub(crate) version: u16,
    pub(crate) flags: HeaderFlags,
    pub(crate) codec_table: &'a CodecTable,
}

impl<'a> Header<'a> {
    pub fn get_codec(&self, index: u64) -> Option<Codec> {
        Some((index, self.codec_table.get(&index)?.clone()).into())
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn flags(&self) -> HeaderFlags {
        self.flags
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderOwned {
    pub(crate) version: u16,
    pub(crate) flags: HeaderFlags,
    pub(crate) codec_table: CodecTable,
}

impl HeaderOwned {
    pub fn insert_codec(&mut self, val: Codec) {
        self.codec_table.insert(val.id, val.into());
    }

    pub fn get_codec(&self, index: u64) -> Option<Codec> {
        self.codec_table
            .get(&index)
            .map(|entry| (index, entry.clone()).into())
    }

    pub fn codecs(&self) -> impl Iterator<Item = Codec> + use<'_> {
        self.codec_table
            .iter()
            .map(|(i, entry)| (*i, entry.clone()).into())
    }

    pub fn codecs_mut(&mut self) -> impl Iterator<Item = Codec> + use<'_> {
        self.codec_table
            .iter_mut()
            .map(|(i, entry)| (*i, entry.clone()).into())
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn flags(&self) -> HeaderFlags {
        self.flags
    }

    pub fn as_ref<'a>(&'a self) -> Header<'a> {
        Header {
            version: self.version,
            flags: self.flags,
            codec_table: &self.codec_table,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Record<'a> {
    pub(crate) codec_id: u64,
    pub(crate) type_id: u64,
    pub(crate) val: Option<&'a [u8]>,
}

impl<'a> Record<'a> {
    pub fn new(codec_id: u64, type_id: u64, val: &'a [u8]) -> Record<'a> {
        Record {
            codec_id,
            type_id,
            val: Some(val),
        }
    }

    pub fn codec_id(&self) -> u64 {
        self.codec_id
    }

    pub fn type_id(&self) -> u64 {
        self.type_id
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.val
    }

    pub fn to_owned(&self) -> RecordOwned {
        RecordOwned {
            codec_id: self.codec_id,
            type_id: self.type_id,
            val: self.val.map(|v| Box::from(v)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordOwned {
    pub(crate) codec_id: u64,
    pub(crate) type_id: u64,
    pub(crate) val: Option<Box<[u8]>>,
}

impl RecordOwned {
    pub fn from_slice(codec_id: u64, type_id: u64, val: &[u8]) -> RecordOwned {
        RecordOwned {
            codec_id,
            type_id,
            val: Some(Box::from(val)),
        }
    }

    pub fn from_box(codec_id: u64, type_id: u64, val: Box<[u8]>) -> RecordOwned {
        RecordOwned {
            codec_id,
            type_id,
            val: Some(val),
        }
    }

    pub fn from_eos() -> RecordOwned {
        RecordOwned {
            codec_id: CODEC_ID_EOS,
            type_id: 0,
            val: None,
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

    pub fn value(&self) -> Option<&[u8]> {
        self.val.as_deref()
    }

    pub fn as_ref<'a>(&'a self) -> Record<'a> {
        Record {
            codec_id: self.codec_id,
            type_id: self.type_id,
            val: self.val.as_deref(),
        }
    }
}
