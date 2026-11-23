#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MobEffectHolder {
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

pub const SERVERBOUND_SET_BEACON: PacketType = PacketType {
    name: "minecraft:set_beacon",
};

pub trait ServerGamePacketListener {
    fn handle_set_beacon_packet(&mut self, packet: &ServerboundSetBeaconPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetBeaconPacket {
    primary: Option<MobEffectHolder>,
    secondary: Option<MobEffectHolder>,
}

impl ServerboundSetBeaconPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetBeaconPacket#<init>
    pub fn new(primary: Option<MobEffectHolder>, secondary: Option<MobEffectHolder>) -> Self {
        Self { primary, secondary }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetBeaconPacket#primary
    pub fn primary(&self) -> Option<MobEffectHolder> {
        self.primary
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetBeaconPacket#secondary
    pub fn secondary(&self) -> Option<MobEffectHolder> {
        self.secondary
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetBeaconPacket#type
    pub fn get_type(&self) -> PacketType {
        SERVERBOUND_SET_BEACON
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetBeaconPacket#handle
    pub fn handle<L: ServerGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_set_beacon_packet(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        primary: Option<MobEffectHolder>,
        secondary: Option<MobEffectHolder>,
    }

    impl MockListener {
        fn new() -> Self {
            Self {
                called: false,
                primary: None,
                secondary: None,
            }
        }
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_set_beacon_packet(&mut self, packet: &ServerboundSetBeaconPacket) {
            self.called = true;
            self.primary = packet.primary();
            self.secondary = packet.secondary();
        }
    }

    #[test]
    fn test_new_and_accessors() {
        let primary = Some(MobEffectHolder { id: 1 });
        let secondary = Some(MobEffectHolder { id: 2 });
        let pkt = ServerboundSetBeaconPacket::new(primary, secondary);
        assert_eq!(pkt.primary(), primary);
        assert_eq!(pkt.secondary(), secondary);
    }

    #[test]
    fn test_none_effects() {
        let pkt = ServerboundSetBeaconPacket::new(None, None);
        assert_eq!(pkt.primary(), None);
        assert_eq!(pkt.secondary(), None);
    }

    #[test]
    fn test_negative_and_extreme_ids() {
        let primary = Some(MobEffectHolder { id: i32::MIN });
        let secondary = Some(MobEffectHolder { id: i32::MAX });
        let pkt = ServerboundSetBeaconPacket::new(primary, secondary);
        assert_eq!(pkt.primary().unwrap().id, i32::MIN);
        assert_eq!(pkt.secondary().unwrap().id, i32::MAX);
    }

    #[test]
    fn test_handle_dispatch() {
        let pkt = ServerboundSetBeaconPacket::new(Some(MobEffectHolder { id: -1 }), None);
        let mut listener = MockListener::new();
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.primary, Some(MobEffectHolder { id: -1 }));
        assert_eq!(listener.secondary, None);
    }

    #[test]
    fn test_packet_type() {
        let pkt = ServerboundSetBeaconPacket::new(None, None);
        assert_eq!(pkt.get_type(), SERVERBOUND_SET_BEACON);
        assert_eq!(pkt.get_type().name, "minecraft:set_beacon");
    }
}
