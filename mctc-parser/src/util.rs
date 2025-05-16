use std::ascii::Char as AsciiChar;

fn to_ascii_slice(bytes: &[u8]) -> &[AsciiChar] {
    // SAFETY: AsciiChar is repr(u8)
    assert_eq!(std::mem::size_of::<AsciiChar>(), std::mem::size_of::<u8>());
    unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const AsciiChar, bytes.len()) }
}

fn slice_is_ascii(bytes: &[u8]) -> bool {
    bytes.iter().all(u8::is_ascii)
}

pub trait AsciiCharExt {
    fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a [AsciiChar]>;
    fn from_bytes_owned(bytes: &[u8]) -> Option<Vec<AsciiChar>>;
}

impl AsciiCharExt for [AsciiChar] {
    fn from_bytes<'a>(bytes: &'a [u8]) -> Option<&'a [AsciiChar]> {
        if slice_is_ascii(bytes) {
            Some(to_ascii_slice(bytes))
        } else {
            None
        }
    }

    fn from_bytes_owned(bytes: &[u8]) -> Option<Vec<AsciiChar>> {
        if slice_is_ascii(bytes) {
            Some(to_ascii_slice(bytes).to_owned())
        } else {
            None
        }
    }
}
