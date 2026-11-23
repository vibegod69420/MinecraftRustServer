use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug)]
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

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    pub fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        for shift in (0..32).step_by(7) {
            let b = self.read_byte()?;
            result |= ((b & 0x7F) as i32) << shift;
            if (b & 0x80) == 0 {
                return Ok(result);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VarInt too big",
        ))
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7F) == 0 {
                self.write_byte(val as u8);
                return;
            }
            self.write_byte(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
    }
}

#[derive(Debug)]
pub struct RegistryFriendlyByteBuf {
    inner: FriendlyByteBuf,
}

impl RegistryFriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            inner: FriendlyByteBuf::new(data),
        }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.inner.into_inner()
    }

    pub fn remaining(&self) -> usize {
        self.inner.remaining()
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        self.inner.read_byte()
    }

    pub fn write_byte(&mut self, b: u8) {
        self.inner.write_byte(b);
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        self.inner.read_var_int()
    }

    pub fn write_var_int(&mut self, value: i32) {
        self.inner.write_var_int(value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonPlayerSpawnInfo {
    pub stub_id: i32,
}

impl CommonPlayerSpawnInfo {
    // JAVA: net.minecraft.network.protocol.game.CommonPlayerSpawnInfo#<init>
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let stub_id = buf.read_var_int()?;
        Ok(Self { stub_id })
    }

    // JAVA: net.minecraft.network.protocol.game.CommonPlayerSpawnInfo#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.stub_id);
    }
}

pub trait ClientGamePacketListener {
    fn handle_respawn(&mut self, packet: &ClientboundRespawnPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRespawnPacket {
    pub common_player_spawn_info: CommonPlayerSpawnInfo,
    pub data_to_keep: i8,
}

impl ClientboundRespawnPacket {
    pub const KEEP_ATTRIBUTE_MODIFIERS: i8 = 1;
    pub const KEEP_ENTITY_DATA: i8 = 2;
    pub const KEEP_ALL_DATA: i8 = 3;

    // JAVA: net.minecraft.network.protocol.game.ClientboundRespawnPacket#<init>
    pub fn new(common_player_spawn_info: CommonPlayerSpawnInfo, data_to_keep: i8) -> Self {
        Self {
            common_player_spawn_info,
            data_to_keep,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRespawnPacket#<init>
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let common_player_spawn_info = CommonPlayerSpawnInfo::read(buf)?;
        let data_to_keep = buf.read_byte()? as i8;
        Ok(Self::new(common_player_spawn_info, data_to_keep))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRespawnPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        self.common_player_spawn_info.write(buf);
        buf.write_byte(self.data_to_keep as u8);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRespawnPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:respawn")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRespawnPacket#handle
    pub fn handle<H: ClientGamePacketListener>(&self, handler: &mut H) {
        handler.handle_respawn(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRespawnPacket#shouldKeep
    pub fn should_keep(&self, flag: i8) -> bool {
        (self.data_to_keep & flag) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let pkt = ClientboundRespawnPacket::new(
            CommonPlayerSpawnInfo { stub_id: 42 },
            ClientboundRespawnPacket::KEEP_ALL_DATA,
        );
        let mut buf = RegistryFriendlyByteBuf::new(Vec::new());
        pkt.write(&mut buf);
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundRespawnPacket::read(&mut reader).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn should_keep_flags() {
        let all = ClientboundRespawnPacket::new(
            CommonPlayerSpawnInfo { stub_id: 0 },
            ClientboundRespawnPacket::KEEP_ALL_DATA,
        );
        assert!(all.should_keep(ClientboundRespawnPacket::KEEP_ATTRIBUTE_MODIFIERS));
        assert!(all.should_keep(ClientboundRespawnPacket::KEEP_ENTITY_DATA));
        assert!(all.should_keep(ClientboundRespawnPacket::KEEP_ALL_DATA));

        let attrs_only = ClientboundRespawnPacket::new(
            CommonPlayerSpawnInfo { stub_id: 0 },
            ClientboundRespawnPacket::KEEP_ATTRIBUTE_MODIFIERS,
        );
        assert!(attrs_only.should_keep(ClientboundRespawnPacket::KEEP_ATTRIBUTE_MODIFIERS));
        assert!(!attrs_only.should_keep(ClientboundRespawnPacket::KEEP_ENTITY_DATA));
        assert!(!attrs_only.should_keep(0));
    }

    #[test]
    fn negative_data_to_keep_roundtrip() {
        let pkt = ClientboundRespawnPacket::new(CommonPlayerSpawnInfo { stub_id: -7 }, -1i8);
        let mut buf = RegistryFriendlyByteBuf::new(Vec::new());
        pkt.write(&mut buf);
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundRespawnPacket::read(&mut reader).unwrap();
        assert_eq!(decoded.data_to_keep, -1);
        assert_eq!(decoded.common_player_spawn_info.stub_id, -7);
        assert!(decoded.should_keep(ClientboundRespawnPacket::KEEP_ENTITY_DATA));
    }

    #[test]
    fn handle_and_type() {
        let pkt = ClientboundRespawnPacket::new(CommonPlayerSpawnInfo { stub_id: 1 }, 0);
        assert_eq!(pkt.packet_type(), PacketType("minecraft:respawn"));

        struct H {
            kept: i8,
        }
        impl ClientGamePacketListener for H {
            fn handle_respawn(&mut self, p: &ClientboundRespawnPacket) {
                self.kept = p.data_to_keep;
            }
        }

        let mut handler = H { kept: -1 };
        pkt.handle(&mut handler);
        assert_eq!(handler.kept, 0);
    }
}
