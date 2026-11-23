use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

// JAVA: net.minecraft.network.protocol.game.GamePacketTypes#CLIENTBOUND_REMOVE_ENTITIES
pub const CLIENTBOUND_REMOVE_ENTITIES: PacketType =
    PacketType("minecraft:clientbound_remove_entities");

#[derive(Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readByte
    fn read_byte(&mut self) -> io::Result<u8> {
        if self.read_pos >= self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read past end of buffer",
            ));
        }
        let b = self.data[self.read_pos];
        self.read_pos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & !0x7f) == 0 {
                self.data.push(value as u8);
                return self;
            }
            self.data.push(((value & 0x7f) | 0x80) as u8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result: i32 = 0;
        let mut shift: u32 = 0;
        loop {
            let b = self.read_byte()?;
            result |= ((b & 0x7f) as i32).wrapping_shl(shift.wrapping_mul(7));
            shift += 1;
            if shift > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if (b & 0x80) == 0 {
                break;
            }
        }
        Ok(result)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readIntIdList
    pub fn read_int_id_list(&mut self) -> io::Result<Vec<i32>> {
        let len = self.read_var_int()?;
        if len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative int id list length",
            ));
        }
        let mut list = Vec::with_capacity(len as usize);
        for _ in 0..len {
            list.push(self.read_var_int()?);
        }
        Ok(list)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeIntIdList
    pub fn write_int_id_list(&mut self, list: &[i32]) -> &mut Self {
        self.write_var_int(list.len() as i32);
        for &v in list {
            self.write_var_int(v);
        }
        self
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writerIndex
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readableBytes
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.read_pos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRemoveEntitiesPacket {
    entity_ids: Vec<i32>,
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleRemoveEntities
    fn handle_remove_entities(&mut self, packet: &ClientboundRemoveEntitiesPacket);
}

impl ClientboundRemoveEntitiesPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket#<init>
    pub fn new(entity_ids: Vec<i32>) -> Self {
        Self { entity_ids }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket#<init>
    pub fn from_slice(entity_ids: &[i32]) -> Self {
        Self {
            entity_ids: entity_ids.to_vec(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let entity_ids = buf.read_int_id_list()?;
        Ok(Self { entity_ids })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int_id_list(&self.entity_ids);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket#type
    pub fn packet_type(&self) -> PacketType {
        CLIENTBOUND_REMOVE_ENTITIES
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_remove_entities(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket#getEntityIds
    pub fn get_entity_ids(&self) -> &[i32] {
        &self.entity_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_multiple_ids() {
        let pkt = ClientboundRemoveEntitiesPacket::new(vec![1, 2, 3, -5, i32::MAX]);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        let decoded = ClientboundRemoveEntitiesPacket::read(&mut r).unwrap();
        assert_eq!(decoded.get_entity_ids(), &[1, 2, 3, -5, i32::MAX]);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn roundtrip_empty() {
        let pkt = ClientboundRemoveEntitiesPacket::from_slice(&[]);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        let decoded = ClientboundRemoveEntitiesPacket::read(&mut r).unwrap();
        assert!(decoded.get_entity_ids().is_empty());
    }

    #[test]
    fn from_slice_constructor() {
        let pkt = ClientboundRemoveEntitiesPacket::from_slice(&[10, 20, 30]);
        assert_eq!(pkt.get_entity_ids(), &[10, 20, 30]);
    }

    #[test]
    fn negative_and_extreme_ids() {
        let pkt = ClientboundRemoveEntitiesPacket::new(vec![i32::MIN, -1, 0]);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        let decoded = ClientboundRemoveEntitiesPacket::read(&mut r).unwrap();
        assert_eq!(decoded.get_entity_ids(), &[i32::MIN, -1, 0]);
    }

    #[test]
    fn handle_dispatches() {
        struct Listener {
            ids: Vec<i32>,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_remove_entities(&mut self, packet: &ClientboundRemoveEntitiesPacket) {
                self.ids = packet.get_entity_ids().to_vec();
            }
        }
        let pkt = ClientboundRemoveEntitiesPacket::new(vec![7, 8, 9]);
        let mut l = Listener { ids: vec![] };
        pkt.handle(&mut l);
        assert_eq!(l.ids, vec![7, 8, 9]);
    }
}
