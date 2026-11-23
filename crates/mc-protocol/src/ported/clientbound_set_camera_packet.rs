use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut i: i32 = 0;
    let mut j: u32 = 0;
    loop {
        if buf.is_empty() {
            return Err(DecodeError {
                message: "VarInt too short".to_string(),
            });
        }
        let b0 = buf[0];
        *buf = &buf[1..];
        i |= ((b0 & 0x7F) as i32).wrapping_shl(j.wrapping_mul(7));
        j = j.wrapping_add(1);
        if j > 5 {
            return Err(DecodeError {
                message: "VarInt too big".to_string(),
            });
        }
        if (b0 & 0x80) == 0 {
            break;
        }
    }
    Ok(i)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        if (value & !0x7F) == 0 {
            buf.push(value as u8);
            return;
        }
        buf.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

pub trait EntityId {
    fn get_id(&self) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetCameraPacket {
    pub camera_id: i32,
}

impl ClientboundSetCameraPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCameraPacket#<init>
    pub fn new(camera_id: i32) -> Self {
        Self { camera_id }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCameraPacket#<init>
    pub fn from_entity<E: EntityId>(entity: &E) -> Self {
        Self {
            camera_id: entity.get_id(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCameraPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        let camera_id = read_var_int(buf)?;
        Ok(Self { camera_id })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCameraPacket#write
    pub fn write(&self, buf: &mut Vec<u8>) {
        write_var_int(buf, self.camera_id);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCameraPacket#type
    pub fn packet_type() -> &'static str {
        "minecraft:set_camera"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetCameraPacket#getEntity
    pub fn get_entity<E, F>(&self, lookup: F) -> Option<E>
    where
        F: FnOnce(i32) -> Option<E>,
    {
        lookup(self.camera_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEntity(i32);

    impl EntityId for MockEntity {
        fn get_id(&self) -> i32 {
            self.0
        }
    }

    #[test]
    fn round_trip() {
        let pkt = ClientboundSetCameraPacket::new(42);
        let mut buf = Vec::new();
        pkt.write(&mut buf);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundSetCameraPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, pkt);
        assert!(slice.is_empty());
    }

    #[test]
    fn from_entity_uses_id() {
        let pkt = ClientboundSetCameraPacket::from_entity(&MockEntity(123));
        assert_eq!(pkt.camera_id, 123);
    }

    #[test]
    fn get_entity_lookup() {
        let pkt = ClientboundSetCameraPacket::new(7);
        let found = pkt.get_entity(|id| if id == 7 { Some("entity") } else { None });
        assert_eq!(found, Some("entity"));
        let missing = pkt.get_entity(|_| None::<()>);
        assert_eq!(missing, None);
    }

    #[test]
    fn negative_and_extreme_ids() {
        for id in [i32::MIN, -1, 0, i32::MAX] {
            let pkt = ClientboundSetCameraPacket::new(id);
            let mut buf = Vec::new();
            pkt.write(&mut buf);
            let mut slice: &[u8] = &buf;
            let decoded = ClientboundSetCameraPacket::read(&mut slice).unwrap();
            assert_eq!(decoded.camera_id, id);
        }
    }

    #[test]
    fn short_buffer_errors() {
        let mut slice: &[u8] = &[];
        assert!(ClientboundSetCameraPacket::read(&mut slice).is_err());
    }
}
