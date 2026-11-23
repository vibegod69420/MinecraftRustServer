use std::io::{self, Cursor, Read};

const PNG_HEADER: i64 = -8_552_249_625_308_161_526_i64;
const IHDR_TYPE: i32 = 1_229_472_850_i32;
const IHDR_SIZE: i32 = 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PngInfo {
    pub width: i32,
    pub height: i32,
}

impl PngInfo {
    // JAVA: net.minecraft.util.PngInfo#fromStream
    pub fn from_stream<R: Read>(reader: &mut R) -> io::Result<Self> {
        let png_header = read_i64_be(reader)?;
        if png_header != PNG_HEADER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad PNG Signature: 0x{:016X}", png_header as u64),
            ));
        }

        let ihdr_size = read_i32_be(reader)?;
        if ihdr_size != IHDR_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad length for IHDR chunk: {ihdr_size}"),
            ));
        }

        let ihdr_type = read_i32_be(reader)?;
        if ihdr_type != IHDR_TYPE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad type for IHDR chunk: 0x{:08X}", ihdr_type as u32),
            ));
        }

        let width = read_i32_be(reader)?;
        let height = read_i32_be(reader)?;
        Ok(Self { width, height })
    }

    // JAVA: net.minecraft.util.PngInfo#fromBytes
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        Self::from_stream(&mut cursor)
    }

    // JAVA: net.minecraft.util.PngInfo#validateHeader
    pub fn validate_header(buf: &[u8]) -> io::Result<()> {
        if buf.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "PNG header missing",
            ));
        }

        let sig = i64::from_be_bytes(buf[0..8].try_into().unwrap());
        if sig != PNG_HEADER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Bad PNG Signature",
            ));
        }

        let ihdr_len = i32::from_be_bytes(buf[8..12].try_into().unwrap());
        if ihdr_len != IHDR_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Bad length for IHDR chunk!",
            ));
        }

        let ihdr_type = i32::from_be_bytes(buf[12..16].try_into().unwrap());
        if ihdr_type != IHDR_TYPE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Bad type for IHDR chunk!",
            ));
        }

        Ok(())
    }
}

fn read_i64_be<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

fn read_i32_be<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_png_header(width: i32, height: i32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&PNG_HEADER.to_be_bytes());
        buf.extend_from_slice(&IHDR_SIZE.to_be_bytes());
        buf.extend_from_slice(&IHDR_TYPE.to_be_bytes());
        buf.extend_from_slice(&width.to_be_bytes());
        buf.extend_from_slice(&height.to_be_bytes());
        buf
    }

    #[test]
    fn parses_valid_header() {
        let bytes = make_png_header(16, 32);
        let info = PngInfo::from_bytes(&bytes).unwrap();
        assert_eq!(info, PngInfo { width: 16, height: 32 });
    }

    #[test]
    fn bad_signature_errors() {
        let mut bytes = make_png_header(1, 1);
        bytes[0] ^= 0xFF;
        let err = PngInfo::from_bytes(&bytes).unwrap_err();
        assert!(err.to_string().contains("Bad PNG Signature"));
    }

    #[test]
    fn bad_ihdr_length_and_type() {
        let mut bad_len = make_png_header(1, 1);
        bad_len[8..12].copy_from_slice(&12i32.to_be_bytes());
        let err = PngInfo::from_bytes(&bad_len).unwrap_err();
        assert!(err.to_string().contains("Bad length for IHDR chunk"));

        let mut bad_type = make_png_header(1, 1);
        bad_type[12..16].copy_from_slice(&0i32.to_be_bytes());
        let err = PngInfo::from_bytes(&bad_type).unwrap_err();
        assert!(err.to_string().contains("Bad type for IHDR chunk"));
    }

    #[test]
    fn validate_header_checks_and_negative_dims() {
        let bytes = make_png_header(-1, -2);
        PngInfo::validate_header(&bytes).unwrap();
        let info = PngInfo::from_bytes(&bytes).unwrap();
        assert_eq!(info.width, -1);
        assert_eq!(info.height, -2);

        let err = PngInfo::validate_header(&[0u8; 8]).unwrap_err();
        assert_eq!(err.to_string(), "PNG header missing");
    }

    #[test]
    fn validate_header_bad_fields() {
        let mut bad_sig = make_png_header(1, 1);
        bad_sig[0] = 0;
        assert_eq!(PngInfo::validate_header(&bad_sig).unwrap_err().to_string(), "Bad PNG Signature");

        let mut bad_len = make_png_header(1, 1);
        bad_len[8..12].copy_from_slice(&0i32.to_be_bytes());
        assert_eq!(PngInfo::validate_header(&bad_len).unwrap_err().to_string(), "Bad length for IHDR chunk!");

        let mut bad_type = make_png_header(1, 1);
        bad_type[12..16].copy_from_slice(&1i32.to_be_bytes());
        assert_eq!(PngInfo::validate_header(&bad_type).unwrap_err().to_string(), "Bad type for IHDR chunk!");
    }
}
