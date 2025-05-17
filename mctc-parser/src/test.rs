use crate::{
    data::{CodecEntry, CodecTable, Header, HeaderFlags},
    MAGIC_BYTES,
};

pub fn ref_header() -> Header {
    Header {
        version: 0,
        flags: HeaderFlags::empty(),
        codec_table: CodecTable::from(vec![
            Some(CodecEntry {
                version: 1,
                name: b"TEST".as_ascii().unwrap().to_vec(),
            }),
            None,
            Some(CodecEntry {
                version: u16::MAX,
                name: b"SomeLongStringThatIsLong".as_ascii().unwrap().to_vec(),
            }),
        ]),
    }
}

pub fn ref_header_bytes() -> Vec<u8> {
    let mut data = Vec::new();
    // Header
    data.extend_from_slice(&MAGIC_BYTES); // Magic bytes
    data.extend_from_slice(&43_u32.to_le_bytes()); // Length
    data.extend_from_slice(&0_u16.to_le_bytes()); // Version
    data.extend_from_slice(&0_u16.to_le_bytes()); // Flags

    // Codec Table
    data.extend_from_slice(&3_u16.to_le_bytes()); // Codec Entries

    // Codec Entry 1
    data.extend_from_slice(&7_u8.to_le_bytes()); // Length
    data.extend_from_slice(&1_u16.to_le_bytes()); // Version
    data.extend_from_slice(b"TEST"); // Name
    data.extend_from_slice(&0_u8.to_le_bytes()); // Guard

    // Codec Entry 2 (empty)
    data.extend_from_slice(&0_u8.to_le_bytes()); // Length

    // Codec Entry 3
    data.extend_from_slice(&27_u8.to_le_bytes()); // Length
    data.extend_from_slice(&(u16::MAX).to_le_bytes()); // Version
    data.extend_from_slice(b"SomeLongStringThatIsLong"); // Name
    data.extend_from_slice(&0_u8.to_le_bytes()); // Guard

    data
}
