use std::marker::PhantomData;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType<T: ?Sized> {
    pub flow: PacketFlow,
    pub id: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> PacketType<T> {
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self {
            flow,
            id,
            _marker: PhantomData,
        }
    }
}

pub trait ServerStatusPacketListener {
    fn handle_status_request(&mut self, packet: &ServerboundStatusRequestPacket);
}

pub type ByteBuf = bytes::BytesMut;

pub struct StreamCodec<B, V> {
    _buf: PhantomData<fn(B)>,
    value: V,
}

impl<B, V: Clone> StreamCodec<B, V> {
    pub fn unit(value: V) -> Self {
        Self {
            _buf: PhantomData,
            value,
        }
    }

    pub fn decode_value(&self) -> V {
        self.value.clone()
    }

    pub fn encode_value(&self, _buf: &mut B, _value: &V) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundStatusRequestPacket(());

impl ServerboundStatusRequestPacket {
    pub const INSTANCE: Self = Self(());

    // JAVA: net.minecraft.network.protocol.status.ServerboundStatusRequestPacket#type
    pub fn packet_type(&self) -> PacketType<Self> {
        PacketType::new(
            PacketFlow::Serverbound,
            Identifier::with_default_namespace("status_request"),
        )
    }

    // JAVA: net.minecraft.network.protocol.status.ServerboundStatusRequestPacket#handle
    pub fn handle<L: ServerStatusPacketListener>(&self, listener: &mut L) {
        listener.handle_status_request(self);
    }
}

pub static STREAM_CODEC: LazyLock<StreamCodec<ByteBuf, ServerboundStatusRequestPacket>> =
    LazyLock::new(|| StreamCodec::unit(ServerboundStatusRequestPacket::INSTANCE));

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: i32,
    }

    impl ServerStatusPacketListener for MockListener {
        fn handle_status_request(&mut self, _packet: &ServerboundStatusRequestPacket) {
            self.called = self.called.wrapping_add(1);
        }
    }

    #[test]
    fn instance_is_singleton_value() {
        let a = ServerboundStatusRequestPacket::INSTANCE;
        let b = ServerboundStatusRequestPacket::INSTANCE;
        assert_eq!(a, b);
    }

    #[test]
    fn packet_type_is_serverbound_status_request() {
        let pt = ServerboundStatusRequestPacket::INSTANCE.packet_type();
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "status_request");
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let mut listener = MockListener { called: 0 };
        ServerboundStatusRequestPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.called, 1);
        ServerboundStatusRequestPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.called, 2);
    }

    #[test]
    fn handle_wrapping_edge_case() {
        let mut listener = MockListener { called: i32::MAX };
        ServerboundStatusRequestPacket::INSTANCE.handle(&mut listener);
        assert_eq!(listener.called, i32::MIN);
    }

    #[test]
    fn stream_codec_unit_decodes_to_instance() {
        let decoded = STREAM_CODEC.decode_value();
        assert_eq!(decoded, ServerboundStatusRequestPacket::INSTANCE);
    }
}
