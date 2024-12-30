pub mod reader;
pub mod error;

use bitflags::bitflags;
use std::collections::HashMap;

const GUARD: u8 = 0x00;
const MAGIC_BYTES: [u8; 4] = *b"MCTC";
const CODEC_ID_EOS: u8 = 0xFF;

// TODO: Impl CodecOwned vs CodecRef?
#[derive(Debug, Clone, PartialEq)]
pub struct Codec {
    id: u8,
    entry: CodecEntry,
}

impl From<(u8, CodecEntry)> for Codec {
    fn from(val: (u8, CodecEntry)) -> Self {
        Codec {
            id: val.0,
            entry: val.1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CodecEntry {
    type_id_len: u8,
    type_length_len: u8,
    name: String,
}

impl From<Codec> for CodecEntry {
    fn from(val: Codec) -> Self {
        CodecEntry {
            type_id_len: val.entry.type_id_len,
            type_length_len: val.entry.type_length_len,
            name: val.entry.name,
        }
    }
}

type CodecTable = HashMap<u8, CodecEntry>;

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
    version: u16,
    flags: HeaderFlags,
    codec_table: &'a [CodecEntry],
}

impl<'a> Header<'a> {
    pub fn get_codec(&self, index: u8) -> Codec {
        let i = index as usize;
        (index, self.codec_table[i].clone()).into()
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
    version: u16,
    flags: HeaderFlags,
    codec_table: CodecTable,
}

impl HeaderOwned {
    pub fn insert_codec(&mut self, val: Codec) {
        self.codec_table.insert(val.id, val.into());
    }

    pub fn get_codec(&self, index: u8) -> Option<Codec> {
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
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Record<'a> {
    codec_id: u8,
    type_id: u64,
    val: &'a [u8],
}

impl<'a> Record<'a> {
    pub fn new(codec_id: u8, type_id: u64, val: &'a [u8]) -> Record<'a> {
        Record {
            codec_id,
            type_id,
            val,
        }
    }

    pub fn codec_id(&self) -> u8 {
        self.codec_id
    }

    pub fn type_id(&self) -> u64 {
        self.type_id
    }

    pub fn value(&self) -> &[u8] {
        self.val
    }

    pub fn to_owned(&self) -> RecordOwned {
        RecordOwned {
            codec_id: self.codec_id,
            type_id: self.type_id,
            val: Box::from(self.val),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordOwned {
    codec_id: u8,
    type_id: u64,
    val: Box<[u8]>,
}

impl RecordOwned {
    pub fn from_slice(codec_id: u8, type_id: u64, val: &[u8]) -> RecordOwned {
        RecordOwned {
            codec_id,
            type_id,
            val: Box::from(val),
        }
    }

    pub fn from_box(codec_id: u8, type_id: u64, val: Box<[u8]>) -> RecordOwned {
        RecordOwned {
            codec_id,
            type_id,
            val,
        }
    }

    pub fn codec_id(&self) -> u8 {
        self.codec_id
    }

    pub fn type_id(&self) -> u64 {
        self.type_id
    }

    pub fn value(&self) -> &[u8] {
        &self.val
    }

    pub fn borrow<'a>(&'a self) -> Record<'a> {
        Record {
            codec_id: self.codec_id,
            type_id: self.type_id,
            val: &self.val,
        }
    }
}
