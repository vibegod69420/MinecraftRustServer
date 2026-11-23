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

/// Minimal packet type marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

/// Minimal listener stub for client game packet handling.
pub trait ClientGamePacketListener {
    fn handle_set_chunk_cache_center(&mut self, packet: &ClientboundSetChunkCacheCenterPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheCenterPacket {
    x: i32,
    z: i32,
}

impl ClientboundSetChunkCacheCenterPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket#<init>
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        Self {
            x: buf.read_var_int(),
            z: buf.read_var_int(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.x);
        buf.write_var_int(self.z);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:set_chunk_cache_center")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_chunk_cache_center(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        seen: Option<(i32, i32)>,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_set_chunk_cache_center(
            &mut self,
            packet: &ClientboundSetChunkCacheCenterPacket,
        ) {
            self.seen = Some((packet.get_x(), packet.get_z()));
        }
    }

    #[test]
    fn round_trip_basic() {
        let pkt = ClientboundSetChunkCacheCenterPacket::new(10, -20);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
        let decoded = ClientboundSetChunkCacheCenterPacket::read(&mut r);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_x(), 10);
        assert_eq!(decoded.get_z(), -20);
    }

    #[test]
    fn round_trip_extremes() {
        let cases = [
            (i32::MIN, i32::MAX),
            (i32::MAX, i32::MIN),
            (0, 0),
            (-1, -1),
        ];
        for (x, z) in cases {
            let pkt = ClientboundSetChunkCacheCenterPacket::new(x, z);
            let mut buf = FriendlyByteBuf::new();
            pkt.write(&mut buf);
            let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
            let decoded = ClientboundSetChunkCacheCenterPacket::read(&mut r);
            assert_eq!(decoded.get_x(), x);
            assert_eq!(decoded.get_z(), z);
        }
    }

    #[test]
    fn handle_dispatches() {
        let pkt = ClientboundSetChunkCacheCenterPacket::new(3, 4);
        let mut listener = TestListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some((3, 4)));
        assert_eq!(
            pkt.packet_type(),
            PacketType("minecraft:set_chunk_cache_center")
        );
    }

    #[test]
    fn varint_negative_round_trip() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        buf.write_var_int(i32::MIN);
        let mut r = FriendlyByteBuf::from_bytes(buf.bytes());
        assert_eq!(r.read_var_int(), -1);
        assert_eq!(r.read_var_int(), i32::MIN);
        assert_eq!(r.remaining(), 0);
    }
}
