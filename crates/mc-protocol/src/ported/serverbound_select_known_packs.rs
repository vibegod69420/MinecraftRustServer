use bytes::{Buf, BufMut};

/// Minimal packet codec error for self-contained configuration packet decoding.
#[derive(Debug, thiserror::Error)]
pub enum PacketCodecError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("varint too large")]
    VarIntTooLarge,
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("string too long: actual={actual}, max={max}")]
    StringTooLong { actual: i32, max: i32 },
    #[error("list too long: actual={actual}, max={max}")]
    ListTooLong { actual: i32, max: i32 },
}

fn write_varint(buf: &mut impl BufMut, value: i32) {
    let mut v = value as u32;
    loop {
        let mut b = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        buf.put_u8(b);
        if v == 0 {
            break;
        }
    }
}

fn read_varint(buf: &mut impl Buf) -> Result<i32, PacketCodecError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    for _ in 0..5 {
        if !buf.has_remaining() {
            return Err(PacketCodecError::UnexpectedEof);
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl(shift);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
    }
    Err(PacketCodecError::VarIntTooLarge)
}

fn write_utf(buf: &mut impl BufMut, s: &str, max_chars: i32) -> Result<(), PacketCodecError> {
    let char_len = s.chars().count() as i32;
    if char_len > max_chars {
        return Err(PacketCodecError::StringTooLong {
            actual: char_len,
            max: max_chars,
        });
    }
    let bytes = s.as_bytes();
    let max_bytes = max_chars.wrapping_mul(3);
    let byte_len = bytes.len() as i32;
    if byte_len > max_bytes {
        return Err(PacketCodecError::StringTooLong {
            actual: byte_len,
            max: max_bytes,
        });
    }
    write_varint(buf, byte_len);
    buf.put_slice(bytes);
    Ok(())
}

fn read_utf(buf: &mut impl Buf, max_chars: i32) -> Result<String, PacketCodecError> {
    let max_bytes = max_chars.wrapping_mul(3);
    let len = read_varint(buf)?;
    if len < 0 || len > max_bytes {
        return Err(PacketCodecError::StringTooLong {
            actual: len,
            max: max_chars,
        });
    }
    let len_usize = len as usize;
    if buf.remaining() < len_usize {
        return Err(PacketCodecError::UnexpectedEof);
    }
    let mut bytes = vec![0u8; len_usize];
    buf.copy_to_slice(&mut bytes);
    let s = String::from_utf8(bytes)?;
    let char_len = s.chars().count() as i32;
    if char_len > max_chars {
        return Err(PacketCodecError::StringTooLong {
            actual: char_len,
            max: max_chars,
        });
    }
    Ok(s)
}

/// Minimal self-contained representation of `net.minecraft.server.packs.repository.KnownPack`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl KnownPack {
    pub fn new(namespace: String, id: String, version: String) -> Self {
        Self {
            namespace,
            id,
            version,
        }
    }

    pub fn stream_encode(&self, buf: &mut impl BufMut) -> Result<(), PacketCodecError> {
        write_utf(buf, &self.namespace, 32767)?;
        write_utf(buf, &self.id, 32767)?;
        write_utf(buf, &self.version, 32767)?;
        Ok(())
    }

    pub fn stream_decode(buf: &mut impl Buf) -> Result<Self, PacketCodecError> {
        let namespace = read_utf(buf, 32767)?;
        let id = read_utf(buf, 32767)?;
        let version = read_utf(buf, 32767)?;
        Ok(Self {
            namespace,
            id,
            version,
        })
    }
}

fn write_known_pack_list(
    buf: &mut impl BufMut,
    list: &[KnownPack],
    max_len: i32,
) -> Result<(), PacketCodecError> {
    let len = list.len() as i32;
    if len > max_len {
        return Err(PacketCodecError::ListTooLong {
            actual: len,
            max: max_len,
        });
    }
    write_varint(buf, len);
    for pack in list {
        pack.stream_encode(buf)?;
    }
    Ok(())
}

fn read_known_pack_list(
    buf: &mut impl Buf,
    max_len: i32,
) -> Result<Vec<KnownPack>, PacketCodecError> {
    let len = read_varint(buf)?;
    if len < 0 || len > max_len {
        return Err(PacketCodecError::ListTooLong {
            actual: len,
            max: max_len,
        });
    }
    let mut out = Vec::with_capacity(len as usize);
    for _ in 0..len {
        out.push(KnownPack::stream_decode(buf)?);
    }
    Ok(out)
}

pub const MAX_KNOWN_PACKS: i32 = 64;

/// Minimal self-contained packet type marker for configuration packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationPacketType {
    ServerboundSelectKnownPacks,
}

