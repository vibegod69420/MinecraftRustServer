use std::marker::PhantomData;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Serverbound,
    Clientbound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    // JAVA: net.minecraft.resources.Identifier#withDefaultNamespace(String)
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            path: path.to_string(),
        }
    }
}

pub struct ServerHandshakePacketListener;

pub trait Packet<L> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketType<T> {
    pub flow: PacketFlow,
    pub id: Identifier,
    _marker: PhantomData<T>,
}

impl<T> PacketType<T> {
    // JAVA: net.minecraft.network.protocol.PacketType#<init>(PacketFlow, Identifier)
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self {
            flow,
            id,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIntentionPacket;

impl Packet<ServerHandshakePacketListener> for ClientIntentionPacket {}

pub struct HandshakePacketTypes;

impl HandshakePacketTypes {
    // JAVA: net.minecraft.network.protocol.handshake.HandshakePacketTypes#createServerbound(String)
    fn create_serverbound<T: Packet<ServerHandshakePacketListener>>(name: &str) -> PacketType<T> {
        PacketType::new(
            PacketFlow::Serverbound,
            Identifier::with_default_namespace(name),
        )
    }

    // JAVA: net.minecraft.network.protocol.handshake.HandshakePacketTypes#CLIENT_INTENTION
    pub fn client_intention() -> &'static PacketType<ClientIntentionPacket> {
        static CLIENT_INTENTION: OnceLock<PacketType<ClientIntentionPacket>> = OnceLock::new();
        CLIENT_INTENTION.get_or_init(|| Self::create_serverbound("intention"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_intention_has_correct_id_and_flow() {
        let pt = HandshakePacketTypes::client_intention();
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "intention");
    }

    #[test]
    fn with_default_namespace_uses_minecraft_namespace() {
        let id = Identifier::with_default_namespace("intention");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "intention");
    }

    #[test]
    fn client_intention_is_stable_singleton() {
        let a = HandshakePacketTypes::client_intention();
        let b = HandshakePacketTypes::client_intention();
        assert_eq!(a, b);
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn packet_type_new_preserves_fields() {
        let id = Identifier::with_default_namespace("custom");
        let pt: PacketType<ClientIntentionPacket> =
            PacketType::new(PacketFlow::Clientbound, id.clone());
        assert_eq!(pt.flow, PacketFlow::Clientbound);
        assert_eq!(pt.id, id);
    }

    #[test]
    fn create_serverbound_produces_serverbound_flow() {
        let pt: PacketType<ClientIntentionPacket> =
            HandshakePacketTypes::create_serverbound("test_packet");
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "test_packet");
    }
}
