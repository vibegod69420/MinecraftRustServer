use std::io;

#[derive(Clone, Debug, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readDouble
    pub fn read_double(&mut self) -> io::Result<f64> {
        if self.pos + 8 > self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(f64::from_be_bytes(bytes))
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeDouble
    pub fn write_double(&mut self, value: f64) -> io::Result<()> {
        self.data.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }
}

pub trait WorldBorderLike {
    // JAVA: net.minecraft.world.level.border.WorldBorder#getLerpTarget
    fn get_lerp_target(&self) -> f64;
}

pub trait ClientGamePacketListener {
    fn handle_set_border_size(&mut self, packet: &ClientboundSetBorderSizePacket);
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetBorderSizePacket {
    size: f64,
}

impl ClientboundSetBorderSizePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket#<init>(WorldBorder)
    pub fn from_world_border(border: &impl WorldBorderLike) -> Self {
        Self {
            size: border.get_lerp_target(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let size = buf.read_double()?;
        Ok(Self { size })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_double(self.size)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:set_border_size"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_border_size(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket#getSize
    pub fn get_size(&self) -> f64 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBorder(f64);
    impl WorldBorderLike for MockBorder {
        fn get_lerp_target(&self) -> f64 {
            self.0
        }
    }

    #[test]
    fn from_world_border_uses_lerp_target() {
        let pkt = ClientboundSetBorderSizePacket::from_world_border(&MockBorder(123.5));
        assert_eq!(pkt.get_size().to_bits(), 123.5f64.to_bits());
    }

    #[test]
    fn roundtrip_normal_size() {
        let pkt = ClientboundSetBorderSizePacket::from_world_border(&MockBorder(59_999_968.0));
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundSetBorderSizePacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_size().to_bits(), pkt.get_size().to_bits());
    }

    #[test]
    fn roundtrip_negative_zero_and_small() {
        for v in [0.0f64, -0.0f64, -1.0f64, f64::MIN_POSITIVE] {
            let pkt = ClientboundSetBorderSizePacket::from_world_border(&MockBorder(v));
            let mut buf = FriendlyByteBuf::default();
            pkt.write(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ClientboundSetBorderSizePacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_size().to_bits(), v.to_bits());
        }
    }

    #[test]
    fn roundtrip_nan_and_infinity_bits() {
        for bits in [
            f64::NAN.to_bits(),
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            u64::MAX,
        ] {
            let v = f64::from_bits(bits);
            let pkt = ClientboundSetBorderSizePacket::from_world_border(&MockBorder(v));
            let mut buf = FriendlyByteBuf::default();
            pkt.write(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ClientboundSetBorderSizePacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_size().to_bits(), bits);
        }
    }

    #[test]
    fn packet_type_name() {
        let pkt = ClientboundSetBorderSizePacket::from_world_border(&MockBorder(1.0));
        assert_eq!(pkt.packet_type(), "minecraft:set_border_size");
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener {
            size: f64,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_set_border_size(&mut self, packet: &ClientboundSetBorderSizePacket) {
                self.size = packet.get_size();
            }
        }
        let pkt = ClientboundSetBorderSizePacket::from_world_border(&MockBorder(42.0));
        let mut listener = MockListener { size: 0.0 };
        pkt.handle(&mut listener);
        assert_eq!(listener.size.to_bits(), 42.0f64.to_bits());
    }
}
