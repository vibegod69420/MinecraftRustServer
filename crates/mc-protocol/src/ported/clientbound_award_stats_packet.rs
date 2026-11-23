use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stat {
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub name: &'static str,
}

pub const CLIENTBOUND_AWARD_STATS: PacketType = PacketType {
    name: "minecraft:award_stats",
};

pub trait ClientGamePacketListener {
    fn handle_award_stats(&mut self, packet: &ClientboundAwardStatsPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAwardStatsPacket {
    stats: HashMap<Stat, i32>,
}

impl ClientboundAwardStatsPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundAwardStatsPacket#<init>
    pub fn new(stats: HashMap<Stat, i32>) -> Self {
        Self { stats }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAwardStatsPacket#stats
    pub fn stats(&self) -> &HashMap<Stat, i32> {
        &self.stats
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAwardStatsPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_AWARD_STATS
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundAwardStatsPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_award_stats(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        called: bool,
        total_entries: usize,
        sample_value: Option<i32>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_award_stats(&mut self, packet: &ClientboundAwardStatsPacket) {
            self.called = true;
            self.total_entries = packet.stats().len();
            self.sample_value = packet.stats().get(&Stat { id: 1 }).copied();
        }
    }

    #[test]
    fn test_new_and_accessors() {
        let mut stats = HashMap::new();
        stats.insert(Stat { id: 1 }, 10);
        stats.insert(Stat { id: 2 }, 20);
        let pkt = ClientboundAwardStatsPacket::new(stats.clone());
        assert_eq!(pkt.stats(), &stats);
    }

    #[test]
    fn test_empty_stats() {
        let pkt = ClientboundAwardStatsPacket::new(HashMap::new());
        assert!(pkt.stats().is_empty());
        assert_eq!(pkt.get_type(), CLIENTBOUND_AWARD_STATS);
    }

    #[test]
    fn test_negative_and_extreme_values() {
        let mut stats = HashMap::new();
        stats.insert(Stat { id: -1 }, i32::MIN);
        stats.insert(Stat { id: i32::MAX }, i32::MAX);
        stats.insert(Stat { id: 0 }, -42);
        let pkt = ClientboundAwardStatsPacket::new(stats);
        assert_eq!(pkt.stats().get(&Stat { id: -1 }), Some(&i32::MIN));
        assert_eq!(pkt.stats().get(&Stat { id: i32::MAX }), Some(&i32::MAX));
        assert_eq!(pkt.stats().get(&Stat { id: 0 }), Some(&-42));
    }

    #[test]
    fn test_handle_dispatch() {
        let mut stats = HashMap::new();
        stats.insert(Stat { id: 1 }, 99);
        let pkt = ClientboundAwardStatsPacket::new(stats);
        let mut listener = MockListener {
            called: false,
            total_entries: 0,
            sample_value: None,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.total_entries, 1);
        assert_eq!(listener.sample_value, Some(99));
    }

    #[test]
    fn test_overflow_wrapping_value_semantics() {
        let mut stats = HashMap::new();
        let value = i32::MAX.wrapping_add(1);
        stats.insert(Stat { id: 7 }, value);
        let pkt = ClientboundAwardStatsPacket::new(stats);
        assert_eq!(pkt.stats().get(&Stat { id: 7 }), Some(&i32::MIN));
    }
}
