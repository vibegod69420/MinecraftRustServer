use std::io;

#[derive(Debug, Clone, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.reader_index)
    }

    fn read_exact(&mut self, dst: &mut [u8]) -> io::Result<()> {
        if self.remaining() < dst.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_exact"));
        }
        let start = self.reader_index;
        let end = start + dst.len();
        dst.copy_from_slice(&self.data[start..end]);
        self.reader_index = end;
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readInt
    pub fn read_int(&mut self) -> io::Result<i32> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        Ok(i32::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeInt
    pub fn write_int(&mut self, value: i32) -> io::Result<()> {
        self.data.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub const CLIENTBOUND_PING: PacketType = PacketType {
    id: "minecraft:common/ping",
};

pub trait ClientCommonPacketListener {
    fn handle_ping(&mut self, packet: &ClientboundPingPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPingPacket {
    id: i32,
}

impl ClientboundPingPacket {
    // JAVA: net.minecraft.network.protocol.common.ClientboundPingPacket#<init>
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundPingPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_int()?;
        Ok(Self { id })
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundPingPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_int(self.id)?;
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundPingPacket#type
    pub fn type_(&self) -> PacketType {
        CLIENTBOUND_PING
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundPingPacket#handle
    pub fn handle<L: ClientCommonPacketListener>(&self, listener: &mut L) {
        listener.handle_ping(self);
    }

    // JAVA: net.minecraft.network.protocol.common.ClientboundPingPacket#getId
    pub fn get_id(&self) -> i32 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_ping_packet() {
        let pkt = ClientboundPingPacket::new(42);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPingPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_id(), 42);
        assert_eq!(pkt.type_(), CLIENTBOUND_PING);
    }

    #[test]
    fn ping_packet_negative_id() {
        let pkt = ClientboundPingPacket::new(-1);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundPingPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_id(), -1);
    }

    #[test]
    fn ping_packet_int_extremes() {
        for v in [i32::MIN, i32::MAX, 0, -123456789] {
            let pkt = ClientboundPingPacket::new(v);
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::from_bytes(buf.into_inner());
            let decoded = ClientboundPingPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_id(), v);
        }
    }

    #[test]
    fn ping_packet_wire_format_big_endian() {
        let pkt = ClientboundPingPacket::new(0x0102_0304);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf).unwrap();
        assert_eq!(buf.into_inner(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn ping_packet_handle_dispatch() {
        struct TestListener {
            id: i32,
        }
        impl ClientCommonPacketListener for TestListener {
            fn handle_ping(&mut self, packet: &ClientboundPingPacket) {
                self.id = packet.get_id();
            }
        }
        let mut listener = TestListener { id: 0 };
        ClientboundPingPacket::new(99).handle(&mut listener);
        assert_eq!(listener.id, 99);
    }
}
