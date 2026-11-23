/// Minimal self-contained byte buffer mirroring Minecraft's FriendlyByteBuf.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    rpos: usize,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            rpos: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec(),
            rpos: 0,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.rpos)
    }

    fn read_byte(&mut self) -> u8 {
        assert!(self.rpos < self.data.len(), "FriendlyByteBuf: out of bounds");
        let b = self.data[self.rpos];
        self.rpos = self.rpos.wrapping_add(1);
        b
    }

    fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_byte();
            i |= ((b0 & 127) as i32).wrapping_shl((j.wrapping_mul(7)) as u32);
            j = j.wrapping_add(1);
            if j > 5 {
                panic!("VarInt too big");
            }
            if (b0 & 128) == 0 {
                break;
            }
        }
        i
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & -128i32) == 0 {
                self.write_byte(value as u8);
                return self;
            }
            self.write_byte(((value & 127) as u8) | 128);
            value = ((value as u32) >> 7) as i32;
        }
    }
}

/// Minimal stub for net.minecraft.world.level.border.WorldBorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldBorder {
    warning_time: i32,
}

impl WorldBorder {
    pub fn new(warning_time: i32) -> Self {
        Self { warning_time }
    }

    // JAVA: net.minecraft.world.level.border.WorldBorder#getWarningTime
    pub fn get_warning_time(&self) -> i32 {
        self.warning_time
    }
}

/// Minimal packet type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

/// Minimal listener stub for client game packet handling.
pub trait ClientGamePacketListener {
    fn handle_set_border_warning_delay(&mut self, packet: &ClientboundSetBorderWarningDelayPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetBorderWarningDelayPacket {
    warning_delay: i32,
}

impl ClientboundSetBorderWarningDelayPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket#<init>
    pub fn from_world_border(world_border: &WorldBorder) -> Self {
        Self {
            warning_delay: world_border.get_warning_time(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        Self {
            warning_delay: buf.read_var_int(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.warning_delay);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:set_border_warning_delay")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_border_warning_delay(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket#getWarningDelay
    pub fn get_warning_delay(&self) -> i32 {
        self.warning_delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<i32>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_set_border_warning_delay(
            &mut self,
            packet: &ClientboundSetBorderWarningDelayPacket,
        ) {
            self.seen = Some(packet.get_warning_delay());
        }
    }

    #[test]
    fn round_trip_positive() {
        let pkt = ClientboundSetBorderWarningDelayPacket::from_world_border(&WorldBorder::new(42));
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = ClientboundSetBorderWarningDelayPacket::read(&mut r);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_warning_delay(), 42);
    }

    #[test]
    fn round_trip_negative_and_zero() {
        for v in [0, -1, -128, -2_147_483_648i32] {
            let pkt = ClientboundSetBorderWarningDelayPacket { warning_delay: v };
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf);
            let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
            let decoded = ClientboundSetBorderWarningDelayPacket::read(&mut r);
            assert_eq!(decoded.get_warning_delay(), v);
        }
    }

    #[test]
    fn round_trip_extreme_positive() {
        for v in [i32::MAX, 127, 128, 255, 256] {
            let pkt = ClientboundSetBorderWarningDelayPacket { warning_delay: v };
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf);
            let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
            let decoded = ClientboundSetBorderWarningDelayPacket::read(&mut r);
            assert_eq!(decoded.get_warning_delay(), v);
        }
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundSetBorderWarningDelayPacket { warning_delay: 7 };
        let mut listener = TestListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some(7));
        assert_eq!(
            pkt.packet_type(),
            PacketType("minecraft:set_border_warning_delay")
        );
    }

    #[test]
    fn varint_encoding_edge_values() {
        let mut buf = FriendlyByteBuf::new();
        for v in [0i32, 1, -1, i32::MIN, i32::MAX] {
            buf.write_var_int(v);
        }
        let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
        for expected in [0i32, 1, -1, i32::MIN, i32::MAX] {
            assert_eq!(r.read_var_int(), expected);
        }
        assert_eq!(r.remaining(), 0);
    }
}
