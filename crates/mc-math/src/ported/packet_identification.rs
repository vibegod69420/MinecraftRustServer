/// Minimal view over a JFR `RecordedEvent` for `PacketIdentification` extraction.
pub trait PacketIdentificationEvent {
    fn get_string(&self, name: &str) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketIdentification {
    pub direction: String,
    pub protocol_id: String,
    pub packet_id: String,
}

impl PacketIdentification {
    // JAVA: net.minecraft.util.profiling.jfr.stats.PacketIdentification#from
    pub fn from<E: PacketIdentificationEvent + ?Sized>(event: &E) -> Self {
        Self {
            direction: event.get_string("packetDirection"),
            protocol_id: event.get_string("protocolId"),
            packet_id: event.get_string("packetId"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEvent {
        direction: String,
        protocol_id: String,
        packet_id: String,
    }

    impl PacketIdentificationEvent for MockEvent {
        fn get_string(&self, name: &str) -> String {
            match name {
                "packetDirection" => self.direction.clone(),
                "protocolId" => self.protocol_id.clone(),
                "packetId" => self.packet_id.clone(),
                _ => panic!("unexpected field: {name}"),
            }
        }
    }

    #[test]
    fn from_basic_values() {
        let stat = PacketIdentification::from(&MockEvent {
            direction: "clientbound".into(),
            protocol_id: "play".into(),
            packet_id: "minecraft:keep_alive".into(),
        });
        assert_eq!(stat.direction, "clientbound");
        assert_eq!(stat.protocol_id, "play");
        assert_eq!(stat.packet_id, "minecraft:keep_alive");
    }

    #[test]
    fn from_empty_strings() {
        let stat = PacketIdentification::from(&MockEvent {
            direction: String::new(),
            protocol_id: String::new(),
            packet_id: String::new(),
        });
        assert!(stat.direction.is_empty());
        assert!(stat.protocol_id.is_empty());
        assert!(stat.packet_id.is_empty());
    }

    #[test]
    fn from_unicode_strings() {
        let stat = PacketIdentification::from(&MockEvent {
            direction: "方向".into(),
            protocol_id: "协议".into(),
            packet_id: "数据包".into(),
        });
        assert_eq!(stat.direction, "方向");
        assert_eq!(stat.protocol_id, "协议");
        assert_eq!(stat.packet_id, "数据包");
    }
}
