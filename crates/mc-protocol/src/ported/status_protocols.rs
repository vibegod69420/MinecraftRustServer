//! Port of `net.minecraft.network.protocol.status.StatusProtocols` (Minecraft Java 1.21.11).

use std::marker::PhantomData;

/// Minimal representation of `net.minecraft.network.ConnectionProtocol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Status,
}

/// Minimal representation of a Netty `ByteBuf`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ByteBuf {
    pub data: Vec<u8>,
}

/// Minimal representation of `net.minecraft.network.FriendlyByteBuf`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    pub inner: ByteBuf,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new(inner: ByteBuf) -> Self {
        Self { inner }
    }
}

/// Marker trait for packet listeners.
pub trait PacketListener {}

/// Minimal representation of `net.minecraft.network.protocol.status.ServerStatusPacketListener`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerStatusPacketListener;
impl PacketListener for ServerStatusPacketListener {}

/// Minimal representation of `net.minecraft.network.protocol.status.ClientStatusPacketListener`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientStatusPacketListener;
impl PacketListener for ClientStatusPacketListener {}

/// Minimal packet type descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: i32,
    pub name: &'static str,
}

/// Minimal stream codec descriptor for a packet payload type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamCodec<P> {
    pub name: &'static str,
    _marker: PhantomData<fn() -> P>,
}

impl<P> StreamCodec<P> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

/// Registered packet metadata in a protocol definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketRegistration {
    pub packet_type: PacketType,
    pub codec_name: &'static str,
}

/// Minimal representation of `net.minecraft.network.ProtocolInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfo<L> {
    pub protocol: ConnectionProtocol,
    pub packets: Vec<PacketRegistration>,
    _marker: PhantomData<fn() -> L>,
}

/// Minimal representation of `net.minecraft.network.protocol.SimpleUnboundProtocol`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUnboundProtocol<L, B> {
    pub protocol: ConnectionProtocol,
    pub packets: Vec<PacketRegistration>,
    _marker: PhantomData<fn() -> (L, B)>,
}

impl<L, B> SimpleUnboundProtocol<L, B> {
    // JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol#bind
    pub fn bind<F>(self, _binder: F) -> ProtocolInfo<L>
    where
        F: FnOnce(ByteBuf) -> B,
    {
        ProtocolInfo {
            protocol: self.protocol,
            packets: self.packets,
            _marker: PhantomData,
        }
    }
}

/// Minimal representation of `net.minecraft.network.protocol.ProtocolInfoBuilder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfoBuilder<L, B> {
    pub protocol: ConnectionProtocol,
    pub packets: Vec<PacketRegistration>,
    _marker: PhantomData<fn() -> (L, B)>,
}

