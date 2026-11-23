use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

// JAVA: net.minecraft.network.protocol.game.GamePacketTypes#SERVERBOUND_ENTITY_TAG_QUERY
pub const SERVERBOUND_ENTITY_TAG_QUERY: PacketType =
    PacketType("minecraft:serverbound_entity_tag_query");

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
pub struct ServerboundEntityTagQueryPacket {
    transaction_id: i32,
    entity_id: i32,
}

pub trait ServerGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ServerGamePacketListener#handleEntityTagQuery
    fn handle_entity_tag_query(&mut self, packet: &ServerboundEntityTagQueryPacket);
}

impl ServerboundEntityTagQueryPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket#<init>
    pub fn new(transaction_id: i32, entity_id: i32) -> Self {
        Self {
            transaction_id,
            entity_id,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket#<init>
    pub fn read(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let transaction_id = buf.read_var_int()?;
        let entity_id = buf.read_var_int()?;
        Ok(Self {
            transaction_id,
            entity_id,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        buf.write_var_int(self.entity_id);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket#type
    pub fn packet_type(&self) -> PacketType {
        SERVERBOUND_ENTITY_TAG_QUERY
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_entity_tag_query(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket#getTransactionId
    pub fn get_transaction_id(&self) -> i32 {
        self.transaction_id
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket#getEntityId
    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let pkt = ServerboundEntityTagQueryPacket::new(123, -456);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        let decoded = ServerboundEntityTagQueryPacket::read(&mut r).unwrap();
        assert_eq!(decoded.get_transaction_id(), 123);
        assert_eq!(decoded.get_entity_id(), -456);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn negative_and_extreme_ids() {
        let pkt = ServerboundEntityTagQueryPacket::new(i32::MIN, i32::MAX);
        let mut buf = FriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut r = FriendlyByteBuf {
            data: buf.into_inner(),
            read_pos: 0,
        };
        let decoded = ServerboundEntityTagQueryPacket::read(&mut r).unwrap();
        assert_eq!(decoded.get_transaction_id(), i32::MIN);
        assert_eq!(decoded.get_entity_id(), i32::MAX);
    }

    #[test]
    fn varint_negative_one_encoding() {
        let mut buf = FriendlyByteBuf::new();
        buf.write_var_int(-1);
        let bytes = buf.into_inner();
        assert_eq!(bytes, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        let mut r = FriendlyByteBuf {
            data: bytes,
            read_pos: 0,
        };
        assert_eq!(r.read_var_int().unwrap(), -1);
    }

    #[test]
    fn handle_dispatches() {
        struct Listener {
            called: bool,
            tid: i32,
            eid: i32,
        }
        impl ServerGamePacketListener for Listener {
            fn handle_entity_tag_query(&mut self, packet: &ServerboundEntityTagQueryPacket) {
                self.called = true;
                self.tid = packet.get_transaction_id();
                self.eid = packet.get_entity_id();
            }
        }
        let pkt = ServerboundEntityTagQueryPacket::new(7, 9);
        let mut l = Listener {
            called: false,
            tid: 0,
            eid: 0,
        };
        pkt.handle(&mut l);
        assert!(l.called);
        assert_eq!(l.tid, 7);
        assert_eq!(l.eid, 9);
    }
}
