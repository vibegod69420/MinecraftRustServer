use std::marker::PhantomData;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Login,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf;

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamCodec(pub &'static str);

pub struct ServerLoginPacketListenerMarker;
pub struct ClientLoginPacketListenerMarker;

#[derive(Debug, Clone)]
pub struct ProtocolBuilder<L> {
    protocol: ConnectionProtocol,
    packets: Vec<(PacketType, StreamCodec)>,
    _listener: PhantomData<fn() -> L>,
}

impl<L> ProtocolBuilder<L> {
    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#addPacket
    pub fn add_packet(mut self, packet_type: PacketType, codec: StreamCodec) -> Self {
        self.packets.push((packet_type, codec));
        self
    }

    pub fn packets(&self) -> &[(PacketType, StreamCodec)] {
        &self.packets
    }

    pub fn protocol(&self) -> ConnectionProtocol {
        self.protocol
    }
}

#[derive(Debug, Clone)]
pub struct SimpleUnboundProtocol<L, B> {
    protocol: ConnectionProtocol,
    packets: Vec<(PacketType, StreamCodec)>,
    _listener: PhantomData<fn() -> L>,
    _buf: PhantomData<fn() -> B>,
}

impl<L, B> SimpleUnboundProtocol<L, B> {
    // JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol#bind
    pub fn bind<F>(&self, _buf_factory: F) -> ProtocolInfo<L>
    where
        F: Fn() -> B,
    {
        ProtocolInfo {
            protocol: self.protocol,
            packets: self.packets.clone(),
            _listener: PhantomData,
        }
    }

    pub fn protocol(&self) -> ConnectionProtocol {
        self.protocol
    }

    pub fn packets(&self) -> &[(PacketType, StreamCodec)] {
        &self.packets
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolInfo<L> {
    protocol: ConnectionProtocol,
    packets: Vec<(PacketType, StreamCodec)>,
    _listener: PhantomData<fn() -> L>,
}

impl<L> ProtocolInfo<L> {
    pub fn protocol(&self) -> ConnectionProtocol {
        self.protocol
    }

    pub fn packets(&self) -> &[(PacketType, StreamCodec)] {
        &self.packets
    }
}

pub struct ProtocolInfoBuilder;

impl ProtocolInfoBuilder {
    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#serverboundProtocol
    pub fn serverbound_protocol<L, F>(
        protocol: ConnectionProtocol,
        f: F,
    ) -> SimpleUnboundProtocol<L, FriendlyByteBuf>
    where
        F: FnOnce(ProtocolBuilder<L>) -> ProtocolBuilder<L>,
    {
        let builder = ProtocolBuilder {
            protocol,
            packets: Vec::new(),
            _listener: PhantomData,
        };
        let built = f(builder);
        SimpleUnboundProtocol {
            protocol: built.protocol,
            packets: built.packets,
            _listener: PhantomData,
            _buf: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#clientboundProtocol
    pub fn clientbound_protocol<L, F>(
        protocol: ConnectionProtocol,
        f: F,
    ) -> SimpleUnboundProtocol<L, FriendlyByteBuf>
    where
        F: FnOnce(ProtocolBuilder<L>) -> ProtocolBuilder<L>,
    {
        let builder = ProtocolBuilder {
            protocol,
            packets: Vec::new(),
            _listener: PhantomData,
        };
        let built = f(builder);
        SimpleUnboundProtocol {
            protocol: built.protocol,
            packets: built.packets,
            _listener: PhantomData,
            _buf: PhantomData,
        }
    }
}

pub struct LoginPacketTypes;
impl LoginPacketTypes {
    pub const SERVERBOUND_HELLO: PacketType = PacketType("minecraft:hello_c2s");
    pub const SERVERBOUND_KEY: PacketType = PacketType("minecraft:key_c2s");
    pub const SERVERBOUND_CUSTOM_QUERY_ANSWER: PacketType =
        PacketType("minecraft:custom_query_answer_c2s");
    pub const SERVERBOUND_LOGIN_ACKNOWLEDGED: PacketType =
        PacketType("minecraft:login_acknowledged_c2s");
    pub const CLIENTBOUND_LOGIN_DISCONNECT: PacketType =
        PacketType("minecraft:login_disconnect_s2c");
    pub const CLIENTBOUND_HELLO: PacketType = PacketType("minecraft:hello_s2c");
    pub const CLIENTBOUND_LOGIN_FINISHED: PacketType =
        PacketType("minecraft:login_finished_s2c");
    pub const CLIENTBOUND_LOGIN_COMPRESSION: PacketType =
        PacketType("minecraft:login_compression_s2c");
    pub const CLIENTBOUND_CUSTOM_QUERY: PacketType = PacketType("minecraft:custom_query_s2c");
}

pub struct CookiePacketTypes;
impl CookiePacketTypes {
    pub const SERVERBOUND_COOKIE_RESPONSE: PacketType =
        PacketType("minecraft:cookie_response_c2s");
    pub const CLIENTBOUND_COOKIE_REQUEST: PacketType = PacketType("minecraft:cookie_request_s2c");
}

macro_rules! packet_stub {
    ($name:ident) => {
        pub struct $name;
        impl $name {
            pub const STREAM_CODEC: StreamCodec = StreamCodec(stringify!($name));
        }
    };
}

packet_stub!(ServerboundHelloPacket);
packet_stub!(ServerboundKeyPacket);
packet_stub!(ServerboundCustomQueryAnswerPacket);
packet_stub!(ServerboundLoginAcknowledgedPacket);
packet_stub!(ServerboundCookieResponsePacket);
packet_stub!(ClientboundLoginDisconnectPacket);
packet_stub!(ClientboundHelloPacket);
packet_stub!(ClientboundLoginFinishedPacket);
packet_stub!(ClientboundLoginCompressionPacket);
packet_stub!(ClientboundCustomQueryPacket);
packet_stub!(ClientboundCookieRequestPacket);

pub struct LoginProtocols;

// JAVA: net.minecraft.network.protocol.login.LoginProtocols#SERVERBOUND_TEMPLATE
pub static SERVERBOUND_TEMPLATE: LazyLock<
    SimpleUnboundProtocol<ServerLoginPacketListenerMarker, FriendlyByteBuf>,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::Login, |b| {
        b.add_packet(
            LoginPacketTypes::SERVERBOUND_HELLO,
            ServerboundHelloPacket::STREAM_CODEC,
        )
        .add_packet(
            LoginPacketTypes::SERVERBOUND_KEY,
            ServerboundKeyPacket::STREAM_CODEC,
        )
        .add_packet(
            LoginPacketTypes::SERVERBOUND_CUSTOM_QUERY_ANSWER,
            ServerboundCustomQueryAnswerPacket::STREAM_CODEC,
        )
        .add_packet(
            LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED,
            ServerboundLoginAcknowledgedPacket::STREAM_CODEC,
        )
        .add_packet(
            CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE,
            ServerboundCookieResponsePacket::STREAM_CODEC,
        )
    })
});

// JAVA: net.minecraft.network.protocol.login.LoginProtocols#SERVERBOUND
pub static SERVERBOUND: LazyLock<ProtocolInfo<ServerLoginPacketListenerMarker>> =
    LazyLock::new(|| SERVERBOUND_TEMPLATE.bind(FriendlyByteBuf::new));

// JAVA: net.minecraft.network.protocol.login.LoginProtocols#CLIENTBOUND_TEMPLATE
pub static CLIENTBOUND_TEMPLATE: LazyLock<
    SimpleUnboundProtocol<ClientLoginPacketListenerMarker, FriendlyByteBuf>,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::clientbound_protocol(ConnectionProtocol::Login, |b| {
        b.add_packet(
            LoginPacketTypes::CLIENTBOUND_LOGIN_DISCONNECT,
            ClientboundLoginDisconnectPacket::STREAM_CODEC,
        )
        .add_packet(
            LoginPacketTypes::CLIENTBOUND_HELLO,
            ClientboundHelloPacket::STREAM_CODEC,
        )
        .add_packet(
            LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED,
            ClientboundLoginFinishedPacket::STREAM_CODEC,
        )
        .add_packet(
            LoginPacketTypes::CLIENTBOUND_LOGIN_COMPRESSION,
            ClientboundLoginCompressionPacket::STREAM_CODEC,
        )
        .add_packet(
            LoginPacketTypes::CLIENTBOUND_CUSTOM_QUERY,
            ClientboundCustomQueryPacket::STREAM_CODEC,
        )
        .add_packet(
            CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST,
            ClientboundCookieRequestPacket::STREAM_CODEC,
        )
    })
});

