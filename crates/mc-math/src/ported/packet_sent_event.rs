use std::net::SocketAddr;

/// Faithful port of `net.minecraft.util.profiling.jfr.event.PacketSentEvent`.
///
/// Base `PacketEvent` fields are inlined for module self-containment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventType {
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketSentEvent {
    pub protocol_id: String,
    pub packet_id: String,
    pub flow: String,
    pub remote_address: SocketAddr,
    pub bytes: i32,
}

impl PacketSentEvent {
    pub const NAME: &'static str = "minecraft.PacketSent";
    pub const TYPE: EventType = EventType { name: Self::NAME };

    // JAVA: net.minecraft.util.profiling.jfr.event.PacketSentEvent#<init>
    pub fn new(
        protocol_id: String,
        packet_id: String,
        flow: String,
        remote_address: SocketAddr,
        bytes: i32,
    ) -> Self {
        Self {
            protocol_id,
            packet_id,
            flow,
            remote_address,
            bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn addr_v4(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
    }

    #[test]
    fn name_and_type_match() {
        assert_eq!(PacketSentEvent::NAME, "minecraft.PacketSent");
        assert_eq!(PacketSentEvent::TYPE.name, PacketSentEvent::NAME);
    }

    #[test]
    fn constructor_stores_fields() {
        let event = PacketSentEvent::new(
            "play".to_string(),
            "minecraft:keep_alive".to_string(),
            "clientbound".to_string(),
            addr_v4(25565),
            42,
        );
        assert_eq!(event.protocol_id, "play");
        assert_eq!(event.packet_id, "minecraft:keep_alive");
        assert_eq!(event.flow, "clientbound");
        assert_eq!(event.remote_address, addr_v4(25565));
        assert_eq!(event.bytes, 42);
    }

    #[test]
    fn supports_negative_and_extreme_byte_counts() {
        let neg = PacketSentEvent::new(String::new(), String::new(), String::new(), addr_v4(1), -1);
        let min = PacketSentEvent::new(String::new(), String::new(), String::new(), addr_v4(1), i32::MIN);
        let max = PacketSentEvent::new(String::new(), String::new(), String::new(), addr_v4(1), i32::MAX);
        assert_eq!(neg.bytes, -1);
        assert_eq!(min.bytes, i32::MIN);
        assert_eq!(max.bytes, i32::MAX);
    }

    #[test]
    fn supports_ipv6_and_clone_equality() {
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 9999);
        let event = PacketSentEvent::new("p".into(), "id".into(), "f".into(), addr, 0);
        let cloned = event.clone();
        assert_eq!(event, cloned);
        assert_eq!(event.remote_address, addr);
    }
}