/// Minimal self-contained listener for
/// `net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener`.
pub trait ServerConfigurationPacketListener {
    fn handle_select_known_packs(&mut self, packet: &ServerboundSelectKnownPacks);
}

/// Port of `net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSelectKnownPacks {
    pub known_packs: Vec<KnownPack>,
}

impl ServerboundSelectKnownPacks {
    // JAVA: net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks#ServerboundSelectKnownPacks
    pub fn new(known_packs: Vec<KnownPack>) -> Self {
        Self { known_packs }
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks#knownPacks
    pub fn known_packs(&self) -> &[KnownPack] {
        &self.known_packs
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks#type
    pub fn type_(&self) -> ConfigurationPacketType {
        ConfigurationPacketType::ServerboundSelectKnownPacks
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks#handle
    pub fn handle(&self, listener: &mut impl ServerConfigurationPacketListener) {
        listener.handle_select_known_packs(self);
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks#STREAM_CODEC
    pub fn stream_encode(&self, buf: &mut impl BufMut) -> Result<(), PacketCodecError> {
        write_known_pack_list(buf, &self.known_packs, MAX_KNOWN_PACKS)
    }

    // JAVA: net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks#STREAM_CODEC
    pub fn stream_decode(buf: &mut impl Buf) -> Result<Self, PacketCodecError> {
        let known_packs = read_known_pack_list(buf, MAX_KNOWN_PACKS)?;
        Ok(Self { known_packs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_empty_packet() {
        let packet = ServerboundSelectKnownPacks::new(Vec::new());
        let mut buf = BytesMut::new();
        packet.stream_encode(&mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = ServerboundSelectKnownPacks::stream_decode(&mut reader).unwrap();
        assert_eq!(decoded, packet);
        assert!(decoded.known_packs().is_empty());
    }

    #[test]
    fn roundtrip_handle_and_type() {
        let packet = ServerboundSelectKnownPacks::new(vec![
            KnownPack::new("minecraft".to_string(), "core".to_string(), "1.21.11".to_string()),
            KnownPack::new("custom".to_string(), "pack".to_string(), "1.0.0".to_string()),
        ]);

        let mut buf = BytesMut::new();
        packet.stream_encode(&mut buf).unwrap();
        let mut reader = buf.freeze();
        let decoded = ServerboundSelectKnownPacks::stream_decode(&mut reader).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(
            packet.type_(),
            ConfigurationPacketType::ServerboundSelectKnownPacks
        );

        struct TestListener {
            called: bool,
            count: usize,
        }
        impl ServerConfigurationPacketListener for TestListener {
            fn handle_select_known_packs(&mut self, packet: &ServerboundSelectKnownPacks) {
                self.called = true;
                self.count = packet.known_packs().len();
            }
        }

        let mut listener = TestListener {
            called: false,
            count: 0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.count, 2);
    }

    #[test]
    fn rejects_oversized_list() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, MAX_KNOWN_PACKS.wrapping_add(1));
        let mut reader = buf.freeze();
        let err = ServerboundSelectKnownPacks::stream_decode(&mut reader).unwrap_err();
        assert!(matches!(
            err,
            PacketCodecError::ListTooLong {
                actual: 65,
                max: 64
            }
        ));
    }

    #[test]
    fn rejects_negative_list_length() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -1);
        let mut reader = buf.freeze();
        let err = ServerboundSelectKnownPacks::stream_decode(&mut reader).unwrap_err();
        assert!(matches!(
            err,
            PacketCodecError::ListTooLong {
                actual: -1,
                max: 64
            }
        ));
    }

    #[test]
    fn varint_roundtrip_overflow_negative_and_float_bits() {
        let neg_zero_bits = f32::to_bits(-0.0_f32) as i32;
        let nan_bits = f32::to_bits(f32::from_bits(0x7FC0_0001u32)) as i32;
        let values = [
            0i32,
            -1i32,
            i32::MIN,
            i32::MAX,
            1i32.wrapping_sub(2), // -1 via wrapping semantics
            neg_zero_bits,
            nan_bits,
        ];

        for &value in &values {
            let mut buf = BytesMut::new();
            write_varint(&mut buf, value);
            let mut reader = buf.freeze();
            let decoded = read_varint(&mut reader).unwrap();
            assert_eq!(decoded, value, "varint roundtrip failed for {value}");
        }

        // Float bit patterns must be preserved exactly when reinterpreted.
        assert_eq!(f32::to_bits(f32::from_bits(neg_zero_bits as u32)), f32::to_bits(-0.0_f32));
        assert_eq!(
            f32::to_bits(f32::from_bits(nan_bits as u32)),
            f32::to_bits(f32::from_bits(0x7FC0_0001u32))
        );
    }
}
