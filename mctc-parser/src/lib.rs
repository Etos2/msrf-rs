pub mod data;
pub mod error;
pub mod reader;

const MAGIC_BYTES: [u8; 4] = *b"MCTC";
// TODO: Support
const CODEC_ID_EOS: u16 = 0xFFFF;

pub struct DefaultOptions {}

impl Default for DefaultOptions {
    fn default() -> Self {
        Self {}
    }
}
