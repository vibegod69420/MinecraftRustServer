use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const LOGIN_PACKET_TYPES_CLIENTBOUND_HELLO: PacketType =
    PacketType("minecraft:login/clientbound_hello");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey {
    pub encoded: Vec<u8>,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error("{0}")]
pub struct CryptException(pub String);

pub struct Crypt;

impl Crypt {
    /// Minimal stub: real Minecraft uses X.509 SubjectPublicKeyInfo parsing.
    pub fn byte_to_public_key(bytes: &[u8]) -> Result<PublicKey, CryptException> {
        if bytes.is_empty() {
            return Err(CryptException("empty key".to_string()));
        }
        Ok(PublicKey {
            encoded: bytes.to_vec(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PacketDecodeError {
    #[error("unexpected end of buffer")]
    Eof,
    #[error("varint too big")]
    VarIntTooBig,
    #[error("string too long")]
    StringTooLong,
    #[error("byte array too long")]
    ByteArrayTooLong,
    #[error("invalid utf8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_idx: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_idx: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_idx: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_idx)
    }

    fn read_byte(&mut self) -> Result<u8, PacketDecodeError> {
        if self.read_idx >= self.data.len() {
            return Err(PacketDecodeError::Eof);
        }
        let b = self.data[self.read_idx];
        self.read_idx = self.read_idx.wrapping_add(1);
        Ok(b)
    }

    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    pub fn read_varint(&mut self) -> Result<i32, PacketDecodeError> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte()?;
            result |= ((b & 0x7F) as i32).wrapping_shl(shift & 31);
            if (b & 0x80) == 0 {
                return Ok(result);
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return Err(PacketDecodeError::VarIntTooBig);
            }
        }
    }

    pub fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut b = (value as u32 & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                b |= 0x80;
            }
            self.write_byte(b);
            if value == 0 {
                return;
            }
        }
    }

    pub fn read_utf(&mut self, max_length: usize) -> Result<String, PacketDecodeError> {
        let max_bytes = max_length.wrapping_mul(3);
        let byte_len_i32 = self.read_varint()?;
        if byte_len_i32 < 0 {
            return Err(PacketDecodeError::StringTooLong);
        }
        let byte_len = byte_len_i32 as usize;
        if byte_len > max_bytes {
            return Err(PacketDecodeError::StringTooLong);
        }
        if byte_len > self.remaining() {
            return Err(PacketDecodeError::Eof);
        }
        let start = self.read_idx;
        let end = start.wrapping_add(byte_len);
        let bytes = self.data[start..end].to_vec();
        self.read_idx = end;
        let s = String::from_utf8(bytes)?;
        if s.encode_utf16().count() > max_length {
            return Err(PacketDecodeError::StringTooLong);
        }
        Ok(s)
    }

    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    pub fn read_byte_array(&mut self) -> Result<Vec<u8>, PacketDecodeError> {
        let len_i32 = self.read_varint()?;
        if len_i32 < 0 {
            return Err(PacketDecodeError::ByteArrayTooLong);
        }
        let len = len_i32 as usize;
        if len > self.remaining() {
            return Err(PacketDecodeError::Eof);
        }
        let start = self.read_idx;
        let end = start.wrapping_add(len);
        let bytes = self.data[start..end].to_vec();
        self.read_idx = end;
        Ok(bytes)
    }

    pub fn write_byte_array(&mut self, value: &[u8]) {
        self.write_varint(value.len() as i32);
        self.data.extend_from_slice(value);
    }

    pub fn read_boolean(&mut self) -> Result<bool, PacketDecodeError> {
        Ok(self.read_byte()? != 0)
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.write_byte(if value { 1 } else { 0 });
    }
}

