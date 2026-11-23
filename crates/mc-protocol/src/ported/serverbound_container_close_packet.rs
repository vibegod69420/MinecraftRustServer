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

    // JAVA: net.minecraft.network.FriendlyByteBuf#readContainerId
    pub fn read_container_id(&mut self) -> io::Result<i32> {
        self.read_var_int()
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeContainerId
    pub fn write_container_id(&mut self, value: i32) -> io::Result<()> {
        self.write_var_int(value)
    }
}

pub trait ServerGamePacketListener {
    fn handle_container_close(&mut self, packet: &ServerboundContainerClosePacket);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundContainerClosePacket {
    container_id: i32,
}

impl ServerboundContainerClosePacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClosePacket#<init>(int)
    pub fn new(container_id: i32) -> Self {
        Self { container_id }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClosePacket#<init>(FriendlyByteBuf)
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let container_id = buf.read_container_id()?;
        Ok(Self { container_id })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClosePacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) -> io::Result<()> {
        buf.write_container_id(self.container_id)
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClosePacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:container_close"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClosePacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_container_close(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundContainerClosePacket#getContainerId
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_container_id_zero() {
        let pkt = ServerboundContainerClosePacket::new(0);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundContainerClosePacket::read(&mut read_buf).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_container_id(), 0);
    }

    #[test]
    fn roundtrip_container_id_negative() {
        let pkt = ServerboundContainerClosePacket::new(-1);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf).unwrap();
        let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
        let decoded = ServerboundContainerClosePacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.get_container_id(), -1);
    }

    #[test]
    fn roundtrip_container_id_extremes() {
        for id in [i32::MIN, i32::MAX, 127, 128, 255, -128] {
            let pkt = ServerboundContainerClosePacket::new(id);
            let mut buf = FriendlyByteBuf::default();
            pkt.write(&mut buf).unwrap();
            let mut read_buf = FriendlyByteBuf::new(buf.into_inner());
            let decoded = ServerboundContainerClosePacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.get_container_id(), id);
        }
    }

    #[test]
    fn packet_type_name() {
        let pkt = ServerboundContainerClosePacket::new(1);
        assert_eq!(pkt.packet_type(), "minecraft:container_close");
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener {
            seen: Option<i32>,
        }
        impl ServerGamePacketListener for MockListener {
            fn handle_container_close(&mut self, packet: &ServerboundContainerClosePacket) {
                self.seen = Some(packet.get_container_id());
            }
        }
        let pkt = ServerboundContainerClosePacket::new(42);
        let mut listener = MockListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some(42));
    }
}
