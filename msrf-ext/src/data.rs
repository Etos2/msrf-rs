#[non_exhaustive]
#[repr(u8)]
pub enum Record {
    SourceDefinition(SourceDefinition) = 0x00,
    SourceRemove(SourceRemove) = 0x01,
}

pub struct SourceDefinition {
    id: u64,
    version: (u8, u8),
    name: String,
}

pub struct SourceRemove {
    id: u64
}
