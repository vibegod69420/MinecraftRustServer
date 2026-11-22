use bytes::{BufMut, BytesMut};
use std::io;

/// Minimal self-contained UUID representation matching Java `java.util.UUID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct McUuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

impl McUuid {
    pub const fn new(most_significant_bits: i64, least_significant_bits: i64) -> Self {
        Self {
            most_significant_bits,
            least_significant_bits,
        }
    }
}

/// Minimal self-contained `FriendlyByteBuf` for login packet encoding/decoding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: BytesMut,
    read_pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: BytesMut::from(bytes),
            read_pos: 0,
        }
    }

    pub fn into_bytes(self) -> BytesMut {
        self.data
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF while reading byte",
            ));
        }
        let b = self.data[self.read_pos];
        self.read_pos = self.read_pos.wrapping_add(1);
        Ok(b)
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        if self.remaining() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF while reading i64",
            ));
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos.wrapping_add(8)]);
        self.read_pos = self.read_pos.wrapping_add(8);
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte()? as i32;
            result |= (b & 0x7F).wrapping_shl(shift & 31);
            if (b & 0x80) == 0 {
                return Ok(result);
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            let mut b = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                b |= 0x80;
            }
            self.data.put_u8(b);
            if val == 0 {
                break;
            }
        }
    }

    pub fn read_utf(&mut self, max_length: i32) -> io::Result<String> {
        let max_bytes = (max_length as u32).wrapping_mul(3) as usize;
        let byte_len_i32 = self.read_var_int()?;
        if byte_len_i32 < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The received encoded string buffer length is less than zero",
            ));
        }
        let byte_len = byte_len_i32 as usize;
        if byte_len > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "The received encoded string buffer length is longer than maximum allowed ({byte_len} > {max_bytes})"
                ),
            ));
        }
        if self.remaining() < byte_len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF while reading UTF string bytes",
            ));
        }
        let start = self.read_pos;
        self.read_pos = self.read_pos.wrapping_add(byte_len);
        let s = String::from_utf8(self.data[start..self.read_pos].to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let char_count = s.encode_utf16().count() as i32;
        if char_count > max_length {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "The received string length is longer than maximum allowed ({char_count} > {max_length})"
                ),
            ));
        }
        Ok(s)
    }

    pub fn write_utf(&mut self, value: &str, max_length: i32) {
        let char_count = value.encode_utf16().count() as i32;
        if char_count > max_length {
            panic!(
                "String too big (was {char_count} characters, max {max_length})"
            );
        }
        let utf8_bytes = value.as_bytes();
        let max_bytes = (max_length as u32).wrapping_mul(3) as usize;
        if utf8_bytes.len() > max_bytes {
            panic!(
                "String too big (was {} bytes encoded, max {max_bytes})",
                utf8_bytes.len()
            );
        }
        self.write_var_int(utf8_bytes.len() as i32);
        self.data.put_slice(utf8_bytes);
    }

    pub fn read_uuid(&mut self) -> io::Result<McUuid> {
        let most_significant_bits = self.read_i64()?;
        let least_significant_bits = self.read_i64()?;
        Ok(McUuid::new(most_significant_bits, least_significant_bits))
    }

    pub fn write_uuid(&mut self, value: McUuid) {
        self.data.put_i64(value.most_significant_bits);
        self.data.put_i64(value.least_significant_bits);
    }
}

/// Minimal packet type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

/// Minimal login packet type constants.
pub struct LoginPacketTypes;

impl LoginPacketTypes {
    pub const SERVERBOUND_HELLO: PacketType = PacketType {
        id: "minecraft:login/serverbound_hello",
    };
}

/// Minimal listener trait for serverbound login hello handling.
pub trait ServerLoginPacketListener {
    fn handle_hello(&mut self, packet: &ServerboundHelloPacket);
}

/// Minimal stream codec representation.
pub struct StreamCodec<T> {
    pub encode: fn(&T, &mut FriendlyByteBuf),
    pub decode: fn(&mut FriendlyByteBuf) -> io::Result<T>,
}

/// Port of `net.minecraft.network.protocol.login.ServerboundHelloPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundHelloPacket {
    pub name: String,
    pub profile_id: McUuid,
}

pub static STREAM_CODEC: StreamCodec<ServerboundHelloPacket> = StreamCodec {
    encode: |packet, buf| packet.write(buf),
    decode: ServerboundHelloPacket::read,
};

