use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Handshaking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Serverbound,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    pub data: Vec<u8>,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>()
    pub fn new() -> Self {
        Self { data: Vec::new() }
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

#[derive(Debug, Clone)]
pub struct StreamCodec<B, T> {
    _marker: PhantomData<(B, T)>,
}

impl<B, T> StreamCodec<B, T> {
    // JAVA: net.minecraft.network.codec.StreamCodec#<init>
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<B, T> Default for StreamCodec<B, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientIntentionPacket {
    // JAVA: net.minecraft.network.protocol.handshake.ClientIntentionPacket#STREAM_CODEC
    pub fn stream_codec() -> StreamCodec<FriendlyByteBuf, Self> {
        StreamCodec::new()
    }
}

pub struct HandshakePacketTypes;

impl HandshakePacketTypes {
    // JAVA: net.minecraft.network.protocol.handshake.HandshakePacketTypes#CLIENT_INTENTION
    pub fn client_intention() -> PacketType<ClientIntentionPacket> {
        PacketType::new(
            PacketFlow::Serverbound,
            Identifier::with_default_namespace("intention"),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketRegistration {
    pub id: Identifier,
}

pub struct ProtocolBuilder<L, B> {
    registrations: Vec<PacketRegistration>,
    _marker: PhantomData<(L, B)>,
}

impl<L, B> ProtocolBuilder<L, B> {
    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#addPacket(PacketType, StreamCodec)
    pub fn add_packet<T>(
        &mut self,
        packet_type: PacketType<T>,
        _codec: StreamCodec<B, T>,
    ) -> &mut Self {
        self.registrations.push(PacketRegistration {
            id: packet_type.id,
        });
        self
    }
}

#[derive(Debug, Clone)]
pub struct SimpleUnboundProtocol<L, B> {
    pub protocol: ConnectionProtocol,
    pub registrations: Vec<PacketRegistration>,
    _marker: PhantomData<(L, B)>,
}

impl<L, B> SimpleUnboundProtocol<L, B> {
    // JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol#bind(Function)
    pub fn bind<F, B2>(&self, _factory: F) -> ProtocolInfo<L>
    where
        F: FnOnce() -> B2,
    {
        ProtocolInfo {
            protocol: self.protocol,
            registrations: self.registrations.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolInfo<L> {
    pub protocol: ConnectionProtocol,
    pub registrations: Vec<PacketRegistration>,
    _marker: PhantomData<L>,
}

pub struct ProtocolInfoBuilder;

impl ProtocolInfoBuilder {
    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#serverboundProtocol(ConnectionProtocol, Consumer)
    pub fn serverbound_protocol<L, B, F>(
        protocol: ConnectionProtocol,
        config: F,
    ) -> SimpleUnboundProtocol<L, B>
    where
        F: FnOnce(&mut ProtocolBuilder<L, B>),
    {
        let mut builder = ProtocolBuilder {
            registrations: Vec::new(),
            _marker: PhantomData,
        };
        config(&mut builder);
        SimpleUnboundProtocol {
            protocol,
            registrations: builder.registrations,
            _marker: PhantomData,
        }
    }
}

pub struct HandshakeProtocols;

impl HandshakeProtocols {
    // JAVA: net.minecraft.network.protocol.handshake.HandshakeProtocols#SERVERBOUND_TEMPLATE
    pub fn serverbound_template(
    ) -> SimpleUnboundProtocol<ServerHandshakePacketListener, FriendlyByteBuf> {
        ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::Handshaking, |builder| {
            builder.add_packet(
                HandshakePacketTypes::client_intention(),
                ClientIntentionPacket::stream_codec(),
            );
        })
    }

    // JAVA: net.minecraft.network.protocol.handshake.HandshakeProtocols#SERVERBOUND
    pub fn serverbound() -> ProtocolInfo<ServerHandshakePacketListener> {
        Self::serverbound_template().bind(FriendlyByteBuf::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serverbound_template_uses_handshaking_protocol() {
        let template = HandshakeProtocols::serverbound_template();
        assert_eq!(template.protocol, ConnectionProtocol::Handshaking);
    }

    #[test]
    fn serverbound_template_registers_intention_packet() {
        let template = HandshakeProtocols::serverbound_template();
        assert_eq!(template.registrations.len(), 1);
        assert_eq!(template.registrations[0].id.namespace, "minecraft");
        assert_eq!(template.registrations[0].id.path, "intention");
    }

    #[test]
    fn serverbound_bind_preserves_protocol_and_registrations() {
        let info = HandshakeProtocols::serverbound();
        assert_eq!(info.protocol, ConnectionProtocol::Handshaking);
        assert_eq!(info.registrations.len(), 1);
        assert_eq!(info.registrations[0].id.path, "intention");
    }

    #[test]
    fn friendly_byte_buf_new_is_empty() {
        let buf = FriendlyByteBuf::new();
        assert!(buf.data.is_empty());
        let buf_default = FriendlyByteBuf::default();
        assert_eq!(buf, buf_default);
    }

    #[test]
    fn client_intention_packet_type_is_serverbound() {
        let pt = HandshakePacketTypes::client_intention();
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id, Identifier::with_default_namespace("intention"));
    }
}
