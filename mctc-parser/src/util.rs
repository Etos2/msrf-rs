use std::io::{Read, Result as IoResult, Write};

use crate::error::{PError, PResult};

pub trait ReadExt {
    fn read_null(&mut self) -> PResult<()>;
    fn read_u8(&mut self) -> PResult<u8>;
    fn read_u16(&mut self) -> PResult<u16>;
    fn read_pv(&mut self) -> PResult<(u64, usize)>;
    fn read_vec(&mut self, bytes: usize) -> PResult<Vec<u8>>;
    fn read_equals(&mut self, comp: &[u8]) -> PResult<()>;
}

#[inline]
fn read_array<const N: usize>(mut rdr: impl Read) -> PResult<[u8; N]> {
    let mut buf = [0u8; N];
    rdr.read_exact(&mut buf)?;
    Ok(buf)
}

impl<R: Read> ReadExt for R {
    #[inline]
    fn read_null(&mut self) -> PResult<()> {
        let data = read_array::<1>(self)?[0];
        if data == 0x00 {
            Ok(())
        } else {
            Err(PError::MismatchBytes {
                found: vec![data; 1],
                expected: vec![0x00; 1],
            })
        }
    }

    #[inline]
    fn read_u8(&mut self) -> PResult<u8> {
        Ok(read_array::<1>(self)?[0])
    }

    #[inline]
    fn read_u16(&mut self) -> PResult<u16> {
        Ok(u16::from_le_bytes(read_array(self)?))
    }

    #[inline]
    fn read_pv(&mut self) -> PResult<(u64, usize)> {
        let tag = self.read_u8()?;
        let len = tag.trailing_zeros() as usize;
        let mut data = [0; 8];

        Ok(
            // Catch single byte varint
            if len == 0 {
                ((tag >> 1) as u64, 1)
            // Catch tag w/data (0bXXXXXX10...0bX100000)
            } else if len < 7 {
                let remainder = tag >> (len + 1); // Remove bit then shift
                self.read_exact(&mut data[..len])?;
                (
                    (u64::from_le_bytes(data) << (7 - len)) + remainder as u64,
                    len + 1,
                )
            // Catch tag w/o data (0b1000000 + 0b00000000)
            } else {
                self.read_exact(&mut data[..len])?;
                (u64::from_le_bytes(data), len + 1)
            },
        )
    }

    #[inline]
    fn read_vec(&mut self, bytes: usize) -> PResult<Vec<u8>> {
        let mut buf = vec![0; bytes];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    fn read_equals(&mut self, expect: &[u8]) -> PResult<()> {
        let mut buf = vec![0; expect.len()];
        self.read_exact(&mut buf)?;
        if buf != *expect {
            Err(PError::MismatchBytes {
                found: buf.to_vec(),
                expected: expect.to_vec(),
            })
        } else {
            Ok(())
        }
    }
}

pub trait WriteExt {
    fn write_null(&mut self) -> IoResult<()>;
    fn write_u8(&mut self, data: u8) -> IoResult<()>;
    fn write_u16(&mut self, data: u16) -> IoResult<()>;
    fn write_pv(&mut self, data: u64) -> IoResult<()>;
}

impl<W: Write> WriteExt for W {
    #[inline]
    fn write_null(&mut self) -> IoResult<()> {
        Ok(self.write_all(&[0x00])?)
    }

    #[inline]
    fn write_u8(&mut self, data: u8) -> IoResult<()> {
        Ok(self.write_all(&data.to_le_bytes())?)
    }

    #[inline]
    fn write_u16(&mut self, data: u16) -> IoResult<()> {
        Ok(self.write_all(&data.to_le_bytes())?)
    }

    #[inline]
    fn write_pv(&mut self, data: u64) -> IoResult<()> {
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
            buf[0] |= if bytes >= u8::BITS as usize {
                0
            } else {
                0x01 << bytes
            };
            self.write_all(&buf[..=bytes])?;
        }

        Ok(())
    }
}
