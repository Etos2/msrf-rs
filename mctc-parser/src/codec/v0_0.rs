use crate::{
    codec::{
        RawDeserialiser, RawSerialiser,
        constants::{HEADER_CONTENTS, HEADER_LEN, MAGIC_BYTES, RECORD_EOS, RECORD_META_MIN_LEN},
        util::{PVarint, extract, insert},
    },
    data::{Header, RecordMeta},
    error::{CodecError, CodecResult},
};

pub struct Serialiser {}

impl RawSerialiser for Serialiser {
    fn serialise_header(&self, buf: &mut [u8], header: &Header) -> CodecResult<usize> {
        let len = buf.len();
        let mut buf = buf;

        if HEADER_LEN > len {
            return Err(CodecError::Needed(HEADER_LEN - len));
        }

        insert(&mut buf, &MAGIC_BYTES)?;
        insert(&mut buf, &PVarint::new(HEADER_CONTENTS as u64).as_slice())?;
        insert(&mut buf, &[header.version.0])?;
        insert(&mut buf, &[header.version.1])?;
        insert(&mut buf, &[0x00])?;

        Ok(len - buf.len())
    }

    fn serialise_record_meta(&self, buf: &mut [u8], meta: &RecordMeta) -> CodecResult<usize> {
        let len = buf.len();
        let mut buf = buf;

        let record_len = meta.length;
        insert(&mut buf, &PVarint::new(record_len).as_slice())?;
        if record_len < RECORD_META_MIN_LEN {
            insert(&mut buf, &meta.source_id.to_le_bytes())?;
            insert(&mut buf, &meta.type_id.to_le_bytes())?;
        } else if record_len != RECORD_EOS {
            return Err(CodecError::RecordLength(record_len));
        }

        Ok(len - buf.len())
    }
}

pub struct Deserialiser {}

impl RawDeserialiser for Deserialiser {
    fn deserialise_header(&self, buf: &[u8]) -> CodecResult<(Header, usize)> {
        let len = buf.len();
        let mut buf = buf;

        if HEADER_LEN > buf.len() {
            return Err(CodecError::Needed(HEADER_LEN - buf.len()));
        }

        // SAFETY: [u8; 4].len() == 4
        let magic_bytes = extract(&mut buf, 4)?.try_into().unwrap();
        if magic_bytes != MAGIC_BYTES {
            return Err(CodecError::MagicByte(magic_bytes));
        }

        // TODO: Better API
        let pv_len = PVarint::len_from_tag(buf[0]); // TODO: Unchecked index
        let length = PVarint::decode(extract(&mut buf, pv_len)?).unwrap(); // TODO: Safety analysis
        let major = extract(&mut buf, 1)?[0];
        let minor = extract(&mut buf, 1)?[0];

        // Skip additional header data
        if let Some(unknown) = (length).checked_sub(HEADER_CONTENTS) {
            buf = &buf[unknown as usize..]; // TODO: Check for truncation
        }

        let guard = extract(&mut buf, 1)?[0];
        if guard != 0 {
            return Err(CodecError::Guard(guard));
        }

        Ok((
            Header {
                version: (major, minor),
            },
            len - buf.len(),
        ))
    }

    fn deserialise_record_meta(&self, buf: &[u8]) -> CodecResult<(RecordMeta, usize)> {
        let len = buf.len();
        let mut buf = buf;

        // TODO: Better API
        let pv_len = PVarint::len_from_tag(buf[0]); // TODO: Unchecked index
        let length = PVarint::decode(extract(&mut buf, pv_len)?).unwrap(); // TODO: Safety analysis
        match length {
            // 0 = End Of Stream indicator
            0 => Ok((RecordMeta::new_eos(), len - buf.len())),
            // Len invariance, must be long enough to contain IDs
            1..RECORD_META_MIN_LEN => Err(CodecError::RecordLength(length)),
            // Contains contents + zero/some data
            RECORD_META_MIN_LEN.. => {
                // SAFETY: [u8; 2].len() == 2
                let source_id = extract(&mut buf, 2)?.try_into().unwrap();
                let source_id = u16::from_le_bytes(source_id);
                // SAFETY: [u8; 2].len() == 2
                let type_id = extract(&mut buf, 2)?.try_into().unwrap();
                let type_id = u16::from_le_bytes(type_id);

                Ok((
                    RecordMeta {
                        length,
                        source_id,
                        type_id,
                    },
                    len - buf.len(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const REF_HEADER: Header = Header { version: (1, 2) };
    const REF_HEADER_BYTES: &[u8; 8] = constcat::concat_bytes!(
        &MAGIC_BYTES,  // Magic bytes
        &[0b111_u8],   // Length Pvarint(3)
        &[1_u8, 2_u8], // Version (Major, Minor)
        &[0x00]        // Guard
    );

    #[test]
    fn decode_header() {
        let des = Deserialiser {};
        let (out, read) = des.deserialise_header(REF_HEADER_BYTES.as_slice()).unwrap();

        assert_eq!(REF_HEADER, out);
        assert_eq!(REF_HEADER_BYTES.len(), read);

        let ser = Serialiser {};
        let mut buf = [0; 8];
        let written = ser.serialise_header(&mut buf, &REF_HEADER).unwrap();

        assert_eq!(REF_HEADER_BYTES, &buf);
        assert_eq!(buf.len(), written);
    }
}
