use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

impl PacketType {
    pub const CLIENTBOUND_LOGIN_COMPRESSION: Self = Self {
        name: "clientbound_login_compression",
    };
}

pub trait ClientLoginPacketListener {
    fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket);
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

    pub fn bytes(&self) -> &[u8] {
        &self.data
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
}

impl Default for FriendlyByteBuf {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginCompressionPacket {
    compression_threshold: i32,
}

impl ClientboundLoginCompressionPacket {
    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket#<init>
    pub fn new(compression_threshold: i32) -> Self {
        Self {
            compression_threshold,
        }
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let compression_threshold = buf.read_var_int()?;
        Ok(Self {
            compression_threshold,
        })
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.compression_threshold);
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::CLIENTBOUND_LOGIN_COMPRESSION
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientLoginPacketListener) {
        listener.handle_compression(self);
    }

    // JAVA: net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket#getCompressionThreshold
    pub fn get_compression_threshold(&self) -> i32 {
        self.compression_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_compression_threshold() {
        let packet = ClientboundLoginCompressionPacket::new(256);
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_bytes());
        let decoded = ClientboundLoginCompressionPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_compression_threshold(), 256);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn negative_and_extreme_threshold_roundtrip() {
        for value in [i32::MIN, -1, -2, 0, 1, i32::MAX] {
            let packet = ClientboundLoginCompressionPacket::new(value);
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_bytes());
            let decoded = ClientboundLoginCompressionPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_compression_threshold(), value);
        }
    }

    #[test]
    fn varint_negative_one_encoding() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        assert_eq!(buf.bytes(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_bytes());
        assert_eq!(read_buf.read_var_int().unwrap(), -1);
    }

    #[test]
    fn varint_too_big_errors() {
        let mut buf = FriendlyByteBuf::from_bytes(vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x00]);
        let err = buf.read_var_int().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct TestListener {
            seen: Option<i32>,
        }
        impl ClientLoginPacketListener for TestListener {
            fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket) {
                self.seen = Some(packet.get_compression_threshold());
            }
        }

        let packet = ClientboundLoginCompressionPacket::new(512);
        let mut listener = TestListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some(512));
        assert_eq!(
            packet.packet_type(),
            PacketType::CLIENTBOUND_LOGIN_COMPRESSION
        );
    }
}