impl ServerboundHelloPacket {
    // JAVA: net.minecraft.network.protocol.login.ServerboundHelloPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let name = buf.read_utf(16)?;
        let profile_id = buf.read_uuid()?;
        Ok(Self { name, profile_id })
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundHelloPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.name, 16);
        buf.write_uuid(self.profile_id);
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundHelloPacket#type
    pub fn get_type(&self) -> PacketType {
        LoginPacketTypes::SERVERBOUND_HELLO
    }

    // JAVA: net.minecraft.network.protocol.login.ServerboundHelloPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerLoginPacketListener) {
        listener.handle_hello(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_basic_packet() {
        let packet = ServerboundHelloPacket {
            name: "Player".to_string(),
            profile_id: McUuid::new(0x0123_4567_89ab_cdef_i64, -0x0123_4567_89ab_cdf0_i64),
        };

        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);

        let mut read_buf = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = ServerboundHelloPacket::read(&mut read_buf).expect("decode should succeed");

        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn roundtrip_edge_case_names_and_empty() {
        let empty = ServerboundHelloPacket {
            name: String::new(),
            profile_id: McUuid::new(0, 0),
        };
        let max_name = ServerboundHelloPacket {
            name: "A".repeat(16),
            profile_id: McUuid::new(i64::MAX, i64::MIN),
        };

        for packet in [empty, max_name] {
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_bytes(buf.bytes());
            let decoded =
                ServerboundHelloPacket::read(&mut read_buf).expect("decode should succeed");
            assert_eq!(decoded, packet);
        }
    }

    #[test]
    fn negative_inputs_and_wrapping_integer_semantics() {
        let packet = ServerboundHelloPacket {
            name: "Neg".to_string(),
            profile_id: McUuid::new(-1, i64::MIN),
        };

        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = ServerboundHelloPacket::read(&mut read_buf).expect("decode should succeed");

        assert_eq!(decoded.profile_id.most_significant_bits, -1);
        assert_eq!(decoded.profile_id.least_significant_bits, i64::MIN);

        let wrapped_add = decoded.profile_id.most_significant_bits.wrapping_add(1);
        let wrapped_sub = decoded.profile_id.least_significant_bits.wrapping_sub(1);
        let wrapped_mul = (decoded.profile_id.most_significant_bits as i32).wrapping_mul(3);
        assert_eq!(wrapped_add, 0);
        assert_eq!(wrapped_sub, i64::MAX);
        assert_eq!(wrapped_mul, -3);
    }

    #[test]
    fn float_bit_patterns_preserved_in_uuid_fields() {
        let most_bits = f64::NAN.to_bits();
        let least_bits = f64::NEG_INFINITY.to_bits();
        let packet = ServerboundHelloPacket {
            name: "Bits".to_string(),
            profile_id: McUuid::new(most_bits as i64, least_bits as i64),
        };

        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = ServerboundHelloPacket::read(&mut read_buf).expect("decode should succeed");

        assert_eq!(
            decoded.profile_id.most_significant_bits as u64,
            most_bits
        );
        assert_eq!(
            decoded.profile_id.least_significant_bits as u64,
            least_bits
        );
        assert!(f64::from_bits(decoded.profile_id.most_significant_bits as u64).is_nan());
        assert_eq!(
            f64::from_bits(decoded.profile_id.least_significant_bits as u64),
            f64::NEG_INFINITY
        );
    }

    #[test]
    fn handle_dispatches_and_type_matches() {
        struct TestListener {
            called: bool,
            seen_name: String,
            seen_profile_id: McUuid,
        }

        impl ServerLoginPacketListener for TestListener {
            fn handle_hello(&mut self, packet: &ServerboundHelloPacket) {
                self.called = true;
                self.seen_name = packet.name.clone();
                self.seen_profile_id = packet.profile_id;
            }
        }

        let packet = ServerboundHelloPacket {
            name: "Steve".to_string(),
            profile_id: McUuid::new(1, 2),
        };

        assert_eq!(packet.get_type(), LoginPacketTypes::SERVERBOUND_HELLO);

        let mut listener = TestListener {
            called: false,
            seen_name: String::new(),
            seen_profile_id: McUuid::new(0, 0),
        };
        packet.handle(&mut listener);

        assert!(listener.called);
        assert_eq!(listener.seen_name, "Steve");
        assert_eq!(listener.seen_profile_id, McUuid::new(1, 2));
    }

    #[test]
    fn stream_codec_roundtrip() {
        let packet = ServerboundHelloPacket {
            name: "Codec".to_string(),
            profile_id: McUuid::new(i64::MIN, -42),
        };

        let mut buf = FriendlyByteBuf::new();
        (STREAM_CODEC.encode)(&packet, &mut buf);

        let mut read_buf = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = (STREAM_CODEC.decode)(&mut read_buf).expect("decode should succeed");
        assert_eq!(decoded, packet);
    }
}
