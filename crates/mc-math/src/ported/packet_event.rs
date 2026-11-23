#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketEvent {
    pub protocol_id: String,
    pub packet_direction: String,
    pub packet_id: String,
    pub remote_address: String,
    pub bytes: i32,
}

impl PacketEvent {
    // JAVA: net.minecraft.util.profiling.jfr.event.PacketEvent#<init>
    pub fn new(
        protocol_id: String,
        packet_direction: String,
        packet_id: String,
        remote_address: impl ToString,
        bytes: i32,
    ) -> Self {
        Self {
            protocol_id,
            packet_direction,
            packet_id,
            remote_address: remote_address.to_string(),
            bytes,
        }
    }
}

pub struct PacketEventFields;

impl PacketEventFields {
    pub const REMOTE_ADDRESS: &'static str = "remoteAddress";
    pub const PROTOCOL_ID: &'static str = "protocolId";
    pub const PACKET_DIRECTION: &'static str = "packetDirection";
    pub const PACKET_ID: &'static str = "packetId";
    pub const BYTES: &'static str = "bytes";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn constructor_stores_fields() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 25565);
        let ev = PacketEvent::new(
            "play".to_string(),
            "c2s".to_string(),
            "minecraft:chat".to_string(),
            addr,
            128,
        );
        assert_eq!(ev.protocol_id, "play");
        assert_eq!(ev.packet_direction, "c2s");
        assert_eq!(ev.packet_id, "minecraft:chat");
        assert_eq!(ev.remote_address, addr.to_string());
        assert_eq!(ev.bytes, 128);
    }

    #[test]
    fn field_constants() {
        assert_eq!(PacketEventFields::REMOTE_ADDRESS, "remoteAddress");
        assert_eq!(PacketEventFields::PROTOCOL_ID, "protocolId");
        assert_eq!(PacketEventFields::PACKET_DIRECTION, "packetDirection");
        assert_eq!(PacketEventFields::PACKET_ID, "packetId");
        assert_eq!(PacketEventFields::BYTES, "bytes");
    }

    #[test]
    fn remote_address_uses_to_string() {
        let ev = PacketEvent::new(
            "p".to_string(),
            "s2c".to_string(),
            "id".to_string(),
            "127.0.0.1:1234",
            0,
        );
        assert_eq!(ev.remote_address, "127.0.0.1:1234");
    }

    #[test]
    fn bytes_edge_cases() {
        let mk = |bytes: i32| {
            PacketEvent::new("p".into(), "d".into(), "i".into(), "addr", bytes)
        };
        assert_eq!(mk(-1).bytes, -1);
        assert_eq!(mk(0).bytes, 0);
        assert_eq!(mk(i32::MIN).bytes, i32::MIN);
        assert_eq!(mk(i32::MAX).bytes, i32::MAX);
    }
}
