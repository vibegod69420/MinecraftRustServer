/// Minimal self-contained stub for `net.minecraft.network.protocol.PacketFlow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

/// Minimal self-contained stub for `net.minecraft.resources.Identifier`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

/// Minimal self-contained stub for `net.minecraft.network.protocol.PacketType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType {
    pub flow: PacketFlow,
    pub id: Identifier,
}

impl PacketType {
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self { flow, id }
    }
}

/// Minimal self-contained stub for `net.minecraft.network.FriendlyByteBuf`.
#[derive(Debug, Clone, Default)]
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

    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        let available = self.data.len().saturating_sub(self.pos).min(8);
        if available > 0 {
            bytes[..available].copy_from_slice(&self.data[self.pos..self.pos + available]);
        }
        self.pos = self.pos.saturating_add(available);
        i64::from_be_bytes(bytes)
    }

    pub fn write_long(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

/// Minimal self-contained stub for `net.minecraft.network.codec.StreamCodec`.
pub struct StreamCodec<B, V> {
    pub encode: fn(&V, &mut B),
    pub decode: fn(&mut B) -> V,
}

/// Minimal self-contained stub for `net.minecraft.network.protocol.ping.ClientPongPacketListener`.
pub trait ClientPongPacketListener {
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket);
}

fn clientbound_pong_response_packet_type() -> PacketType {
    PacketType::new(
        PacketFlow::Clientbound,
        Identifier::with_default_namespace("pong_response"),
    )
}

/// Port of `net.minecraft.network.protocol.ping.ClientboundPongResponsePacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPongResponsePacket {
    pub time: i64,
}

impl ClientboundPongResponsePacket {
    pub fn new(time: i64) -> Self {
        Self { time }
    }

    // JAVA: net.minecraft.network.protocol.ping.ClientboundPongResponsePacket#<init>
    fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        Self {
            time: buf.read_long(),
        }
    }

    // JAVA: net.minecraft.network.protocol.ping.ClientboundPongResponsePacket#write
    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_long(self.time);
    }

    // JAVA: net.minecraft.network.protocol.ping.ClientboundPongResponsePacket#type
    pub fn packet_type(&self) -> PacketType {
        clientbound_pong_response_packet_type()
    }

    // JAVA: net.minecraft.network.protocol.ping.ClientboundPongResponsePacket#handle
    pub fn handle<L: ClientPongPacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_pong_response(self);
    }
}

pub static STREAM_CODEC: StreamCodec<FriendlyByteBuf, ClientboundPongResponsePacket> =
    StreamCodec {
        encode: ClientboundPongResponsePacket::write,
        decode: ClientboundPongResponsePacket::from_buf,
    };

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        received: Option<i64>,
    }

    impl ClientPongPacketListener for TestListener {
        fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket) {
            self.received = Some(packet.time);
        }
    }

    #[test]
    fn new_stores_time() {
        let packet = ClientboundPongResponsePacket::new(12345);
        assert_eq!(packet.time, 12345);
    }

    #[test]
    fn write_read_round_trip_preserves_time() {
        let original = ClientboundPongResponsePacket::new(0x0123_4567_89AB_CDEFu64 as i64);
        let mut buf = FriendlyByteBuf::new();
        original.write(&mut buf);
        let mut read_buf = FriendlyByteBuf {
            data: buf.into_inner(),
            pos: 0,
        };
        let decoded = ClientboundPongResponsePacket::from_buf(&mut read_buf);
        assert_eq!(decoded, original);
    }

    #[test]
    fn negative_and_extreme_values_round_trip() {
        for &value in &[i64::MIN, -1, 0, 1, i64::MAX] {
            let packet = ClientboundPongResponsePacket::new(value);
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf {
                data: buf.into_inner(),
                pos: 0,
            };
            let decoded = ClientboundPongResponsePacket::from_buf(&mut read_buf);
            assert_eq!(decoded.time, value);
        }
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = TestListener { received: None };
        let packet = ClientboundPongResponsePacket::new(-42);
        packet.handle(&mut listener);
        assert_eq!(listener.received, Some(-42));
    }

    #[test]
    fn packet_type_is_clientbound_pong_response() {
        let packet = ClientboundPongResponsePacket::new(0);
        let pt = packet.packet_type();
        assert_eq!(pt.flow, PacketFlow::Clientbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "pong_response");
    }

    #[test]
    fn stream_codec_round_trip() {
        let packet = ClientboundPongResponsePacket::new(i64::MIN);
        let mut buf = FriendlyByteBuf::new();
        (STREAM_CODEC.encode)(&packet, &mut buf);
        let mut read_buf = FriendlyByteBuf {
            data: buf.into_inner(),
            pos: 0,
        };
        let decoded = (STREAM_CODEC.decode)(&mut read_buf);
        assert_eq!(decoded, packet);
    }
}
