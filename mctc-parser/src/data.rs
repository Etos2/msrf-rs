use bitflags::bitflags;

use crate::{CODEC_ID_EOS, CURRENT_VERSION};

// Codecs are stored in a "sparse" vec
pub type CodecTable = Vec<Option<Codec>>;

// TODO: Impl CodecOwned vs CodecRef?
#[derive(Debug, Clone, PartialEq)]
pub struct Codec {
    pub(crate) version: u16,
    pub(crate) name: String,
}

impl Codec {
    pub fn new(version: u16, name: impl Into<String>) -> Codec {
        Codec {
            version,
            name: name.into(),
        }
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

bitflags! {
    #[derive(Debug, Default, Copy, Clone, PartialEq)]
    pub struct HeaderFlags: u16 {
        // TODO
    }
}

impl From<u16> for HeaderFlags {
    fn from(val: u16) -> Self {
        HeaderFlags(val.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub(crate) version: u16,
    pub(crate) flags: HeaderFlags,
    pub(crate) codec_table: CodecTable,
}

impl Header {
    // TODO: Better ID solution?
    pub fn register_codec(&mut self, codec: Codec) -> Option<u64> {
        if !self.contains_name(&codec.name) {
            match self.find_free() {
                Some(index) => {
                    self.codec_table[index] = Some(codec);
                    Some(index as u64)
                }
                None => {
                    let index = self.codec_table.len();
                    if index == CODEC_ID_EOS as usize {
                        return None;
                    } else {
                        self.codec_table.push(Some(codec));
                        Some(index as u64)
                    }
                }
            }
        } else {
            None
        }
    }

    // TODO: Test!
    pub fn remove_codec_id(&mut self, id: u64) -> Option<()> {
        let entry = self.codec_table.get_mut(id as usize)?;
        if *entry != None {
            *entry = None;
            Some(())
        } else {
            None
        }
    }

    // TODO: Test!
    pub fn remove_codec(&mut self, codec: &Codec) -> Option<()> {
        let index = self
            .codec_table
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.as_ref().map(|codec| (i, codec)))
            .find_map(|(i, c)| if c.name == codec.name { Some(i) } else { None })?;

        self.codec_table[index] = None;
        Some(())
    }

    fn find_free(&self) -> Option<usize> {
        self.codec_table
            .iter()
            .enumerate()
            .find_map(|(i, c)| match c {
                Some(_) => None,
                None => Some(i),
            })
    }

    fn contains_name(&self, name: &str) -> bool {
        self.codec_table
            .iter()
            .filter_map(Option::as_ref)
            .any(|c| c.name == name)
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn flags(&self) -> HeaderFlags {
        self.flags
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            flags: Default::default(),
            codec_table: Default::default(),
        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub(crate) codec_id: u64,
    pub(crate) type_id: u64,
    pub(crate) val: Option<Box<[u8]>>,
}

impl Record {
    pub fn from_slice(codec_id: u64, type_id: u64, val: &[u8]) -> Self {
        Record {
            codec_id,
            type_id,
            val: Some(Box::from(val)),
        }
    }

    pub fn from_box(codec_id: u64, type_id: u64, val: Box<[u8]>) -> Self {
        Record {
            codec_id,
            type_id,
            val: Some(val),
        }
    }

    pub fn new_eos() -> Self {
        Record {
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

    pub fn len(&self) -> usize {
        self.val.as_ref().map(|data| data.len()).unwrap_or(0)
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.val.as_deref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn header_from_raw_table<S>(names: &[Option<S>]) -> Header
    where
        S: Into<String> + Clone,
    {
        Header {
            version: 0,
            flags: HeaderFlags::empty(),
            codec_table: names
                .iter()
                .cloned()
                .map(|opt_name| {
                    opt_name.map(|name| Codec {
                        version: 0,
                        name: name.into(),
                    })
                })
                .collect(),
        }
    }

    #[test]
    fn test_header_register() {
        let mut header = header_from_raw_table(&[Some("test_0"), Some("test_1"), Some("test_2")]);
        assert!(header.register_codec(Codec::new(0, "test_0")).is_none());
        assert!(header.register_codec(Codec::new(0, "test_1")).is_none());
        assert!(header.register_codec(Codec::new(0, "test_2")).is_none());

        header.register_codec(Codec::new(0, "test_3")).unwrap();

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
        assert!(header.register_codec(Codec::new(0, "test_0")).is_none());
        assert!(header.register_codec(Codec::new(0, "test_1")).is_none());
        assert!(header.register_codec(Codec::new(0, "test_4")).is_none());

        header.register_codec(Codec::new(0, "test_2")).unwrap();
        header.register_codec(Codec::new(0, "test_3")).unwrap();
        header.register_codec(Codec::new(0, "test_5")).unwrap();

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
