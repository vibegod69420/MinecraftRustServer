use std::marker::PhantomData;

/// Mirror of `net.minecraft.network.ConnectionProtocol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Configuration,
}

/// Minimal packet registration entry capturing the Java packet type and stream codec identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketRegistration {
    pub packet_type: &'static str,
    pub codec: &'static str,
}

/// Minimal placeholder for `net.minecraft.network.FriendlyByteBuf`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf;

/// Minimal placeholder for `net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServerConfigurationPacketListener;

/// Minimal placeholder for `net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientConfigurationPacketListener;

/// Minimal mirror of `net.minecraft.network.protocol.SimpleUnboundProtocol`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUnboundProtocol<L, B> {
    pub protocol: ConnectionProtocol,
    pub packets: Vec<PacketRegistration>,
    _marker: PhantomData<fn() -> (L, B)>,
}

impl<L, B> SimpleUnboundProtocol<L, B> {
    // JAVA: net.minecraft.network.protocol.SimpleUnboundProtocol#bind
    pub fn bind<F>(&self, _factory: F) -> ProtocolInfo<L>
    where
        F: Fn() -> B,
    {
        ProtocolInfo {
            protocol: self.protocol,
            packets: self.packets.clone(),
            _marker: PhantomData,
        }
    }
}

/// Minimal mirror of `net.minecraft.network.ProtocolInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfo<L> {
    pub protocol: ConnectionProtocol,
    pub packets: Vec<PacketRegistration>,
    _marker: PhantomData<fn() -> L>,
}

/// Minimal mirror of `net.minecraft.network.protocol.ProtocolInfoBuilder`.
#[derive(Debug)]
pub struct ProtocolInfoBuilder<L, B> {
    protocol: ConnectionProtocol,
    packets: Vec<PacketRegistration>,
    _marker: PhantomData<fn() -> (L, B)>,
}

