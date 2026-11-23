use bytes::{Buf, BufMut, BytesMut};
use std::io::Cursor;

pub trait TickRateManager {
    fn tickrate(&self) -> f32;
    fn is_frozen(&self) -> bool;
}

pub trait ClientGamePacketListener {
    fn handle_ticking_state(&mut self, packet: &ClientboundTickingStatePacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundTickingStatePacket {
    tick_rate: f32,
    is_frozen: bool,
}

impl ClientboundTickingStatePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#<init>
    pub fn new(tick_rate: f32, is_frozen: bool) -> Self {
        Self {
            tick_rate,
            is_frozen,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#<init>
    pub fn read(buf: &mut Cursor<&[u8]>) -> Option<Self> {
        if buf.remaining() < 5 {
            return None;
        }
        let tick_rate = buf.get_f32();
        let is_frozen = buf.get_u8() != 0;
        Some(Self {
            tick_rate,
            is_frozen,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#from
    pub fn from_manager(manager: &impl TickRateManager) -> Self {
        Self::new(manager.tickrate(), manager.is_frozen())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#write
    pub fn write(&self, buf: &mut BytesMut) {
        buf.put_f32(self.tick_rate);
        buf.put_u8(if self.is_frozen { 1 } else { 0 });
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#tickRate
    pub fn tick_rate(&self) -> f32 {
        self.tick_rate
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#isFrozen
    pub fn is_frozen(&self) -> bool {
        self.is_frozen
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:ticking_state"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStatePacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_ticking_state(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct MockManager {
        tick_rate: f32,
        frozen: bool,
    }

    impl TickRateManager for MockManager {
        fn tickrate(&self) -> f32 {
            self.tick_rate
        }

        fn is_frozen(&self) -> bool {
            self.frozen
        }
    }

    #[test]
    fn round_trip_normal() {
        let pkt = ClientboundTickingStatePacket::new(20.0, false);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ClientboundTickingStatePacket::read(&mut cur).unwrap();
        assert_eq!(pkt.tick_rate().to_bits(), decoded.tick_rate().to_bits());
        assert_eq!(pkt.is_frozen(), decoded.is_frozen());
    }

    #[test]
    fn round_trip_frozen_negative_rate() {
        let pkt = ClientboundTickingStatePacket::new(-1.5, true);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ClientboundTickingStatePacket::read(&mut cur).unwrap();
        assert_eq!(pkt.tick_rate().to_bits(), decoded.tick_rate().to_bits());
        assert!(decoded.is_frozen());
    }

    #[test]
    fn round_trip_nan_and_infinity_bits() {
        let nan = f32::from_bits(0x7FC0_0001);
        let pkt_nan = ClientboundTickingStatePacket::new(nan, false);
        let mut buf = BytesMut::new();
        pkt_nan.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded_nan = ClientboundTickingStatePacket::read(&mut cur).unwrap();
        assert_eq!(nan.to_bits(), decoded_nan.tick_rate().to_bits());

        let pkt_inf = ClientboundTickingStatePacket::new(f32::NEG_INFINITY, true);
        let mut buf2 = BytesMut::new();
        pkt_inf.write(&mut buf2);
        let mut cur2 = Cursor::new(&buf2[..]);
        let decoded_inf = ClientboundTickingStatePacket::read(&mut cur2).unwrap();
        assert_eq!(f32::NEG_INFINITY.to_bits(), decoded_inf.tick_rate().to_bits());
        assert!(decoded_inf.is_frozen());
    }

    #[test]
    fn from_manager_and_handle() {
        struct Listener {
            rate: f32,
            frozen: bool,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_ticking_state(&mut self, packet: &ClientboundTickingStatePacket) {
                self.rate = packet.tick_rate();
                self.frozen = packet.is_frozen();
            }
        }

        let mgr = MockManager {
            tick_rate: 10.0,
            frozen: true,
        };
        let pkt = ClientboundTickingStatePacket::from_manager(&mgr);
        assert_eq!(pkt.tick_rate().to_bits(), 10.0f32.to_bits());
        assert!(pkt.is_frozen());
        assert_eq!(pkt.packet_type(), "minecraft:ticking_state");

        let mut listener = Listener {
            rate: 0.0,
            frozen: false,
        };
        pkt.handle(&mut listener);
        assert_eq!(listener.rate.to_bits(), 10.0f32.to_bits());
        assert!(listener.frozen);
    }
}
