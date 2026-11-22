use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn minecraft(path: impl Into<String>) -> Self {
        Self::new("minecraft", path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPacketPayloadType {
    pub id: Identifier,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_index: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            read_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn readable_bytes(&self) -> usize {
        self.data.len().saturating_sub(self.read_index)
    }

    pub fn skip_bytes(&mut self, count: usize) {
        let new_index = self.read_index.saturating_add(count);
        self.read_index = new_index.min(self.data.len());
    }

    pub fn write_utf(&mut self, value: &str) -> Result<(), String> {
        self.write_utf_with_max_len(value, 32767)
    }

    pub fn write_utf_with_max_len(&mut self, value: &str, max_len: usize) -> Result<(), String> {
        let bytes = value.as_bytes();
        let char_count = value.chars().count();
        if char_count > max_len {
            return Err(format!("String too long ({} > {})", char_count, max_len));
        }
        let max_bytes = max_len.saturating_mul(3);
        if bytes.len() > max_bytes {
            return Err(format!(
                "String too big (was {} bytes encoded, max {})",
                bytes.len(),
                max_bytes
            ));
        }
        write_var_int(self, bytes.len() as i32);
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    pub fn read_utf(&mut self) -> Result<String, String> {
        self.read_utf_with_max_len(32767)
    }

    pub fn read_utf_with_max_len(&mut self, max_len: usize) -> Result<String, String> {
        let max_bytes = max_len.saturating_mul(3);
        let len = read_var_int(self)?;
        if len > max_bytes as i32 {
            return Err(format!(
                "The received encoded string buffer length is longer than maximum allowed ({} > {})",
                len, max_bytes
            ));
        }
        if len < 0 {
            return Err(
                "The received encoded string buffer length is less than zero!".to_string(),
            );
        }
        let len = len as usize;
        if len > self.readable_bytes() {
            return Err(format!(
                "Not enough bytes in buffer, expected {}, got {}",
                len,
                self.readable_bytes()
            ));
        }
        let start = self.read_index;
        let end = start.saturating_add(len);
        let s = std::str::from_utf8(&self.data[start..end])
            .map_err(|e| format!("Invalid UTF-8: {e}"))?;
        let char_count = s.chars().count();
        if char_count > max_len {
            return Err(format!(
                "The received string length is longer than maximum allowed ({} > {})",
                char_count, max_len
            ));
        }
        self.read_index = end;
        Ok(s.to_string())
    }
}

fn write_var_int(buf: &mut FriendlyByteBuf, value: i32) {
    let mut v = value as u32;
    loop {
        if (v & 0xFFFF_FF80) == 0 {
            buf.data.push(v as u8);
            return;
        }
        buf.data.push(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
}

fn read_var_int(buf: &mut FriendlyByteBuf) -> Result<i32, String> {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    loop {
        if buf.read_index >= buf.data.len() {
            return Err("VarInt too big".to_string());
        }
        let b = buf.data[buf.read_index] as i8;
        buf.read_index = buf.read_index.saturating_add(1);
        i |= ((b & 127) as i32).wrapping_shl(((j.wrapping_mul(7)) & 31) as u32);
        j = j.wrapping_add(1);
        if j > 5 {
            return Err("VarInt too big".to_string());
        }
        if (b as u8 & 128) == 0 {
            break;
        }
    }
    Ok(i)
}

pub struct StreamCodec<T> {
    pub encode: fn(&T, &mut FriendlyByteBuf) -> Result<(), String>,
    pub decode: fn(&mut FriendlyByteBuf) -> Result<T, String>,
}

impl<T> StreamCodec<T> {
    pub fn of(
        encode: fn(&T, &mut FriendlyByteBuf) -> Result<(), String>,
        decode: fn(&mut FriendlyByteBuf) -> Result<T, String>,
    ) -> Self {
        Self { encode, decode }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandPayload {
    pub brand: String,
}

pub const BRAND_TYPE_PATH: &str = "brand";

pub fn brand_payload_type() -> CustomPacketPayloadType {
    CustomPacketPayloadType {
        id: Identifier::minecraft(BRAND_TYPE_PATH),
    }
}

pub fn brand_stream_codec() -> &'static StreamCodec<BrandPayload> {
    static CODEC: OnceLock<StreamCodec<BrandPayload>> = OnceLock::new();
    CODEC.get_or_init(|| StreamCodec::of(BrandPayload::write, BrandPayload::read))
}

impl BrandPayload {
    // JAVA: net.minecraft.network.protocol.common.custom.BrandPayload#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, String> {
        Ok(Self {
            brand: buf.read_utf()?,
        })
    }

    // JAVA: net.minecraft.network.protocol.common.custom.BrandPayload#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> Result<(), String> {
        buf.write_utf(&self.brand)
    }

    // JAVA: net.minecraft.network.protocol.common.custom.BrandPayload#type
    pub fn payload_type(&self) -> CustomPacketPayloadType {
        brand_payload_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_round_trip() {
        let payload = BrandPayload {
            brand: "vanilla".to_string(),
        };
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = BrandPayload::read(&mut read_buf).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn brand_empty_and_unicode() {
        for brand in ["", "héllo-世界", "fabric"] {
            let payload = BrandPayload {
                brand: brand.to_string(),
            };
            let mut buf = FriendlyByteBuf::new();
            payload.write(&mut buf).unwrap();
            let decoded = BrandPayload::read(&mut FriendlyByteBuf::from_bytes(buf.into_inner()))
                .unwrap();
            assert_eq!(decoded.brand, brand);
        }
    }

    #[test]
    fn brand_type_is_minecraft_brand() {
        let payload = BrandPayload {
            brand: "x".to_string(),
        };
        let ty = payload.payload_type();
        assert_eq!(ty, brand_payload_type());
        assert_eq!(ty.id.namespace, "minecraft");
        assert_eq!(ty.id.path, "brand");
    }

    #[test]
    fn stream_codec_round_trip() {
        let codec = brand_stream_codec();
        let payload = BrandPayload {
            brand: "server".to_string(),
        };
        let mut buf = FriendlyByteBuf::new();
        (codec.encode)(&payload, &mut buf).unwrap();
        let decoded = (codec.decode)(&mut FriendlyByteBuf::from_bytes(buf.into_inner())).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn varint_negative_and_extremes() {
        for v in [0, -1, i32::MIN, i32::MAX, -128, 128] {
            let mut buf = FriendlyByteBuf::new();
            write_var_int(&mut buf, v);
            let got = read_var_int(&mut FriendlyByteBuf::from_bytes(buf.into_inner())).unwrap();
            assert_eq!(got, v, "varint round-trip failed for {v}");
        }
    }

    #[test]
    fn read_utf_rejects_negative_length() {
        let mut buf = FriendlyByteBuf::new();
        write_var_int(&mut buf, -1);
        let err = FriendlyByteBuf::from_bytes(buf.into_inner())
            .read_utf()
            .unwrap_err();
        assert!(err.contains("less than zero"));
    }
}
