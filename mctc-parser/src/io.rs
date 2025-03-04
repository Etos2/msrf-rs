use std::io::{Read, Result, Write};

// TODO: Document public api
#[derive(Debug)]
pub struct IoCounter<T> {
    inner: T,
    bytes: usize,
}

impl<T> IoCounter<T> {
    pub fn new(inner: T) -> Self {
        IoCounter { inner, bytes: 0 }
    }

    pub fn count(&self) -> usize {
        self.bytes
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<R: Read> Read for IoCounter<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes = self.inner.read(buf)?;
        self.bytes += bytes;
        Ok(bytes)
    }
}

impl<W: Write> Write for IoCounter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let bytes = self.inner.write(buf)?;
        self.bytes += bytes;
        Ok(bytes)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

// TODO: Document public api
pub trait ReadExt {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_i8(&mut self) -> Result<i8>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_u24(&mut self) -> Result<u32>;
    fn read_i24(&mut self) -> Result<i32>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_u64(&mut self) -> Result<u64>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_f64(&mut self) -> Result<f64>;
    fn read_pvarint(&mut self) -> Result<u64>;
    fn read_chunk(&mut self, bytes: usize) -> Result<Vec<u8>>;
}

#[inline]
fn read_array<const N: usize>(mut rdr: impl Read) -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    rdr.read_exact(&mut buf)?;
    Ok(buf)
}

impl<T: Read> ReadExt for T {
    fn read_u8(&mut self) -> Result<u8> {
        Ok(read_array::<1>(self)?[0])
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(read_array::<1>(self)?[0] as i8)
    }

    fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(read_array::<2>(self)?))
    }

    fn read_i16(&mut self) -> Result<i16> {
        Ok(u16::from_le_bytes(read_array::<2>(self)?) as i16)
    }

    fn read_u24(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf[..3])?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_i24(&mut self) -> Result<i32> {
        const SHIFT: u32 = 8;
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf[..3])?;
        Ok((u32::from_le_bytes(buf) << SHIFT) as i32 >> SHIFT)
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(read_array::<4>(self)?))
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(u32::from_le_bytes(read_array::<4>(self)?) as i32)
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(read_array::<8>(self)?))
    }

    fn read_i64(&mut self) -> Result<i64> {
        Ok(u64::from_le_bytes(read_array::<8>(self)?) as i64)
    }

    fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(read_array::<4>(self)?))
    }

    fn read_f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(read_array::<8>(self)?))
    }

    // TODO: Optimise + Bench
    fn read_pvarint(&mut self) -> Result<u64> {
        let tag = self.read_u8()?;
        let len = tag.trailing_zeros() as usize;
        let mut data = [0; 8];
        self.read_exact(&mut data[..len])?;

        Ok(
            // Catch tag w/data (0bXXXXXXX1...0bX100000)
            if len < 7 {
                let remainder = tag >> (len + 1); // Remove guard bit
                (u64::from_le_bytes(data) << (7 - len)) + remainder as u64
            // Catch tag w/o data (0b1000000 + 0b00000000)
            } else {
                u64::from_le_bytes(data)
            },
        )
    }

    fn read_chunk(&mut self, bytes: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; bytes];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}

pub trait WriteExt {
    fn write_u8(&mut self, data: u8) -> Result<()>;
    fn write_i8(&mut self, data: i8) -> Result<()>;
    fn write_u16(&mut self, data: u16) -> Result<()>;
    fn write_i16(&mut self, data: i16) -> Result<()>;
    fn write_u24(&mut self, data: u32) -> Result<()>;
    fn write_i24(&mut self, data: i32) -> Result<()>;
    fn write_u32(&mut self, data: u32) -> Result<()>;
    fn write_i32(&mut self, data: i32) -> Result<()>;
    fn write_u64(&mut self, data: u64) -> Result<()>;
    fn write_i64(&mut self, data: i64) -> Result<()>;
    fn write_f32(&mut self, data: f32) -> Result<()>;
    fn write_f64(&mut self, data: f64) -> Result<()>;
    fn write_pvarint(&mut self, data: u64) -> Result<()>;
    fn write_chunk(&mut self, data: &[u8]) -> Result<()>;
}

impl<T: Write> WriteExt for T {
    fn write_u8(&mut self, data: u8) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_i8(&mut self, data: i8) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_u16(&mut self, data: u16) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_i16(&mut self, data: i16) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    // TODO: out of bounds check?
    fn write_u24(&mut self, data: u32) -> Result<()> {
        self.write_all(&data.to_le_bytes()[..3])
    }

    // TODO: out of bounds check?
    fn write_i24(&mut self, data: i32) -> Result<()> {
        self.write_all(&data.to_le_bytes()[..3])
    }

    fn write_u32(&mut self, data: u32) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_i32(&mut self, data: i32) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_u64(&mut self, data: u64) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_i64(&mut self, data: i64) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_f32(&mut self, data: f32) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    fn write_f64(&mut self, data: f64) -> Result<()> {
        self.write_all(&data.to_le_bytes())
    }

    // TODO: Optimise + Bench
    // TODO: Rewrite (data << 1 + 1 will simplify logic for non-full u64)
    fn write_pvarint(&mut self, data: u64) -> Result<()> {
        let mut buf = [0u8; 9];
        let zeros = data.leading_zeros();

        // Catch empty u64
        if zeros == 64 {
            self.write_all(&[0x01])?;
        // Catch full u64
        } else if zeros == 0 {
            buf[1..].copy_from_slice(&data.to_le_bytes());
            self.write_all(&buf)?;
        // Catch var u64
        } else {
            let bytes = 8 - ((zeros - 1) / 7) as usize;
            let data = data << bytes + 1;
            buf[..=bytes].copy_from_slice(&data.to_le_bytes()[..=bytes]);
            buf[0] |= if bytes >= 8 { 0 } else { 0x01 << bytes };
            self.write_all(&buf[..=bytes])?;
        }

        Ok(())
    }

    fn write_chunk(&mut self, data: &[u8]) -> Result<()> {
        self.write_all(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_write_u24() {
        let input = 1235678;
        let mut io = Cursor::new([0; 3]);
        io.write_u24(input).unwrap();
        io.set_position(0);
        let output = io.read_u24().unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn read_write_i24() {
        let input = -1235678;
        let mut io = Cursor::new([0; 3]);
        io.write_i24(input).unwrap();
        io.set_position(0);
        let output = io.read_i24().unwrap();
        assert_eq!(input, output);
    }
}
