use std::io::Read;

use msrf_io::error::CodecResult;

use crate::{
    codec::constants::{HEADER_LEN, RECORD_META_MIN_LEN},
    data::{Header, RecordMeta},
};

pub mod codec;
pub mod data;
pub mod reader;

const CURRENT_VERSION: u16 = 0;
