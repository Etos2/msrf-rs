use std::{
    collections::{BTreeMap, btree_map::Entry},
    num::NonZeroU16,
};

pub mod codec;
pub mod error;
pub mod reader;
pub mod writer;

pub const MSRF_EXT_NAME: &str = "msrf-ext";
pub const MSRF_EXT_MAP_ID: u16 = 0x00;
pub const MSRF_EXT_VERSION: u16 = 0x00;

pub const ID_SOURCE_ADD: u16 = 0x00;
pub const ID_SOURCE_REMOVE: u16 = 0x01;

pub trait AssignedId {
    const TYPE_ID: u16;
    fn type_id(&self) -> u16 {
        Self::TYPE_ID
    }
}

// TODO: &str
#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdd {
    pub(crate) id: u16,
    pub(crate) version: u16,
    pub(crate) name: String,
}

impl SourceAdd {
    pub fn new(id: u16, version: u16, name: impl Into<String>) -> SourceAdd {
        SourceAdd { id, version, name: name.into() }
    }
}

impl AssignedId for SourceAdd {
    const TYPE_ID: u16 = ID_SOURCE_ADD;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRemove {
    pub(crate) id: u16,
}

impl SourceRemove {
    pub fn new(id: u16) -> SourceRemove {
        SourceRemove { id }
    }
}

impl AssignedId for SourceRemove {
    const TYPE_ID: u16 = ID_SOURCE_REMOVE;
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[repr(u16)]
pub enum Record {
    SourceAdd(SourceAdd) = SourceAdd::TYPE_ID,
    SourceRemove(SourceRemove) = SourceRemove::TYPE_ID,
}

impl Record {
    pub fn type_id(&self) -> u16 {
        match self {
            Record::SourceAdd(_) => SourceAdd::TYPE_ID,
            Record::SourceRemove(_) => SourceRemove::TYPE_ID,
        }
    }
}

impl From<Record> for u16 {
    fn from(value: Record) -> Self {
        value.type_id()
    }
}

impl From<SourceAdd> for Record {
    fn from(value: SourceAdd) -> Self {
        Record::SourceAdd(value)
    }
}

impl From<SourceRemove> for Record {
    fn from(value: SourceRemove) -> Self {
        Record::SourceRemove(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct Source {
    name: String,
    version: u16,
}

impl Source {
    fn new(name: impl Into<String>, version: u16) -> Source {
        Source { name: name.into(), version }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn version(&self) -> u16 {
        self.version
    }
}

#[derive(Debug)]
pub struct SourceRegistrar {
    map: BTreeMap<u16, Source>,
    next_id: NonZeroU16,
}

impl SourceRegistrar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: impl Into<String> + AsRef<str>, version: u16) -> Result<u16, u16> {
        if let Some(id) = self.get_by_source(name.as_ref()) {
            return Err(id);
        }

        let id = self.next_id.get();
        self.map.insert(id, Source::new(name.into(), version));
        self.next_id = self.next_free_id();
        Ok(id)
    }

    pub fn register_root(&mut self, name: impl Into<String>, version: u16) -> Option<&str> {
        match self.map.entry(0) {
            Entry::Occupied(o) => Some(o.into_mut().name()),
            Entry::Vacant(v) => {
                v.insert(Source::new(name, version));
                None
            }
        }
    }

    pub fn register_existing<'a>(
        &mut self,
        id: NonZeroU16,
        name: impl Into<String>,
        version: u16,
    ) -> Option<&str> {
        // TODO: HACK! Borrow checker stops us from placing this in the `Vacant` branch below (need `self.map.keys` when already mutably borrowed `self.map`).
        if self.next_id == id {
            // TODO: HACK! `Self.next_id`` cannot equal `id` if already occupied, increment to fake incoming insertion
            self.next_id = NonZeroU16::new(self.next_id.get() + 1).expect("N + 1 is always > 0");
            self.next_id = self.next_free_id();
        }

        match self.map.entry(id.get()) {
            Entry::Occupied(o) => Some(o.into_mut().name()),
            Entry::Vacant(v) => {
                v.insert(Source::new(name, version));
                None
            }
        }
    }

    pub fn remove_by_id(&mut self, id: u16) -> Option<Source> {
        self.map.remove(&id).inspect(|_| {
            if let Some(new_id) = NonZeroU16::new(id)
                && self.next_id > new_id
            {
                self.next_id = new_id;
            }
        })
    }

    pub fn remove_by_source(&mut self, source: impl AsRef<str>) -> Option<u16> {
        self.get_by_source(source).inspect(|id| {
            let _ = self.remove_by_id(*id);
        })
    }

    pub fn get_by_id(&self, id: u16) -> Option<&Source> {
        self.map.get(&id)
    }

    pub fn get_by_source(&self, source: impl AsRef<str>) -> Option<u16> {
        let name_rhs = source.as_ref();
        self.map
            .iter()
            .find(|(_, src)| src.name() == name_rhs)
            .map(|(id, _)| *id)
    }

    pub fn sources(&self) -> impl Iterator<Item = (u16, &str, u16)> {
        self.map.iter().map(|(id, src)| (*id, src.name(), src.version()))
    }

    fn next_free_id(&self) -> NonZeroU16 {
        let mut id_candidate = self.next_id.get();
        for id in self
            .map
            .keys()
            .copied()
            .skip_while(|id| *id < self.next_id.get())
        {
            if id != id_candidate {
                break;
            }
            // TODO: Return option instead?
            id_candidate = id_candidate.checked_add(1).expect("id overflow");
        }
        NonZeroU16::new(id_candidate).expect("self.next_id() was somehow zero")
    }
}

impl Default for SourceRegistrar {
    fn default() -> Self {
        Self {
            map: Default::default(),
            next_id: NonZeroU16::new(1).unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::u16;

    use super::*;

    
    const ROOT_A: &str = "msrf-ext";
    const ROOT_B: &str = "arbitrary-ext";
    const SOURCE_A: &str = "pxls-space-ext";
    const SOURCE_B: &str = "canvas-ext";
    const SOURCE_C: &str = "r-place-ext";

    #[test]
    fn source_registrar_register() {
        let mut registrar = SourceRegistrar::new();

        // Register SOURCE_A & SOURCE_B
        assert_eq!(registrar.register(SOURCE_A, 123), Ok(1));
        assert_eq!(registrar.register(SOURCE_B, 324), Ok(2));
        assert_eq!(registrar.get_by_id(1), Some(&Source::new(SOURCE_A, 123)));
        assert_eq!(registrar.get_by_id(2), Some(&Source::new(SOURCE_B, 324)));

        // Register SOURCE_C after removing SOURCE_A
        assert_eq!(registrar.remove_by_id(1), Some(Source::new(SOURCE_A, 123)));
        assert_eq!(registrar.register(SOURCE_C, 0), Ok(1));
        assert_eq!(registrar.get_by_id(1), Some(&Source::new(SOURCE_C, 0)));
        assert_eq!(registrar.get_by_id(2), Some(&Source::new(SOURCE_B, 324)));
    }

    #[test]
    fn source_registrar_iter() {
        let mut registrar = SourceRegistrar::new();

        assert_eq!(registrar.register_root(ROOT_A, 567), None);
        assert_eq!(registrar.register(SOURCE_A, 123), Ok(1));
        assert_eq!(registrar.register(SOURCE_B, 324), Ok(2));
        assert_eq!(registrar.register(SOURCE_C, 0), Ok(3));

        {
            let mut iter = registrar.sources();
            assert_eq!(iter.next(), Some((0, ROOT_A, 567)));
            assert_eq!(iter.next(), Some((1, SOURCE_A, 123)));
            assert_eq!(iter.next(), Some((2, SOURCE_B, 324)));
            assert_eq!(iter.next(), Some((3, SOURCE_C, 0)));
            assert_eq!(iter.next(), None);
        }

        assert_eq!(registrar.remove_by_id(1), Some(Source::new(SOURCE_A, 123)));
        assert_eq!(registrar.remove_by_source(SOURCE_B), Some(2));

        {
            let mut iter = registrar.sources();
            assert_eq!(iter.next(), Some((0, ROOT_A, 567)));
            assert_eq!(iter.next(), Some((3, SOURCE_C, 0)));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn source_registrar_register_root() {
        let mut registrar = SourceRegistrar::new();

        // Register ROOT_A
        assert_eq!(registrar.register_root(ROOT_A, 567), None);
        assert_eq!(registrar.register_root(ROOT_B, 890), Some(ROOT_A));
        assert_eq!(registrar.get_by_id(0), Some(&Source::new(ROOT_A, 567)));

        // Register ROOT_B by removing ROOT_A by ID
        assert_eq!(registrar.remove_by_id(0), Some(Source::new(ROOT_A, 567)));
        assert_eq!(registrar.register_root(ROOT_B, 890), None);
        assert_eq!(registrar.get_by_id(0), Some(&Source::new(ROOT_B, 890)));

        // Register ROOT_A by removing ROOT_B by Source
        assert_eq!(registrar.remove_by_source(ROOT_B), Some(0));
        assert_eq!(registrar.register_root(ROOT_A, 567), None);
        assert_eq!(registrar.get_by_id(0), Some(&Source::new(ROOT_A, 567)));
    }

    #[test]
    fn source_registrar_id_selection() {
        let mut registrar = SourceRegistrar::new();

        // Check ordinary sequential ID
        assert_eq!(registrar.next_free_id().get(), 1);
        assert_eq!(registrar.register(ROOT_A, 567), Ok(1));
        assert_eq!(registrar.next_free_id().get(), 2);
        assert_eq!(registrar.register(ROOT_B, 890), Ok(2));
        assert_eq!(registrar.next_free_id().get(), 3);

        // Check invalid self.next_id handling (this shouldn't occur, but is still useful)
        registrar.next_id = NonZeroU16::new(1).unwrap();
        assert_eq!(registrar.next_free_id().get(), 3);
        registrar.next_id = NonZeroU16::new(u16::MAX).unwrap();
        assert_eq!(registrar.next_free_id().get(), u16::MAX);

        // Check removal logic
        assert_eq!(registrar.remove_by_source(ROOT_B), Some(2));
        assert_eq!(registrar.next_free_id().get(), 2);
        assert_eq!(registrar.remove_by_id(1), Some(Source::new(ROOT_A, 576)));
        assert_eq!(registrar.next_free_id().get(), 1);
    }
}
