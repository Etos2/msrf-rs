use crate::{data::Header, io::Guard, MAGIC_BYTES};

pub fn ref_header() -> Header {
    Header { version: (0, 0) }
}

pub fn ref_header_bytes() -> Vec<u8> {
    let mut data = Vec::new();

    // FIXME: PVarint length
    // Header
    data.extend_from_slice(&MAGIC_BYTES); // Magic bytes
    data.extend_from_slice(&43_u64.to_le_bytes()); // Length
    data.extend_from_slice(&0_u8.to_le_bytes()); // Version (Major)
    data.extend_from_slice(&0_u8.to_le_bytes()); // Version (Minor)
    data.extend_from_slice(&Guard::new(&43_u32.to_le_bytes()).get().to_le_bytes()); // Guard

    data
}
