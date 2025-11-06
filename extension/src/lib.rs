pub mod codec;
pub mod error;
pub mod reader;

pub const MSRF_EXT_MAP_ID: u16 = 0x00;
pub const ID_SOURCE_ADD: u16 = 0x00;
pub const ID_SOURCE_REMOVE: u16 = 0x01;

pub trait AssignedId {
    const TYPE_ID: u16;
    fn type_id(&self) -> u16 {
        Self::TYPE_ID
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdd {
    pub(crate) id: u16,
    pub(crate) version: u16,
    pub(crate) name: String,
}

impl AssignedId for SourceAdd {
    const TYPE_ID: u16 = ID_SOURCE_ADD;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRemove {
    pub(crate) id: u16,
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