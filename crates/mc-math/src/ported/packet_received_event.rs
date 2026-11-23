use std::net::SocketAddr;

/// Faithful port of `net.minecraft.util.profiling.jfr.event.PacketReceivedEvent`.
///
/// Base `PacketEvent` fields are inlined for module self-containment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventType {
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketReceivedEvent {
    pub protocol_id: String,
    pub packet_id: String,
    pub flow: String,
    pub remote_address: SocketAddr,
    pub bytes: i32,
}

impl PacketReceivedEvent {
    pub const NAME: &'static str = "minecraft.PacketReceived";
    pub const TYPE: EventType = EventType { name: Self::NAME };

    // JAVA: net.minecraft.util.profiling.jfr.event.PacketReceivedEvent#<init>
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
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), port)
    }

    #[test]
    fn name_and_type_match() {
        assert_eq!(PacketReceivedEvent::NAME, "minecraft.PacketReceived");
        assert_eq!(PacketReceivedEvent::TYPE.name, PacketReceivedEvent::NAME);
    }

    #[test]
    fn constructor_stores_fields() {
        let event = PacketReceivedEvent::new(
            "login".to_string(),
            "minecraft:hello".to_string(),
            "serverbound".to_string(),
            addr_v4(12345),
            128,
        );
        assert_eq!(event.protocol_id, "login");
        assert_eq!(event.packet_id, "minecraft:hello");
        assert_eq!(event.flow, "serverbound");
        assert_eq!(event.remote_address, addr_v4(12345));
        assert_eq!(event.bytes, 128);
    }

    #[test]
    fn supports_negative_and_extreme_byte_counts() {
        let neg = PacketReceivedEvent::new(String::new(), String::new(), String::new(), addr_v4(1), -7);
        let min = PacketReceivedEvent::new(String::new(), String::new(), String::new(), addr_v4(1), i32::MIN);
        let max = PacketReceivedEvent::new(String::new(), String::new(), String::new(), addr_v4(1), i32::MAX);
        assert_eq!(neg.bytes, -7);
        assert_eq!(min.bytes, i32::MIN);
        assert_eq!(max.bytes, i32::MAX);
    }

    #[test]
    fn supports_ipv6_and_clone_equality() {
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 7777);
        let event = PacketReceivedEvent::new("p".into(), "id".into(), "f".into(), addr, 1);
        let cloned = event.clone();
        assert_eq!(event, cloned);
        assert_eq!(event.remote_address, addr);
    }
}
