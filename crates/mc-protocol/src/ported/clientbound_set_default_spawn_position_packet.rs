use bytes::{Buf, BufMut, BytesMut};

/// Minimal self-contained buffer for packet encoding/decoding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    data: BytesMut,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: BytesMut::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: BytesMut::from(bytes),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data.to_vec()
    }

    pub fn remaining(&self) -> usize {
        self.data.remaining()
    }

    pub fn read_varint(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.data.get_u8() as i32;
            result |= (b & 0x7f).wrapping_shl(shift & 31);
            if (b & 0x80) == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                return result;
            }
        }
    }

    pub fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut b = ((value as u32) & 0x7f) as u8;
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

    pub fn read_utf_with_max(&mut self, max_len: usize) -> String {
        let len = self.read_varint() as usize;
        let max_bytes = max_len.saturating_mul(3).saturating_add(3);
        assert!(len <= max_bytes, "String too long (bytes)");
        let bytes = self.data.split_to(len);
        let s = String::from_utf8(bytes.to_vec()).expect("invalid utf8");
        assert!(s.chars().count() <= max_len, "String too long (chars)");
        s
    }

    pub fn write_utf_with_max(&mut self, value: &str, max_len: usize) {
        let char_count = value.chars().count();
        assert!(char_count <= max_len, "String too long");
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.put_slice(bytes);
    }

    pub fn read_f32(&mut self) -> f32 {
        self.data.get_f32()
    }

    pub fn write_f32(&mut self, value: f32) {
        self.data.put_f32(value);
    }
}

/// Minimal self-contained RespawnData representation.
/// In Java this is `net.minecraft.world.level.storage.LevelData.RespawnData`.
#[derive(Debug, Clone, PartialEq)]
pub struct RespawnData {
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: f32,
    pub pitch: f32,
}

impl RespawnData {
    fn read(buf: &mut FriendlyByteBuf) -> Self {
        let dimension = buf.read_utf_with_max(32_767);
        let x = buf.read_varint();
        let y = buf.read_varint();
        let z = buf.read_varint();
        let yaw = buf.read_f32();
        let pitch = buf.read_f32();
        Self {
            dimension,
            x,
            y,
            z,
            yaw,
            pitch,
        }
    }

    fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf_with_max(&self.dimension, 32_767);
        buf.write_varint(self.x);
        buf.write_varint(self.y);
        buf.write_varint(self.z);
        buf.write_f32(self.yaw);
        buf.write_f32(self.pitch);
    }
}

pub const PACKET_TYPE_CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION: &str =
    "minecraft:clientbound_set_default_spawn_position";

pub trait ClientGamePacketListener {
    fn handle_set_spawn(&mut self, packet: &ClientboundSetDefaultSpawnPositionPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetDefaultSpawnPositionPacket {
    respawn_data: RespawnData,
}

impl ClientboundSetDefaultSpawnPositionPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDefaultSpawnPositionPacket#<init>
    pub fn new(respawn_data: RespawnData) -> Self {
        Self { respawn_data }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDefaultSpawnPositionPacket#respawnData
    pub fn respawn_data(&self) -> &RespawnData {
        &self.respawn_data
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDefaultSpawnPositionPacket#type
    pub fn type_name(&self) -> &'static str {
        PACKET_TYPE_CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetDefaultSpawnPositionPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_set_spawn(self);
    }

    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let respawn_data = RespawnData::read(buf);
        Self::new(respawn_data)
    }

    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        self.respawn_data.write(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        seen: Option<ClientboundSetDefaultSpawnPositionPacket>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_set_spawn(&mut self, packet: &ClientboundSetDefaultSpawnPositionPacket) {
            self.seen = Some(packet.clone());
        }
    }

    fn sample_data() -> RespawnData {
        RespawnData {
            dimension: "minecraft:overworld".to_owned(),
            x: 100,
            y: 64,
            z: -200,
            yaw: 90.0,
            pitch: -45.0,
        }
    }

    #[test]
    fn roundtrip_spawn_position() {
        let packet = ClientboundSetDefaultSpawnPositionPacket::new(sample_data());
        let mut buf = FriendlyByteBuf::new();
        packet.encode(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_bytes(&buf.into_bytes());
        let decoded = ClientboundSetDefaultSpawnPositionPacket::decode(&mut read_buf);
        assert_eq!(decoded, packet);
        assert_eq!(read_buf.remaining(), 0);
    }

    #[test]
    fn negative_and_extreme_coordinates() {
        let data = RespawnData {
            dimension: "minecraft:the_nether".to_owned(),
            x: i32::MIN,
            y: -1,
            z: i32::MAX,
            yaw: 0.0,
            pitch: 0.0,
        };
        let packet = ClientboundSetDefaultSpawnPositionPacket::new(data);
        let mut buf = FriendlyByteBuf::new();
        packet.encode(&mut buf);
        let decoded = ClientboundSetDefaultSpawnPositionPacket::decode(
            &mut FriendlyByteBuf::from_bytes(&buf.into_bytes()),
        );
        assert_eq!(decoded.respawn_data().x, i32::MIN);
        assert_eq!(decoded.respawn_data().y, -1);
        assert_eq!(decoded.respawn_data().z, i32::MAX);
    }

    #[test]
    fn float_bits_preserved() {
        let data = RespawnData {
            dimension: "minecraft:overworld".to_owned(),
            x: 0,
            y: 0,
            z: 0,
            yaw: f32::from_bits(0x7FC0_0001),
            pitch: f32::NEG_INFINITY,
        };
        let packet = ClientboundSetDefaultSpawnPositionPacket::new(data);
        let mut buf = FriendlyByteBuf::new();
        packet.encode(&mut buf);
        let decoded = ClientboundSetDefaultSpawnPositionPacket::decode(
            &mut FriendlyByteBuf::from_bytes(&buf.into_bytes()),
        );
        assert_eq!(decoded.respawn_data().yaw.to_bits(), 0x7FC0_0001);
        assert_eq!(
            decoded.respawn_data().pitch.to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
    }

    #[test]
    fn type_and_handle() {
        let packet = ClientboundSetDefaultSpawnPositionPacket::new(sample_data());
        assert_eq!(
            packet.type_name(),
            "minecraft:clientbound_set_default_spawn_position"
        );
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some(packet));
    }
}
