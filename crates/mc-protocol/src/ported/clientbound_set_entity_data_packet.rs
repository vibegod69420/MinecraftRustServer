use bytes::{BufMut, BytesMut};
use std::io;

#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf {
    data: BytesMut,
    rpos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: BytesMut::from(bytes),
            rpos: 0,
        }
    }

    pub fn into_inner(self) -> BytesMut {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.rpos)
    }

    fn read_byte_raw(&mut self) -> io::Result<u8> {
        if self.remaining() < 1 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_byte"));
        }
        let b = self.data[self.rpos];
        self.rpos += 1;
        Ok(b)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;
        loop {
            let b = self.read_byte_raw()? as i32;
            value |= (b & 127) << (position.wrapping_mul(7));
            if (b & 128) == 0 {
                break;
            }
            position += 1;
            if position >= 5 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }
        Ok(value)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) -> &mut Self {
        loop {
            if (value & !0x7F) == 0 {
                self.write_byte(value as i8);
                return self;
            }
            self.write_byte(((value & 0x7F) | 0x80) as i8);
            value = ((value as u32) >> 7) as i32;
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readUnsignedByte
    pub fn read_unsigned_byte(&mut self) -> io::Result<i32> {
        let b = self.read_byte_raw()?;
        Ok(b as i32)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeByte
    pub fn write_byte(&mut self, value: i8) -> &mut Self {
        self.data.put_u8(value as u8);
        self
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        if self.remaining() < len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_bytes"));
        }
        let out = self.data[self.rpos..self.rpos + len].to_vec();
        self.rpos += len;
        Ok(out)
    }
}

/// Self-contained stub for SynchedEntityData.DataValue.
/// Real MC wire format is serializer-registry dependent; this stub uses a
/// length-prefixed payload so the packet pack/unpack logic can be tested in isolation.
pub mod synched_entity_data {
    use super::{io, RegistryFriendlyByteBuf};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DataValue {
        pub id: u8,
        pub serializer_id: i32,
        pub value: Vec<u8>,
    }

    impl DataValue {
        // JAVA: net.minecraft.network.syncher.SynchedEntityData.DataValue#write
        pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
            buf.write_byte(self.id as i8);
            buf.write_var_int(self.serializer_id);
            buf.write_var_int(self.value.len() as i32);
            buf.write_bytes(&self.value);
        }

        // JAVA: net.minecraft.network.syncher.SynchedEntityData.DataValue#read
        pub fn read(buf: &mut RegistryFriendlyByteBuf, id: i32) -> io::Result<Self> {
            let serializer_id = buf.read_var_int()?;
            let len = buf.read_var_int()?;
            if len < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "negative DataValue length",
                ));
            }
            let value = buf.read_bytes(len as usize)?;
            Ok(Self {
                id: id as u8,
                serializer_id,
                value,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType;

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_SET_ENTITY_DATA: PacketType = PacketType;
}

pub trait ClientGamePacketListener {
    fn handle_set_entity_data(&mut self, packet: &ClientboundSetEntityDataPacket);
}

pub const EOF_MARKER: i32 = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetEntityDataPacket {
    id: i32,
    packed_items: Vec<synched_entity_data::DataValue>,
}

impl ClientboundSetEntityDataPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#<init>
    pub fn new(id: i32, packed_items: Vec<synched_entity_data::DataValue>) -> Self {
        Self { id, packed_items }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#<init>
    pub fn from_buf(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let id = buf.read_var_int()?;
        let packed_items = Self::unpack(buf)?;
        Ok(Self { id, packed_items })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#pack
    fn pack(items: &[synched_entity_data::DataValue], buf: &mut RegistryFriendlyByteBuf) {
        for item in items {
            item.write(buf);
        }
        buf.write_byte(EOF_MARKER as i8);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#unpack
    fn unpack(
        buf: &mut RegistryFriendlyByteBuf,
    ) -> io::Result<Vec<synched_entity_data::DataValue>> {
        let mut items = Vec::new();
        loop {
            let b = buf.read_unsigned_byte()?;
            if b == EOF_MARKER {
                break;
            }
            items.push(synched_entity_data::DataValue::read(buf, b)?);
        }
        Ok(items)
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.id);
        Self::pack(&self.packed_items, buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#type
    pub fn packet_type(&self) -> PacketType {
        GamePacketTypes::CLIENTBOUND_SET_ENTITY_DATA
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_entity_data(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#id
    pub fn id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket#packedItems
    pub fn packed_items(&self) -> &[synched_entity_data::DataValue] {
        &self.packed_items
    }
}

#[cfg(test)]
mod tests {
    use super::synched_entity_data::DataValue;
    use super::*;

    #[test]
    fn roundtrip_empty_items() {
        let pkt = ClientboundSetEntityDataPacket::new(42, vec![]);
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_inner();
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(&bytes);
        let decoded = ClientboundSetEntityDataPacket::from_buf(&mut read_buf).unwrap();
        assert_eq!(decoded, pkt);
        assert_eq!(decoded.id(), 42);
        assert!(decoded.packed_items().is_empty());
    }

    #[test]
    fn roundtrip_multiple_items() {
        let items = vec![
            DataValue {
                id: 0,
                serializer_id: 1,
                value: vec![0x7F],
            },
            DataValue {
                id: 5,
                serializer_id: 2,
                value: vec![1, 2, 3, 4],
            },
            DataValue {
                id: 254,
                serializer_id: 0,
                value: vec![],
            },
        ];
        let pkt = ClientboundSetEntityDataPacket::new(123, items.clone());
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_inner();
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(&bytes);
        let decoded = ClientboundSetEntityDataPacket::from_buf(&mut read_buf).unwrap();
        assert_eq!(decoded.id(), 123);
        assert_eq!(decoded.packed_items(), &items[..]);
    }

    #[test]
    fn negative_and_large_entity_id_varint() {
        let pkt = ClientboundSetEntityDataPacket::new(-1, vec![]);
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf);
        let bytes = buf.into_inner();
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(&bytes);
        let decoded = ClientboundSetEntityDataPacket::from_buf(&mut read_buf).unwrap();
        assert_eq!(decoded.id(), -1);

        let pkt2 = ClientboundSetEntityDataPacket::new(i32::MAX, vec![]);
        let mut buf2 = RegistryFriendlyByteBuf::new();
        pkt2.write(&mut buf2);
        let bytes2 = buf2.into_inner();
        let mut read_buf2 = RegistryFriendlyByteBuf::from_bytes(&bytes2);
        let decoded2 = ClientboundSetEntityDataPacket::from_buf(&mut read_buf2).unwrap();
        assert_eq!(decoded2.id(), i32::MAX);
    }

    #[test]
    fn eof_marker_terminates_unpack() {
        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_var_int(7);
        buf.write_byte(EOF_MARKER as i8);
        let bytes = buf.into_inner();
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(&bytes);
        let decoded = ClientboundSetEntityDataPacket::from_buf(&mut read_buf).unwrap();
        assert_eq!(decoded.id(), 7);
        assert!(decoded.packed_items().is_empty());
    }

    #[test]
    fn handle_dispatches_to_listener() {
        struct MockListener {
            called: bool,
            seen_id: i32,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_set_entity_data(&mut self, packet: &ClientboundSetEntityDataPacket) {
                self.called = true;
                self.seen_id = packet.id();
            }
        }
        let pkt = ClientboundSetEntityDataPacket::new(99, vec![]);
        let mut listener = MockListener {
            called: false,
            seen_id: 0,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.seen_id, 99);
    }
}