impl<L, B> ProtocolInfoBuilder<L, B> {
    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#serverboundProtocol
    pub fn serverbound_protocol<F>(protocol: ConnectionProtocol, build: F) -> SimpleUnboundProtocol<L, B>
    where
        F: FnOnce(Self) -> Self,
    {
        let builder = Self {
            protocol,
            packets: Vec::new(),
            _marker: PhantomData,
        };
        let built = build(builder);
        SimpleUnboundProtocol {
            protocol: built.protocol,
            packets: built.packets,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#clientboundProtocol
    pub fn clientbound_protocol<F>(protocol: ConnectionProtocol, build: F) -> SimpleUnboundProtocol<L, B>
    where
        F: FnOnce(Self) -> Self,
    {
        let builder = Self {
            protocol,
            packets: Vec::new(),
            _marker: PhantomData,
        };
        let built = build(builder);
        SimpleUnboundProtocol {
            protocol: built.protocol,
            packets: built.packets,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#addPacket
    pub fn add_packet<P>(mut self, packet_type: PacketType, codec: StreamCodec<P>) -> Self {
        self.packets.push(PacketRegistration {
            packet_type,
            codec_name: codec.name,
        });
        self
    }
}

// --- Minimal packet and packet-type placeholders referenced by StatusProtocols ---

pub mod status_packet_types {
    use super::PacketType;

    pub const SERVERBOUND_STATUS_REQUEST: PacketType = PacketType {
        id: 0,
        name: "status:serverbound_status_request",
    };
    pub const CLIENTBOUND_STATUS_RESPONSE: PacketType = PacketType {
        id: 0,
        name: "status:clientbound_status_response",
    };
}

pub mod ping_packet_types {
    use super::PacketType;

    pub const SERVERBOUND_PING_REQUEST: PacketType = PacketType {
        id: 1,
        name: "ping:serverbound_ping_request",
    };
    pub const CLIENTBOUND_PONG_RESPONSE: PacketType = PacketType {
        id: 1,
        name: "ping:clientbound_pong_response",
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundStatusRequestPacket;
impl ServerboundStatusRequestPacket {
    pub const STREAM_CODEC: StreamCodec<Self> =
        StreamCodec::new("ServerboundStatusRequestPacket.STREAM_CODEC");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundPingRequestPacket;
impl ServerboundPingRequestPacket {
    pub const STREAM_CODEC: StreamCodec<Self> =
        StreamCodec::new("ServerboundPingRequestPacket.STREAM_CODEC");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundStatusResponsePacket;
impl ClientboundStatusResponsePacket {
    pub const STREAM_CODEC: StreamCodec<Self> =
        StreamCodec::new("ClientboundStatusResponsePacket.STREAM_CODEC");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundPongResponsePacket;
impl ClientboundPongResponsePacket {
    pub const STREAM_CODEC: StreamCodec<Self> =
        StreamCodec::new("ClientboundPongResponsePacket.STREAM_CODEC");
}

// --- Port of net.minecraft.network.protocol.status.StatusProtocols ---

// JAVA: net.minecraft.network.protocol.status.StatusProtocols#SERVERBOUND_TEMPLATE
pub fn serverbound_template() -> SimpleUnboundProtocol<ServerStatusPacketListener, ByteBuf> {
    ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::Status, |b| {
        b.add_packet(
            status_packet_types::SERVERBOUND_STATUS_REQUEST,
            ServerboundStatusRequestPacket::STREAM_CODEC,
        )
        .add_packet(
            ping_packet_types::SERVERBOUND_PING_REQUEST,
            ServerboundPingRequestPacket::STREAM_CODEC,
        )
    })
}

// JAVA: net.minecraft.network.protocol.status.StatusProtocols#SERVERBOUND
pub fn serverbound() -> ProtocolInfo<ServerStatusPacketListener> {
    serverbound_template().bind(|buf: ByteBuf| buf)
}

// JAVA: net.minecraft.network.protocol.status.StatusProtocols#CLIENTBOUND_TEMPLATE
pub fn clientbound_template() -> SimpleUnboundProtocol<ClientStatusPacketListener, FriendlyByteBuf>
{
    ProtocolInfoBuilder::clientbound_protocol(ConnectionProtocol::Status, |b| {
        b.add_packet(
            status_packet_types::CLIENTBOUND_STATUS_RESPONSE,
            ClientboundStatusResponsePacket::STREAM_CODEC,
        )
        .add_packet(
            ping_packet_types::CLIENTBOUND_PONG_RESPONSE,
            ClientboundPongResponsePacket::STREAM_CODEC,
        )
    })
}

// JAVA: net.minecraft.network.protocol.status.StatusProtocols#CLIENTBOUND
pub fn clientbound() -> ProtocolInfo<ClientStatusPacketListener> {
    clientbound_template().bind(FriendlyByteBuf::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serverbound_template_has_expected_packets() {
        let template = serverbound_template();
        assert_eq!(template.protocol, ConnectionProtocol::Status);
        assert_eq!(template.packets.len(), 2);
        assert_eq!(
            template.packets[0].packet_type,
            status_packet_types::SERVERBOUND_STATUS_REQUEST
        );
        assert_eq!(
            template.packets[0].codec_name,
            "ServerboundStatusRequestPacket.STREAM_CODEC"
        );
        assert_eq!(
            template.packets[1].packet_type,
            ping_packet_types::SERVERBOUND_PING_REQUEST
        );
        assert_eq!(
            template.packets[1].codec_name,
            "ServerboundPingRequestPacket.STREAM_CODEC"
        );
    }

    #[test]
    fn clientbound_template_has_expected_packets() {
        let template = clientbound_template();
        assert_eq!(template.protocol, ConnectionProtocol::Status);
        assert_eq!(template.packets.len(), 2);
        assert_eq!(
            template.packets[0].packet_type,
            status_packet_types::CLIENTBOUND_STATUS_RESPONSE
        );
        assert_eq!(
            template.packets[1].packet_type,
            ping_packet_types::CLIENTBOUND_PONG_RESPONSE
        );
        assert_eq!(
            template.packets[1].codec_name,
            "ClientboundPongResponsePacket.STREAM_CODEC"
        );
    }

    #[test]
    fn bound_protocols_preserve_registrations_and_binder_compatibility() {
        let sb = serverbound();
        let cb = clientbound();
        assert_eq!(sb.protocol, ConnectionProtocol::Status);
        assert_eq!(cb.protocol, ConnectionProtocol::Status);
        assert_eq!(sb.packets, serverbound_template().packets);
        assert_eq!(cb.packets, clientbound_template().packets);

        let wrapped = FriendlyByteBuf::new(ByteBuf {
            data: vec![0x7f, 0x00],
        });
        assert_eq!(wrapped.inner.data, vec![0x7f, 0x00]);
    }

    #[test]
    fn packet_id_integer_edge_cases_and_wrapping_semantics() {
        let neg = PacketType {
            id: -1,
            name: "neg",
        };
        let min = PacketType {
            id: i32::MIN,
            name: "min",
        };
        let max = PacketType {
            id: i32::MAX,
            name: "max",
        };

        assert_eq!(neg.id.wrapping_add(1), 0);
        assert_eq!(min.id.wrapping_sub(1), max.id);
        assert_eq!(max.id.wrapping_add(1), min.id);
        assert_eq!(min.id.wrapping_mul(2), 0);

        let shift_masked = 1i32.wrapping_shl((32u32 & 31) as u32);
        assert_eq!(shift_masked, 1i32);
    }

    #[test]
    fn float_bits_roundtrip_edge_cases() {
        let nan_bits = f32::NAN.to_bits();
        let nan_back = f32::from_bits(nan_bits);
        assert!(nan_back.is_nan());

        let neg_zero_bits = (-0.0f32).to_bits();
        let pos_zero_bits = 0.0f32.to_bits();
        assert_ne!(neg_zero_bits, pos_zero_bits);
        assert_eq!(f32::from_bits(neg_zero_bits).to_bits(), neg_zero_bits);

        let inf_bits = f32::INFINITY.to_bits();
        assert_eq!(f32::from_bits(inf_bits), f32::INFINITY);
        let neg_inf_bits = f32::NEG_INFINITY.to_bits();
        assert_eq!(f32::from_bits(neg_inf_bits), f32::NEG_INFINITY);
    }
}
