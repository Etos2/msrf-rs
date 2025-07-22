pub const ID_SOURCE_ADD: u16 = 0x00;
pub const ID_SOURCE_REMOVE: u16 = 0x01;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdd {
    pub(crate) id: u64,
    pub(crate) version: (u8, u8),
    pub(crate) name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRemove {
    pub(crate) id: u64,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum Record {
    SourceAdd(SourceAdd) = ID_SOURCE_ADD as u8,
    SourceRemove(SourceRemove) = ID_SOURCE_REMOVE as u8,
}

impl Record {
    pub fn type_id(&self) -> u16 {
        match self {
            Record::SourceAdd(_) => ID_SOURCE_ADD,
            Record::SourceRemove(_) => ID_SOURCE_REMOVE,
        }.into()
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
