use bytes::{Buf, BufMut, BytesMut};
use std::io::Cursor;

pub trait CombatTracker {
    fn get_combat_duration(&self) -> i32;
}

pub trait ClientGamePacketListener {
    fn handle_player_combat_end(&mut self, packet: &ClientboundPlayerCombatEndPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerCombatEndPacket {
    duration: i32,
}

impl ClientboundPlayerCombatEndPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket#<init>
    pub fn new(duration: i32) -> Self {
        Self { duration }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket#<init>
    pub fn from_combat_tracker(tracker: &impl CombatTracker) -> Self {
        Self::new(tracker.get_combat_duration())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket#<init>
    pub fn read(buf: &mut Cursor<&[u8]>) -> Option<Self> {
        let duration = read_varint(buf)?;
        Some(Self { duration })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket#write
    pub fn write(&self, buf: &mut BytesMut) {
        write_varint(buf, self.duration);
    }

    pub fn duration(&self) -> i32 {
        self.duration
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:player_combat_end"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_player_combat_end(self);
    }
}

fn write_varint(buf: &mut BytesMut, value: i32) {
    let mut val = value as u32;
    loop {
        let mut temp = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            temp |= 0x80;
        }
        buf.put_u8(temp);
        if val == 0 {
            break;
        }
    }
}

fn read_varint(buf: &mut Cursor<&[u8]>) -> Option<i32> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return None;
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as u32) << shift;
        if (b & 0x80) == 0 {
            return Some(result as i32);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct MockTracker {
        duration: i32,
    }

    impl CombatTracker for MockTracker {
        fn get_combat_duration(&self) -> i32 {
            self.duration
        }
    }

    #[test]
    fn round_trip_normal() {
        let pkt = ClientboundPlayerCombatEndPacket::new(100);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ClientboundPlayerCombatEndPacket::read(&mut cur).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.duration(), 100);
    }

    #[test]
    fn round_trip_negative_duration() {
        let pkt = ClientboundPlayerCombatEndPacket::new(-1);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ClientboundPlayerCombatEndPacket::read(&mut cur).unwrap();
        assert_eq!(decoded.duration(), -1);
    }

    #[test]
    fn round_trip_min_max_zero() {
        for value in [i32::MIN, i32::MAX, 0, -123456] {
            let pkt = ClientboundPlayerCombatEndPacket::new(value);
            let mut buf = BytesMut::new();
            pkt.write(&mut buf);
            let mut cur = Cursor::new(&buf[..]);
            let decoded = ClientboundPlayerCombatEndPacket::read(&mut cur).unwrap();
            assert_eq!(decoded.duration(), value);
        }
    }

    #[test]
    fn from_combat_tracker_and_handle() {
        struct Listener {
            duration: i32,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_player_combat_end(&mut self, packet: &ClientboundPlayerCombatEndPacket) {
                self.duration = packet.duration();
            }
        }

        let tracker = MockTracker { duration: 42 };
        let pkt = ClientboundPlayerCombatEndPacket::from_combat_tracker(&tracker);
        assert_eq!(pkt.duration(), 42);
        assert_eq!(pkt.packet_type(), "minecraft:player_combat_end");

        let mut listener = Listener { duration: 0 };
        pkt.handle(&mut listener);
        assert_eq!(listener.duration, 42);
    }
}
