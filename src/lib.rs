use std::io::Read;

use msrf_io::error::CodecResult;

use crate::{
    codec::{
        AnySerialiser, RawSerialiser,
        constants::{HEADER_LEN, RECORD_META_MIN_LEN},
    },
    data::{Header, RecordMeta},
};

pub mod codec;
pub mod data;
pub mod reader;

const VERSION_TABLE: &'static [(u8, u8)] = &[(0, 0)];
const CURRENT_VERSION: (u8, u8) = (0, 0);