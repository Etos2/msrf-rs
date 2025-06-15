use crate::{data::Header, io::util::Guard, MAGIC_BYTES};

pub fn ref_header() -> Header {
    Header { version: (1, 2) }
}

pub fn ref_header_bytes() -> Vec<u8> {
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&MAGIC_BYTES); // Magic bytes
    data.extend_from_slice(&0b111_u8.to_le_bytes()); // Length Pvarint(3)
    data.extend_from_slice(&1_u8.to_le_bytes()); // Version (Major)
    data.extend_from_slice(&2_u8.to_le_bytes()); // Version (Minor)
    data.extend_from_slice(&Guard::new(&3u8.to_le_bytes()).get().to_le_bytes()); // Guard

    data
}