impl<L, B> ProtocolInfoBuilder<L, B> {
    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#serverboundProtocol
    pub fn serverbound_protocol<F>(
        protocol: ConnectionProtocol,
        build: F,
    ) -> SimpleUnboundProtocol<L, B>
    where
        F: FnOnce(&mut Self),
    {
        let mut builder = Self {
            protocol,
            packets: Vec::new(),
            _marker: PhantomData,
        };
        build(&mut builder);
        SimpleUnboundProtocol {
            protocol: builder.protocol,
            packets: builder.packets,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#clientboundProtocol
    pub fn clientbound_protocol<F>(
        protocol: ConnectionProtocol,
        build: F,
    ) -> SimpleUnboundProtocol<L, B>
    where
        F: FnOnce(&mut Self),
    {
        let mut builder = Self {
            protocol,
            packets: Vec::new(),
            _marker: PhantomData,
        };
        build(&mut builder);
        SimpleUnboundProtocol {
            protocol: builder.protocol,
            packets: builder.packets,
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.network.protocol.ProtocolInfoBuilder#addPacket
    pub fn add_packet(&mut self, packet_type: &'static str, codec: &'static str) -> &mut Self {
        self.packets.push(PacketRegistration { packet_type, codec });
        self
    }
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationProtocols#SERVERBOUND_TEMPLATE
pub fn serverbound_template(
) -> SimpleUnboundProtocol<ServerConfigurationPacketListener, FriendlyByteBuf> {
    ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::Configuration, |b| {
        b.add_packet(
            "CommonPacketTypes.SERVERBOUND_CLIENT_INFORMATION",
            "ServerboundClientInformationPacket.STREAM_CODEC",
        )
        .add_packet(
            "CookiePacketTypes.SERVERBOUND_COOKIE_RESPONSE",
            "ServerboundCookieResponsePacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.SERVERBOUND_CUSTOM_PAYLOAD",
            "ServerboundCustomPayloadPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.SERVERBOUND_FINISH_CONFIGURATION",
            "ServerboundFinishConfigurationPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.SERVERBOUND_KEEP_ALIVE",
            "ServerboundKeepAlivePacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.SERVERBOUND_PONG",
            "ServerboundPongPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.SERVERBOUND_RESOURCE_PACK",
            "ServerboundResourcePackPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.SERVERBOUND_SELECT_KNOWN_PACKS",
            "ServerboundSelectKnownPacks.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.SERVERBOUND_CUSTOM_CLICK_ACTION",
            "ServerboundCustomClickActionPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.SERVERBOUND_ACCEPT_CODE_OF_CONDUCT",
            "ServerboundAcceptCodeOfConductPacket.STREAM_CODEC",
        );
    })
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationProtocols#SERVERBOUND
pub fn serverbound() -> ProtocolInfo<ServerConfigurationPacketListener> {
    serverbound_template().bind(FriendlyByteBuf::default)
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationProtocols#CLIENTBOUND_TEMPLATE
pub fn clientbound_template(
) -> SimpleUnboundProtocol<ClientConfigurationPacketListener, FriendlyByteBuf> {
    ProtocolInfoBuilder::clientbound_protocol(ConnectionProtocol::Configuration, |b| {
        b.add_packet(
            "CookiePacketTypes.CLIENTBOUND_COOKIE_REQUEST",
            "ClientboundCookieRequestPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_CUSTOM_PAYLOAD",
            "ClientboundCustomPayloadPacket.CONFIG_STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_DISCONNECT",
            "ClientboundDisconnectPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.CLIENTBOUND_FINISH_CONFIGURATION",
            "ClientboundFinishConfigurationPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_KEEP_ALIVE",
            "ClientboundKeepAlivePacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_PING",
            "ClientboundPingPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.CLIENTBOUND_RESET_CHAT",
            "ClientboundResetChatPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.CLIENTBOUND_REGISTRY_DATA",
            "ClientboundRegistryDataPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_RESOURCE_PACK_POP",
            "ClientboundResourcePackPopPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_RESOURCE_PACK_PUSH",
            "ClientboundResourcePackPushPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_STORE_COOKIE",
            "ClientboundStoreCookiePacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_TRANSFER",
            "ClientboundTransferPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.CLIENTBOUND_UPDATE_ENABLED_FEATURES",
            "ClientboundUpdateEnabledFeaturesPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_UPDATE_TAGS",
            "ClientboundUpdateTagsPacket.STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.CLIENTBOUND_SELECT_KNOWN_PACKS",
            "ClientboundSelectKnownPacks.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_CUSTOM_REPORT_DETAILS",
            "ClientboundCustomReportDetailsPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_SERVER_LINKS",
            "ClientboundServerLinksPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_CLEAR_DIALOG",
            "ClientboundClearDialogPacket.STREAM_CODEC",
        )
        .add_packet(
            "CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG",
            "ClientboundShowDialogPacket.CONTEXT_FREE_STREAM_CODEC",
        )
        .add_packet(
            "ConfigurationPacketTypes.CLIENTBOUND_CODE_OF_CONDUCT",
            "ClientboundCodeOfConductPacket.STREAM_CODEC",
        );
    })
}

// JAVA: net.minecraft.network.protocol.configuration.ConfigurationProtocols#CLIENTBOUND
pub fn clientbound() -> ProtocolInfo<ClientConfigurationPacketListener> {
    clientbound_template().bind(FriendlyByteBuf::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serverbound_template_has_expected_packets_in_order() {
        let template = serverbound_template();
        assert_eq!(template.protocol, ConnectionProtocol::Configuration);
        let packets = &template.packets;
        assert_eq!(packets.len(), 10);
        assert_eq!(
            packets[0],
            PacketRegistration {
                packet_type: "CommonPacketTypes.SERVERBOUND_CLIENT_INFORMATION",
                codec: "ServerboundClientInformationPacket.STREAM_CODEC",
            }
        );
        assert_eq!(
            packets[3],
            PacketRegistration {
                packet_type: "ConfigurationPacketTypes.SERVERBOUND_FINISH_CONFIGURATION",
                codec: "ServerboundFinishConfigurationPacket.STREAM_CODEC",
            }
        );
        assert_eq!(
            packets[9],
            PacketRegistration {
                packet_type: "ConfigurationPacketTypes.SERVERBOUND_ACCEPT_CODE_OF_CONDUCT",
                codec: "ServerboundAcceptCodeOfConductPacket.STREAM_CODEC",
            }
        );
    }

    #[test]
    fn clientbound_template_has_expected_count_and_special_codecs() {
        let template = clientbound_template();
        assert_eq!(template.protocol, ConnectionProtocol::Configuration);
        let packets = &template.packets;
        assert_eq!(packets.len(), 20);
        // Special codec edge case: CONFIG_STREAM_CODEC
        assert_eq!(
            packets[1],
            PacketRegistration {
                packet_type: "CommonPacketTypes.CLIENTBOUND_CUSTOM_PAYLOAD",
                codec: "ClientboundCustomPayloadPacket.CONFIG_STREAM_CODEC",
            }
        );
        // Special codec edge case: CONTEXT_FREE_STREAM_CODEC
        assert_eq!(
            packets[18],
            PacketRegistration {
                packet_type: "CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG",
                codec: "ClientboundShowDialogPacket.CONTEXT_FREE_STREAM_CODEC",
            }
        );
        assert_eq!(
            packets[19].packet_type,
            "ConfigurationPacketTypes.CLIENTBOUND_CODE_OF_CONDUCT"
        );
    }

    #[test]
    fn bind_preserves_protocol_and_packets() {
        let server_template = serverbound_template();
        let server_info = serverbound();
        assert_eq!(server_info.protocol, ConnectionProtocol::Configuration);
        assert_eq!(server_info.packets, server_template.packets);

        let client_template = clientbound_template();
        let client_info = clientbound();
        assert_eq!(client_info.protocol, ConnectionProtocol::Configuration);
        assert_eq!(client_info.packets, client_template.packets);
    }

    #[test]
    fn java_integer_semantics_edge_cases() {
        // Wrapping/overflow semantics for Java int arithmetic.
        let count: i32 = serverbound_template().packets.len() as i32; // 10
        assert_eq!(count.wrapping_add(i32::MAX), i32::MIN.wrapping_add(9));
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        assert_eq!((-1_i32).wrapping_mul(2), -2);

        // Negative inputs and masked shifts (Java shift mask behavior).
        let neg: i32 = -1;
        let shift_count: u32 = 32;
        assert_eq!(neg.wrapping_shl(shift_count & 31), neg);
        let neg_long_shift: i64 = -1;
        assert_eq!(neg_long_shift.wrapping_shr(64_u32 & 63), neg_long_shift);
    }

    #[test]
    fn float_bit_edge_cases() {
        // Exact f32/f64 bit patterns must be preserved.
        let neg_one_f32 = -1.0_f32;
        assert_eq!(neg_one_f32.to_bits(), 0xBF80_0000);
        assert_eq!(f32::from_bits(0x7FC0_0001).to_bits(), 0x7FC0_0001); // NaN payload preserved
        assert_eq!(f32::NEG_INFINITY.to_bits(), 0xFF80_0000);

        let neg_one_f64 = -1.0_f64;
        assert_eq!(neg_one_f64.to_bits(), 0xBFF0_0000_0000_0000);
        assert_eq!(
            f64::from_bits(0x7FF8_0000_0000_0001).to_bits(),
            0x7FF8_0000_0000_0001
        );
    }
}
