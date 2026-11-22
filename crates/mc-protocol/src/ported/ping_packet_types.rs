use std::sync::LazyLock;

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

/// Port of `net.minecraft.network.protocol.ping.PingPacketTypes`.
pub struct PingPacketTypes;

impl PingPacketTypes {
    // JAVA: net.minecraft.network.protocol.ping.PingPacketTypes#createClientbound
    fn create_clientbound(name: &str) -> PacketType {
        PacketType::new(
            PacketFlow::Clientbound,
            Identifier::with_default_namespace(name),
        )
    }

    // JAVA: net.minecraft.network.protocol.ping.PingPacketTypes#createServerbound
    fn create_serverbound(name: &str) -> PacketType {
        PacketType::new(
            PacketFlow::Serverbound,
            Identifier::with_default_namespace(name),
        )
    }

    pub fn clientbound_pong_response() -> &'static PacketType {
        static VALUE: LazyLock<PacketType> =
            LazyLock::new(|| PingPacketTypes::create_clientbound("pong_response"));
        &VALUE
    }

    pub fn serverbound_ping_request() -> &'static PacketType {
        static VALUE: LazyLock<PacketType> =
            LazyLock::new(|| PingPacketTypes::create_serverbound("ping_request"));
        &VALUE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clientbound_pong_response_has_correct_type() {
        let pt = PingPacketTypes::clientbound_pong_response();
        assert_eq!(pt.flow, PacketFlow::Clientbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "pong_response");
    }

    #[test]
    fn serverbound_ping_request_has_correct_type() {
        let pt = PingPacketTypes::serverbound_ping_request();
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id.namespace, "minecraft");
        assert_eq!(pt.id.path, "ping_request");
    }

    #[test]
    fn create_clientbound_uses_default_namespace_and_flow() {
        let pt = PingPacketTypes::create_clientbound("custom");
        assert_eq!(pt.flow, PacketFlow::Clientbound);
        assert_eq!(pt.id, Identifier::with_default_namespace("custom"));
    }

    #[test]
    fn create_serverbound_uses_default_namespace_and_flow() {
        let pt = PingPacketTypes::create_serverbound("custom");
        assert_eq!(pt.flow, PacketFlow::Serverbound);
        assert_eq!(pt.id, Identifier::with_default_namespace("custom"));
    }

    #[test]
    fn packet_types_are_distinct_across_flows() {
        let client = PingPacketTypes::create_clientbound("same");
        let server = PingPacketTypes::create_serverbound("same");
        assert_ne!(client, server);
        assert_ne!(
            PingPacketTypes::clientbound_pong_response(),
            PingPacketTypes::serverbound_ping_request()
        );
    }

    #[test]
    fn identifier_default_namespace_is_minecraft() {
        let id = Identifier::with_default_namespace("any_path");
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "any_path");
    }
}
