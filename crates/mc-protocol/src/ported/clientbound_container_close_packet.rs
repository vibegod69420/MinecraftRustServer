use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

pub const CLIENTBOUND_CONTAINER_CLOSE: PacketType = PacketType {
    name: "minecraft:container_close",
};

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    buf: BytesMut,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            buf: BytesMut::from(bytes),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readContainerId
    pub fn read_container_id(&mut self) -> i32 {
        self.buf.get_u8() as i32
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeContainerId
    pub fn write_container_id(&mut self, container_id: i32) {
        self.buf.put_u8((container_id & 0xFF) as u8);
    }
}

pub trait ClientGamePacketListener {
    fn handle_container_close(&mut self, packet: &ClientboundContainerClosePacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundContainerClosePacket {
    container_id: i32,
}

impl ClientboundContainerClosePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerClosePacket#<init>
    pub fn new(container_id: i32) -> Self {
        Self { container_id }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerClosePacket#<init>
    fn read(buf: &mut FriendlyByteBuf) -> Self {
        let container_id = buf.read_container_id();
        Self { container_id }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerClosePacket#write
    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerClosePacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_CONTAINER_CLOSE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerClosePacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_container_close(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerClosePacket#getContainerId
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        container_id: i32,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_container_close(&mut self, packet: &ClientboundContainerClosePacket) {
            self.called = true;
            self.container_id = packet.get_container_id();
        }
    }

    #[test]
    fn test_new_and_getter() {
        let pkt = ClientboundContainerClosePacket::new(42);
        assert_eq!(pkt.get_container_id(), 42);
    }

    #[test]
    fn test_write_read_roundtrip() {
        let pkt = ClientboundContainerClosePacket::new(123);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundContainerClosePacket::read(&mut read_buf);
        assert_eq!(decoded.get_container_id(), 123);
    }

    #[test]
    fn test_negative_container_id_masking() {
        let pkt = ClientboundContainerClosePacket::new(-1);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundContainerClosePacket::read(&mut read_buf);
        assert_eq!(decoded.get_container_id(), 255);
    }

    #[test]
    fn test_overflow_container_id_masking() {
        let pkt = ClientboundContainerClosePacket::new(256 + 5);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.as_bytes());
        let decoded = ClientboundContainerClosePacket::read(&mut read_buf);
        assert_eq!(decoded.get_container_id(), 5);
    }

    #[test]
    fn test_handle_dispatch_and_type() {
        let pkt = ClientboundContainerClosePacket::new(7);
        assert_eq!(pkt.get_type(), CLIENTBOUND_CONTAINER_CLOSE);
        let mut listener = MockListener {
            called: false,
            container_id: -1,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.container_id, 7);
    }
}