// JAVA: net.minecraft.network.protocol.login.LoginProtocols#CLIENTBOUND
pub static CLIENTBOUND: LazyLock<ProtocolInfo<ClientLoginPacketListenerMarker>> =
    LazyLock::new(|| CLIENTBOUND_TEMPLATE.bind(FriendlyByteBuf::new));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serverbound_template_has_expected_packets() {
        let t = &*SERVERBOUND_TEMPLATE;
        assert_eq!(t.protocol(), ConnectionProtocol::Login);
        let packets = t.packets();
        assert_eq!(packets.len(), 5);
        assert_eq!(packets[0].0, LoginPacketTypes::SERVERBOUND_HELLO);
        assert_eq!(packets[1].0, LoginPacketTypes::SERVERBOUND_KEY);
        assert_eq!(
            packets[2].0,
            LoginPacketTypes::SERVERBOUND_CUSTOM_QUERY_ANSWER
        );
        assert_eq!(packets[3].0, LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED);
        assert_eq!(packets[4].0, CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE);
    }

    #[test]
    fn clientbound_template_has_expected_packets() {
        let t = &*CLIENTBOUND_TEMPLATE;
        assert_eq!(t.protocol(), ConnectionProtocol::Login);
        let packets = t.packets();
        assert_eq!(packets.len(), 6);
        assert_eq!(packets[0].0, LoginPacketTypes::CLIENTBOUND_LOGIN_DISCONNECT);
        assert_eq!(packets[1].0, LoginPacketTypes::CLIENTBOUND_HELLO);
        assert_eq!(packets[2].0, LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED);
        assert_eq!(
            packets[3].0,
            LoginPacketTypes::CLIENTBOUND_LOGIN_COMPRESSION
        );
        assert_eq!(packets[4].0, LoginPacketTypes::CLIENTBOUND_CUSTOM_QUERY);
        assert_eq!(packets[5].0, CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST);
    }

    #[test]
    fn bound_serverbound_preserves_packets() {
        let info = &*SERVERBOUND;
        assert_eq!(info.protocol(), ConnectionProtocol::Login);
        assert_eq!(info.packets().len(), 5);
        assert_eq!(
            info.packets()[2].1,
            ServerboundCustomQueryAnswerPacket::STREAM_CODEC
        );
    }

    #[test]
    fn bound_clientbound_preserves_packets() {
        let info = &*CLIENTBOUND;
        assert_eq!(info.protocol(), ConnectionProtocol::Login);
        assert_eq!(info.packets().len(), 6);
        assert_eq!(
            info.packets()[5].0,
            CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST
        );
    }

    #[test]
    fn packet_codecs_match_registered_types() {
        let s = SERVERBOUND_TEMPLATE.packets();
        assert_eq!(s[0].1, ServerboundHelloPacket::STREAM_CODEC);
        assert_eq!(s[4].1, ServerboundCookieResponsePacket::STREAM_CODEC);
        let c = CLIENTBOUND_TEMPLATE.packets();
        assert_eq!(c[0].1, ClientboundLoginDisconnectPacket::STREAM_CODEC);
        assert_eq!(c[5].1, ClientboundCookieRequestPacket::STREAM_CODEC);
    }

    #[test]
    fn all_serverbound_packet_types_are_distinct() {
        let packets = SERVERBOUND_TEMPLATE.packets();
        for i in 0..packets.len() {
            for j in (i + 1)..packets.len() {
                assert_ne!(
                    packets[i].0, packets[j].0,
                    "duplicate packet type at {i} and {j}"
                );
            }
        }
    }
}