pub trait ClientLoginPacketListener {
    fn handle_hello(&mut self, packet: &ClientboundHelloPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundHelloPacket {
    server_id: String,
    public_key: Vec<u8>,
    challenge: Vec<u8>,
    should_authenticate: bool,
}

impl ClientboundHelloPacket {
    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#<init>
    pub fn new(
        server_id: String,
        public_key: Vec<u8>,
        challenge: Vec<u8>,
        should_authenticate: bool,
    ) -> Self {
        Self {
            server_id,
            public_key,
            challenge,
            should_authenticate,
        }
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Result<Self, PacketDecodeError> {
        let server_id = buf.read_utf(20)?;
        let public_key = buf.read_byte_array()?;
        let challenge = buf.read_byte_array()?;
        let should_authenticate = buf.read_boolean()?;
        Ok(Self {
            server_id,
            public_key,
            challenge,
            should_authenticate,
        })
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.server_id);
        buf.write_byte_array(&self.public_key);
        buf.write_byte_array(&self.challenge);
        buf.write_boolean(self.should_authenticate);
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#type
    pub fn packet_type(&self) -> PacketType {
        LOGIN_PACKET_TYPES_CLIENTBOUND_HELLO
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#handle
    pub fn handle<L: ClientLoginPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_hello(self);
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#getServerId
    pub fn get_server_id(&self) -> &str {
        &self.server_id
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#getPublicKey
    pub fn get_public_key(&self) -> Result<PublicKey, CryptException> {
        Crypt::byte_to_public_key(&self.public_key)
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#getChallenge
    pub fn get_challenge(&self) -> &[u8] {
        &self.challenge
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundHelloPacket#shouldAuthenticate
    pub fn should_authenticate(&self) -> bool {
        self.should_authenticate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let packet = ClientboundHelloPacket::new(
            "server".to_string(),
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            true,
        );
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundHelloPacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
        assert_eq!(decoded.get_server_id(), "server");
        assert_eq!(decoded.get_challenge(), &[5, 6, 7, 8]);
        assert!(decoded.should_authenticate());
        assert_eq!(
            decoded.packet_type(),
            LOGIN_PACKET_TYPES_CLIENTBOUND_HELLO
        );
    }

    #[test]
    fn test_empty_and_edge_values() {
        let packet = ClientboundHelloPacket::new(String::new(), vec![], vec![0xFF, 0x00], false);
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundHelloPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_server_id(), "");
        assert_eq!(decoded.get_public_key().unwrap_err().0, "empty key");
        assert_eq!(decoded.get_challenge(), &[0xFF, 0x00]);
        assert!(!decoded.should_authenticate());
    }

    #[test]
    fn test_varint_negative_and_extreme_values() {
        let mut buf = FriendlyByteBuf::new();
        let values = [0i32, -1, i32::MIN, i32::MAX, -128, 128, -2_147_483_648];
        for &v in &values {
            buf.write_varint(v);
        }
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        for &v in &values {
            let got = read_buf.read_varint().unwrap();
            assert_eq!(got, v, "varint round-trip failed for {v}");
        }
    }

    #[test]
    fn test_float_bits_preserved_in_byte_array() {
        let f32_bits = f32::NEG_INFINITY.to_bits();
        let f64_bits = f64::NAN.to_bits();
        let mut payload = Vec::new();
        payload.extend_from_slice(&f32_bits.to_le_bytes());
        payload.extend_from_slice(&f64_bits.to_le_bytes());

        let packet = ClientboundHelloPacket::new("f".to_string(), payload, vec![], true);
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let decoded =
            ClientboundHelloPacket::read(&mut FriendlyByteBuf::from_bytes(buf.into_inner()))
                .unwrap();

        let pk = decoded.get_public_key().unwrap();
        let got_f32 = f32::from_bits(u32::from_le_bytes(pk.encoded[0..4].try_into().unwrap()));
        let got_f64 = f64::from_bits(u64::from_le_bytes(pk.encoded[4..12].try_into().unwrap()));
        assert_eq!(got_f32.to_bits(), f32_bits);
        assert_eq!(got_f64.to_bits(), f64_bits);
    }

    #[test]
    fn test_handle_and_listener_dispatch() {
        struct MockListener {
            seen: Option<ClientboundHelloPacket>,
        }
        impl ClientLoginPacketListener for MockListener {
            fn handle_hello(&mut self, packet: &ClientboundHelloPacket) {
                self.seen = Some(packet.clone());
            }
        }

        let packet = ClientboundHelloPacket::new("id".to_string(), vec![10, 20], vec![30], true);
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        let seen = listener.seen.unwrap();
        assert_eq!(seen.get_server_id(), "id");
        let pk = seen.get_public_key().unwrap();
        assert_eq!(pk.encoded, vec![10, 20]);
        assert_eq!(seen.get_challenge(), &[30]);
        assert!(seen.should_authenticate());
    }
}
