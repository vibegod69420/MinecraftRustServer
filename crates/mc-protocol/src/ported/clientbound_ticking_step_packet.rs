use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    pub data: BytesMut,
    pub reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: BytesMut) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    fn read_byte(&mut self) -> i8 {
        let b = self.data[self.reader_index] as i8;
        self.reader_index += 1;
        b
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_byte();
            i |= ((b0 as i32) & 127) << (j.wrapping_mul(7) & 31);
            j = j.wrapping_add(1);
            if j > 5 {
                panic!("VarInt too big");
            }
            if ((b0 as u8) & 0x80) == 0 {
                break;
            }
        }
        i
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let mut b = (value & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                b |= 0x80;
            }
            self.data.put_u8(b);
            if value == 0 {
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub const CLIENTBOUND_TICKING_STEP_TYPE: PacketType = PacketType {
    id: "minecraft:ticking_step",
};

pub trait ClientGamePacketListener {
    fn handle_ticking_step(&mut self, packet: &ClientboundTickingStepPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TickRateManager {
    frozen_ticks_to_run: i32,
}

impl TickRateManager {
    pub fn new(frozen_ticks_to_run: i32) -> Self {
        Self {
            frozen_ticks_to_run,
        }
    }

    pub fn frozen_ticks_to_run(&self) -> i32 {
        self.frozen_ticks_to_run
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundTickingStepPacket {
    tick_steps: i32,
}

impl ClientboundTickingStepPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStepPacket#<init>
    pub fn new(tick_steps: i32) -> Self {
        Self { tick_steps }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStepPacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        Self {
            tick_steps: buf.read_var_int(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStepPacket#from
    pub fn from_tick_rate_manager(manager: &TickRateManager) -> Self {
        Self::new(manager.frozen_ticks_to_run())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStepPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.tick_steps);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStepPacket#type
    pub fn get_type(&self) -> PacketType {
        CLIENTBOUND_TICKING_STEP_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStepPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_ticking_step(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTickingStepPacket#tickSteps
    pub fn tick_steps(&self) -> i32 {
        self.tick_steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_tick_steps_edge_values() {
        for v in [0, 1, -1, 20, 127, 128, i32::MAX, i32::MIN] {
            let pkt = ClientboundTickingStepPacket::new(v);
            let mut buf = FriendlyByteBuf::default();
            pkt.write(&mut buf);
            buf.reader_index = 0;
            let decoded = ClientboundTickingStepPacket::from_buf(&mut buf);
            assert_eq!(decoded.tick_steps(), v, "tick_steps mismatch for {}", v);
        }
    }

    #[test]
    fn from_tick_rate_manager_uses_frozen_ticks_to_run() {
        let mgr = TickRateManager::new(42);
        let pkt = ClientboundTickingStepPacket::from_tick_rate_manager(&mgr);
        assert_eq!(pkt.tick_steps(), 42);

        let neg_mgr = TickRateManager::new(-7);
        let neg_pkt = ClientboundTickingStepPacket::from_tick_rate_manager(&neg_mgr);
        assert_eq!(neg_pkt.tick_steps(), -7);
    }

    #[test]
    fn varint_encoding_negative_one_is_five_bytes() {
        let pkt = ClientboundTickingStepPacket::new(-1);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        assert_eq!(buf.data.as_ref(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);
    }

    struct MockListener {
        called: bool,
        tick_steps: i32,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_ticking_step(&mut self, packet: &ClientboundTickingStepPacket) {
            self.called = true;
            self.tick_steps = packet.tick_steps();
        }
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ClientboundTickingStepPacket::new(123);
        let mut listener = MockListener {
            called: false,
            tick_steps: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.tick_steps, 123);
        assert_eq!(pkt.get_type(), CLIENTBOUND_TICKING_STEP_TYPE);
    }
}
