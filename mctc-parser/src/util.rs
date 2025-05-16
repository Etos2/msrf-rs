use std::ascii::Char as AsciiChar;

// SAFETY: bytes must be valid ascii
unsafe fn to_ascii_slice(bytes: &[u8]) -> &[AsciiChar] {
    // SAFETY: AsciiChar is repr(u8)
    assert_eq!(std::mem::size_of::<AsciiChar>(), std::mem::size_of::<u8>());
    unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const AsciiChar, bytes.len()) }
}

fn slice_is_ascii(bytes: &[u8]) -> bool {
    bytes.iter().all(u8::is_ascii)
}

pub trait AsciiCharExt {
    fn new_checked<'a>(bytes: &'a [u8]) -> Option<&'a [AsciiChar]>;
    fn new<'a>(bytes: &'a [u8]) -> &'a [AsciiChar];
}

impl AsciiCharExt for [AsciiChar] {
    fn new_checked<'a>(bytes: &'a [u8]) -> Option<&'a [AsciiChar]> {
        if slice_is_ascii(bytes) {
            Some(unsafe { to_ascii_slice(bytes) })
        } else {
            None
        }
    }

    fn new<'a>(bytes: &'a [u8]) -> &'a [AsciiChar] {
        if slice_is_ascii(bytes) {
            unsafe { to_ascii_slice(bytes) }
        } else {
            panic!("invalid ascii")
        }
    }
}
