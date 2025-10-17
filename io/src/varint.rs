const TAG_WITH_DATA_LEN: usize = 7;

pub fn len(tag: u8) -> usize {
    tag.trailing_zeros() as usize + 1
}

pub fn to_le_bytes(val: u64) -> [u8; 9] {
    let zeros = val.leading_zeros();
    let mut buf = [0; 9];

    // Catch empty u64
    if zeros == 64 {
        buf[0] = 0x01;
        buf
    // Catch full u64
    } else if zeros == 0 {
        buf[1..].copy_from_slice(&val.to_le_bytes());
        buf
    // Catch var u64
    } else {
        let bytes = 8 - ((zeros - 1) / TAG_WITH_DATA_LEN as u32) as usize;
        let data = val << (bytes + 1);
        buf[..=bytes].copy_from_slice(&data.to_le_bytes()[..=bytes]);
        buf[0] |= if bytes >= 8 { 0 } else { 0x01 << bytes };
        buf
    }
}

// PANIC: Will panic when data.is_empty()
pub fn from_le_bytes(data: &[u8]) -> u64 {
    let mut out = [0; 8];
    let len = len(data[0]);

    if len <= TAG_WITH_DATA_LEN {
        out[..len].copy_from_slice(&data[..len]);
        u64::from_le_bytes(out) >> len
    } else {
        out[..len - 1].copy_from_slice(&data[1..len]);
        u64::from_le_bytes(out)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialise_pvarint() {
        fn harness(val: u64) {
            let var = to_le_bytes(val);
            let dec = from_le_bytes(&var);
            assert_eq!(
                val, dec,
                "failed to manually encode/decode {val:X} != {dec:X}"
            );
        }

        harness(0x00); // 2^7-1
        harness(0x7F); // 2^7-1
        harness(0x3FFF); // 2^14-1
        harness(0x1FFFFF); // 2^21-1
        harness(0xFFFFFFF); // 2^28-1
        harness(0x7FFFFFFFF); // 2^35-1
        harness(0x3FFFFFFFFFF); // 2^42-1
        harness(0x1FFFFFFFFFFFF); // 2^49-1
        harness(0xFFFFFFFFFFFFFF); // 2^56-1
        harness(0xFFFFFFFFFFFFFFFF); // 2^64
    }
}
