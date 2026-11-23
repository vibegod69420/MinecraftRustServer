use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub const CLIENTBOUND_ANIMATE: PacketType = PacketType("minecraft:clientbound_animate");

pub const SWING_MAIN_HAND: i32 = 0;
pub const WAKE_UP: i32 = 2;
pub const SWING_OFF_HAND: i32 = 3;
pub const CRITICAL_HIT: i32 = 4;
pub const MAGIC_CRITICAL_HIT: i32 = 5;

pub trait ClientGamePacketListener {
    fn handle_animate(&mut self, packet: &ClientboundAnimatePacket);
}

pub trait EntityLike {
    fn get_id(&self) -> i32;
}

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readableBytes
    pub fn readable_bytes(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writerIndex/getBytes
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    pub fn read_byte(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_byte"));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    pub fn write_byte(&mut self, value: u8) {
        self.data.push(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUnsignedByte
    pub fn read_unsigned_byte(&mut self) -> io::Result<i32> {
        Ok(self.read_byte()? as i32)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let byte = self.read_byte()? as i32;
            value |= (byte & 0x7F) << (position & 31);
            if (byte & 0x80) == 0 {
                return Ok(value);
            }
            position = position.wrapping_add(7);
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7Fu32) == 0 {
                self.write_byte(val as u8);
                return;
            }
            self.write_byte(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAnimatePacket {
    id: i32,
    action: i32,
}

impl ClientboundAnimatePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#<init>
    pub fn from_entity<E: EntityLike>(entity: &E, action: i32) -> Self {
        Self {
            id: entity.get_id(),
            action,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#<init>
    pub fn new(id: i32, action: i32) -> Self {
        Self { id, action }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_var_int()?;
        let action = buf.read_unsigned_byte()?;
        Ok(Self { id, action })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_byte((self.action & 0xFF) as u8);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_ANIMATE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_animate(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAnimatePacket#getAction
    pub fn get_action(&self) -> i32 {
        self.action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEntity {
        id: i32,
    }

    impl EntityLike for MockEntity {
        fn get_id(&self) -> i32 {
            self.id
        }
    }

    struct MockListener {
        called: bool,
        id: i32,
        action: i32,
    }

    impl MockListener {
        fn new() -> Self {
            Self {
                called: false,
                id: 0,
                action: -1,
            }
        }
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_animate(&mut self, packet: &ClientboundAnimatePacket) {
            self.called = true;
            self.id = packet.get_id();
            self.action = packet.get_action();
        }
    }

    #[test]
    fn roundtrip_basic() {
        let pkt = ClientboundAnimatePacket::new(42, SWING_MAIN_HAND);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundAnimatePacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn from_entity_and_negative_id() {
        let entity = MockEntity { id: -1 };
        let pkt = ClientboundAnimatePacket::from_entity(&entity, CRITICAL_HIT);
        assert_eq!(pkt.get_id(), -1);
        assert_eq!(pkt.get_action(), CRITICAL_HIT);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let decoded = ClientboundAnimatePacket::read(&mut FriendlyByteBuf::from_vec(buf.into_vec())).unwrap();
        assert_eq!(decoded, pkt);
    }

    #[test]
    fn action_is_unsigned_byte_on_wire() {
        let pkt = ClientboundAnimatePacket::new(1, 255);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_vec();
        assert_eq!(*bytes.last().unwrap(), 255u8);
        let decoded = ClientboundAnimatePacket::read(&mut FriendlyByteBuf::from_vec(bytes)).unwrap();
        assert_eq!(decoded.get_action(), 255);
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundAnimatePacket::new(10, MAGIC_CRITICAL_HIT);
        let mut listener = MockListener::new();
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.id, 10);
        assert_eq!(listener.action, MAGIC_CRITICAL_HIT);
    }

    #[test]
    fn packet_type_and_constants() {
        let pkt = ClientboundAnimatePacket::new(0, SWING_OFF_HAND);
        assert_eq!(pkt.packet_type(), CLIENTBOUND_ANIMATE);
        assert_eq!(WAKE_UP, 2);
        assert_eq!(SWING_OFF_HAND, 3);
    }
}
