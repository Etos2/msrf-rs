use std::ascii::Char as AsciiChar;

use bitflags::bitflags;

use crate::{Codec, CODEC_ID_EOS};

// TODO: Use ID + CodecEntry pairing (compared to existing (Index = ID + CodecEntry Pairing)
// Codecs are stored in a "sparse" vec
#[derive(Debug, Clone, PartialEq)]
pub struct CodecTable(pub(crate) Vec<Option<CodecEntry>>);

// TODO: Rewrite API to passively consider sparse CodecEntries in a non-intrusive manner
// TODO: Read API to find existing Codecs to determine if they are needed for reading
// TODO: Determine if this should be public (reader and writer registers for the user, is manual necessary?)
impl CodecTable {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: Better ID solution?
    pub fn register<C: Codec>(&mut self) -> Option<u64> {
        let entry = CodecEntry::new_ascii(C::VERSION, C::NAME);
        self.register_impl(entry)
    }

    #[inline]
    fn register_impl(&mut self, entry: CodecEntry) -> Option<u64> {
        if !self.contains_name(&entry.name.as_ref()) {
            match self.find_free() {
                Some(index) => {
                    self.0[index] = Some(entry);
                    Some(index as u64)
                }
                None => {
                    let index = self.0.len();
                    if index == CODEC_ID_EOS as usize {
                        return None;
                    } else {
                        self.0.push(Some(entry));
                        Some(index as u64)
                    }
                }
            }
        } else {
            None
        }
    }

    // TODO: Test!
    pub fn remove_id(&mut self, id: u64) -> Option<()> {
        let entry = self.0.get_mut(id as usize)?;
        if *entry != None {
            *entry = None;
            Some(())
        } else {
            None
        }
    }

    // TODO: Test!
    pub fn remove_name(&mut self, codec: &CodecEntry) -> Option<()> {
        let index = self
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.as_ref().map(|codec| (i, codec)))
            .find_map(|(i, c)| if c.name == codec.name { Some(i) } else { None })?;

        self.0[index] = None;
        Some(())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_ref(&self) -> &[Option<CodecEntry>] {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Vec<Option<CodecEntry>> {
        self.0
    }

    #[inline]
    fn find_free(&self) -> Option<usize> {
        self.0.iter().enumerate().find_map(|(i, c)| match c {
            Some(_) => None,
            None => Some(i),
        })
    }

    // TODO: Binary search? Explore how this works with sparse vec
    #[inline]
    fn contains_name(&self, name: &[AsciiChar]) -> bool {
        self.0
            .iter()
            .filter_map(Option::as_ref)
            .any(|c| c.name == name)
    }

    pub fn push(&mut self, entry: Option<CodecEntry>) {
        self.0.push(entry);
    }
}

impl From<Vec<Option<CodecEntry>>> for CodecTable {
    fn from(value: Vec<Option<CodecEntry>>) -> Self {
        CodecTable(value)
    }
}

impl Default for CodecTable {
    fn default() -> Self {
        Self(Default::default())
    }
}

// TODO: Impl CodecOwned vs CodecRef?
#[derive(Debug, Clone, PartialEq)]
pub struct CodecEntry {
    pub(crate) version: u16,
    pub(crate) name: Vec<AsciiChar>,
}

impl CodecEntry {
    pub fn new(version: u16, name: impl AsRef<str>) -> Option<CodecEntry> {
        Some(CodecEntry {
            version,
            name: name.as_ref().as_ascii()?.to_owned(),
        })
    }

    pub fn new_ascii(version: u16, name: impl AsRef<[AsciiChar]>) -> CodecEntry {
        CodecEntry {
            version,
            name: name.as_ref().to_owned(),
        }
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

bitflags! {
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct HeaderFlags: u16 {
        // TODO
    }
}

impl HeaderFlags {
    pub fn into_inner(self) -> u16 {
        self.bits()
    }
}

impl From<u16> for HeaderFlags {
    fn from(val: u16) -> Self {
        HeaderFlags(val.into())
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Header {
    pub(crate) version: u16,
    pub(crate) flags: HeaderFlags,
    pub(crate) codec_table: CodecTable,
}

impl Header {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn flags(&self) -> HeaderFlags {
        self.flags
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    fn header_from_raw_table<S>(names: &[Option<S>]) -> Header
    where
        S: AsRef<[u8]> + Clone,
    {
        Header {
            version: 0,
            flags: HeaderFlags::empty(),
            codec_table: CodecTable(
                names
                    .iter()
                    .cloned()
                    .map(|opt_name| {
                        opt_name.map(|name| CodecEntry {
                            version: 0,
                            name: name.as_ref().as_ascii().unwrap().to_vec(),
                        })
                    })
                    .collect(),
            ),
        }
    }

    #[test]
    fn test_header_register() {
        let mut header = header_from_raw_table(&[Some("test_0"), Some("test_1"), Some("test_2")]);
        assert!(header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_0").unwrap())
            .is_none());
        assert!(header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_1").unwrap())
            .is_none());
        assert!(header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_2").unwrap())
            .is_none());

        header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_3").unwrap())
            .unwrap();

        assert_eq!(
            header,
            header_from_raw_table(&[
                Some("test_0"),
                Some("test_1"),
                Some("test_2"),
                Some("test_3")
            ])
        )
    }

    #[test]
    fn test_header_register_fragmented() {
        let mut header =
            header_from_raw_table(&[Some("test_0"), Some("test_1"), None, None, Some("test_4")]);
        assert!(header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_0").unwrap())
            .is_none());
        assert!(header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_1").unwrap())
            .is_none());
        assert!(header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_4").unwrap())
            .is_none());

        header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_2").unwrap())
            .unwrap();
        header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_3").unwrap())
            .unwrap();
        header
            .codec_table
            .register_impl(CodecEntry::new(0, "test_5").unwrap())
            .unwrap();

        assert_eq!(
            header,
            header_from_raw_table(&[
                Some("test_0"),
                Some("test_1"),
                Some("test_2"),
                Some("test_3"),
                Some("test_4"),
                Some("test_5"),
            ])
        )
    }
}
