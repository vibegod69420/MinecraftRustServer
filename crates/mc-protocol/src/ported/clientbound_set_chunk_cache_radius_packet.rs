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

    fn read_u8(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn write_u8(&mut self, b: u8) -> io::Result<()> {
        self.data.push(b);
        Ok(())
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.read_u8()? as i32;
            result |= (byte & 0x7F) << shift;
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> io::Result<()> {
        loop {
            if (value & !0x7F) == 0 {
                self.write_u8(value as u8)?;
                return Ok(());
            }
            self.write_u8(((value & 0x7F) | 0x80) as u8)?;
            value = ((value as u32) >> 7) as i32;
        }
    }
}

pub trait ClientGamePacketListener {
    fn handle_set_chunk_cache_radius(&mut self, packet: &ClientboundSetChunkCacheRadiusPacket);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheRadiusPacket {
    radius: i32,
}

impl ClientboundSetChunkCacheRadiusPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket#<init>(int)
    pub fn new(radius: i32) -> Self {
        Self { radius }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let radius = buf.read_var_int()?;
        Ok(Self { radius })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_var_int(self.radius)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:set_chunk_cache_radius"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_chunk_cache_radius(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket#getRadius
    pub fn get_radius(&self) -> i32 {
        self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_radius() {
        let pkt = ClientboundSetChunkCacheRadiusPacket::new(10);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundSetChunkCacheRadiusPacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_radius(), 10);
    }

    #[test]
    fn roundtrip_negative_radius() {
        let pkt = ClientboundSetChunkCacheRadiusPacket::new(-1);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundSetChunkCacheRadiusPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_radius(), -1);
    }

    #[test]
    fn roundtrip_extreme_radii() {
        for radius in [i32::MIN, i32::MAX, 0, 127, 128] {
            let pkt = ClientboundSetChunkCacheRadiusPacket::new(radius);
            let mut buf = FriendlyByteBuf::default();
            pkt.write(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ClientboundSetChunkCacheRadiusPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_radius(), radius);
        }
    }

    #[test]
    fn packet_type_name() {
        let pkt = ClientboundSetChunkCacheRadiusPacket::new(2);
        assert_eq!(pkt.packet_type(), "minecraft:set_chunk_cache_radius");
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener {
            seen: Option<i32>,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_set_chunk_cache_radius(
                &mut self,
                packet: &ClientboundSetChunkCacheRadiusPacket,
            ) {
                self.seen = Some(packet.get_radius());
            }
        }
        let pkt = ClientboundSetChunkCacheRadiusPacket::new(8);
        let mut listener = MockListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some(8));
    }
}
