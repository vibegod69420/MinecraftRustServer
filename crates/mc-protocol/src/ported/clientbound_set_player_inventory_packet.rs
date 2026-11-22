use bytes::{BufMut, BytesMut};
use std::io;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    buf: BytesMut,
    read_pos: usize,
}

impl FriendlyByteBuf {
    // JAVA: net.minecraft.network.FriendlyByteBuf#<init>
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self {
            buf: BytesMut::from(&data[..]),
            read_pos: 0,
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf.to_vec()
    }

    fn read_byte(&mut self) -> io::Result<i8> {
        if self.read_pos >= self.buf.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let b = self.buf[self.read_pos] as i8;
        self.read_pos += 1;
        Ok(b)
    }

    fn write_byte(&mut self, value: i8) {
        self.buf.put_i8(value);
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> io::Result<i32> {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_byte()?;
            i |= ((b0 as i32) & 127) << j.wrapping_mul(7);
            j = j.wrapping_add(1);
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
            if ((b0 as i32) & 128) != 128 {
                break;
            }
        }
        Ok(i)
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            if (value & -128) != 0 {
                self.write_byte(((value & 127) | 128) as i8);
                value = ((value as u32) >> 7) as i32;
            } else {
                self.write_byte(value as i8);
                break;
            }
        }
    }
}

pub type RegistryFriendlyByteBuf = FriendlyByteBuf;

/// Minimal self-contained ItemStack representation.
/// Supports the common 1.21.x optional codec case with an empty DataComponentPatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item_id: i32,
    pub count: i32,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#EMPTY
    pub fn empty() -> Self {
        Self {
            item_id: 0,
            count: 0,
        }
    }

    // JAVA: net.minecraft.world.item.ItemStack#isEmpty
    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }

    // JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC (decode)
    pub fn read_optional(buf: &mut FriendlyByteBuf) -> io::Result<Self> {
        let count = buf.read_var_int()?;
        if count <= 0 {
            return Ok(Self::empty());
        }
        let item_id = buf.read_var_int()?;
        let components_set = buf.read_var_int()?;
        let components_removed = buf.read_var_int()?;
        if components_set != 0 || components_removed != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-empty DataComponentPatch not supported in minimal self-contained port",
            ));
        }
        Ok(Self { item_id, count })
    }

    // JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC (encode)
    pub fn write_optional(buf: &mut FriendlyByteBuf, stack: &ItemStack) {
        buf.write_var_int(stack.count);
        if stack.count > 0 {
            buf.write_var_int(stack.item_id);
            // DataComponentPatch.EMPTY: 0 components set, 0 removed
            buf.write_var_int(0);
            buf.write_var_int(0);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType<T>(PhantomData<T>);

pub trait ClientGamePacketListener {
    fn handle_set_player_inventory(&mut self, packet: &ClientboundSetPlayerInventoryPacket);
}

pub const CLIENTBOUND_SET_PLAYER_INVENTORY: PacketType<ClientboundSetPlayerInventoryPacket> =
    PacketType(PhantomData);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetPlayerInventoryPacket {
    pub slot: i32,
    pub contents: ItemStack,
}

impl ClientboundSetPlayerInventoryPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerInventoryPacket#<init>
    pub fn new(slot: i32, contents: ItemStack) -> Self {
        Self { slot, contents }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerInventoryPacket#<init> (codec-derived)
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> io::Result<Self> {
        let slot = buf.read_var_int()?;
        let contents = ItemStack::read_optional(buf)?;
        Ok(Self::new(slot, contents))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerInventoryPacket#write (codec-derived)
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_var_int(self.slot);
        ItemStack::write_optional(buf, &self.contents);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerInventoryPacket#type
    pub fn get_type(&self) -> PacketType<Self> {
        CLIENTBOUND_SET_PLAYER_INVENTORY
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSetPlayerInventoryPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_set_player_inventory(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty_item() {
        let packet = ClientboundSetPlayerInventoryPacket::new(5, ItemStack::empty());
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetPlayerInventoryPacket::read(&mut read_buf).unwrap();
        assert_eq!(packet, decoded);
        assert!(decoded.contents.is_empty());
    }

    #[test]
    fn roundtrip_present_item() {
        let item = ItemStack {
            item_id: 42,
            count: 64,
        };
        let packet = ClientboundSetPlayerInventoryPacket::new(9, item.clone());
        let mut buf = FriendlyByteBuf::new();
        packet.write(&mut buf);
        let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
        let decoded = ClientboundSetPlayerInventoryPacket::read(&mut read_buf).unwrap();
        assert_eq!(decoded.slot, 9);
        assert_eq!(decoded.contents, item);
    }

    #[test]
    fn roundtrip_negative_and_extreme_slots() {
        for slot in [0, -1, i32::MIN, i32::MAX] {
            let packet = ClientboundSetPlayerInventoryPacket::new(slot, ItemStack::empty());
            let mut buf = FriendlyByteBuf::new();
            packet.write(&mut buf);
            let mut read_buf = FriendlyByteBuf::from_vec(buf.into_vec());
            let decoded = ClientboundSetPlayerInventoryPacket::read(&mut read_buf).unwrap();
            assert_eq!(decoded.slot, slot);
        }
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener {
            seen: Option<(i32, i32)>,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_set_player_inventory(
                &mut self,
                packet: &ClientboundSetPlayerInventoryPacket,
            ) {
                self.seen = Some((packet.slot, packet.contents.count));
            }
        }
        let packet = ClientboundSetPlayerInventoryPacket::new(
            3,
            ItemStack {
                item_id: 1,
                count: 16,
            },
        );
        let mut listener = MockListener { seen: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen, Some((3, 16)));
    }
}
