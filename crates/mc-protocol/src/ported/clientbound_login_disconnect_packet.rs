use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

impl PacketType {
    pub const CLIENTBOUND_LOGIN_DISCONNECT: Self = Self {
        name: "clientbound_login_disconnect",
    };
}

pub trait ClientLoginPacketListener {
    fn handle_disconnect(&mut self, packet: &ClientboundLoginDisconnectPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            pos: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;
        loop {
            let current_byte = self.read_byte()? as i32;
            value |= (current_byte & 127) << ((position.wrapping_mul(7)) & 31);
            position = position.wrapping_add(1);
            if position > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (current_byte & 128) != 128 {
                break;
            }
        }
        Ok(value)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & -128i32) == 0 {
                self.write_byte(value as u8);
                return self;
            }
            self.write_byte(((value & 127) | 128) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
    pub fn read_utf(&mut self, max_bytes: i32) -> io::Result<String> {
        let len = self.read_var_int()?;
        if len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The received encoded string buffer length is less than zero! Weird string!",
            ));
        }
        let len = len as usize;
        let max_bytes = max_bytes.max(0) as usize;
        if len > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The received encoded string buffer length is longer than maximum allowed",
            ));
        }
        if self.remaining() < len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes in buffer",
            ));
        }
        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
    pub fn write_utf(&mut self, value: &str, max_bytes: i32) -> io::Result<&mut Self> {
        let bytes = value.as_bytes();
        let max_bytes = max_bytes.max(0) as usize;
        if bytes.len() > max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too big",
            ));
        }
        self.write_var_int(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
        Ok(self)
    }
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

/// Port of the login disconnect packet. The `reason` is stored as the JSON
/// component text carried on the wire by the lenient JSON string codec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginDisconnectPacket {
    reason: String,
}

impl ClientboundLoginDisconnectPacket {
    const MAX_REASON_BYTES: i32 = 262_144;

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket#<init>
    pub fn new(reason: String) -> Self {
        Self { reason }
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let reason = buf.read_utf(Self::MAX_REASON_BYTES)?;
        Ok(Self { reason })
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_utf(&self.reason, Self::MAX_REASON_BYTES)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket#reason
    pub fn reason(&self) -> &str {
        &self.reason
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::CLIENTBOUND_LOGIN_DISCONNECT
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientLoginPacketListener) {
        listener.handle_disconnect(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_reason_json() {
        let reason = r#"{"text":"Disconnected"}"#.to_string();
        let packet = ClientboundLoginDisconnectPacket::new(reason.clone());
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_bytes());
        let decoded = ClientboundLoginDisconnectPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.reason(), reason);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn empty_and_unicode_reason_roundtrip() {
        for reason in ["", r#"{"text":"Привет 🌍"}"#, r#"{"translate":"multiplayer.disconnect.kicked"}"#] {
            let packet = ClientboundLoginDisconnectPacket::new(reason.to_string());
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_bytes());
            let decoded = ClientboundLoginDisconnectPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.reason(), reason);
        }
    }

    #[test]
    fn oversized_reason_rejected_on_write() {
        let big = "a".repeat(262_145);
        let packet = ClientboundLoginDisconnectPacket::new(big);
        let mut buf = FriendlyByteBuf::new();
        let err = packet.write(&mut buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn oversized_reason_rejected_on_read() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(262_145);
        buf.data.extend(std::iter::repeat_n(b'a', 262_145));
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_bytes());
        let err = ClientboundLoginDisconnectPacket::read(&mut read_buf).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct TestListener {
            seen: Option<String>,
        }
        impl ClientLoginPacketListener for TestListener {
            fn handle_disconnect(&mut self, packet: &ClientboundLoginDisconnectPacket) {
                self.seen = Some(packet.reason().to_string());
            }
        }

        let packet = ClientboundLoginDisconnectPacket::new("bye".to_string());
        let mut listener = TestListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen.as_deref(), Some("bye"));
        assert_eq!(
            packet.packet_type(),
            PacketType::CLIENTBOUND_LOGIN_DISCONNECT
        );
    }
}
