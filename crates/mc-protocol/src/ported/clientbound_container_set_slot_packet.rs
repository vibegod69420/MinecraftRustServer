use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    ClientboundContainerSetSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
    pub item_id: i32,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#<init>
    pub fn new(item_id: i32, count: i32) -> Self {
        Self { item_id, count }
    }

    // JAVA: net.minecraft.world.item.ItemStack#copy
    pub fn copy(&self) -> Self {
        self.clone()
    }

    // JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC decode
    pub fn read_optional(buf: &mut impl Buf) -> Self {
        let count = read_var_int(buf);
        if count <= 0 {
            Self { count: 0, item_id: 0 }
        } else {
            let item_id = read_var_int(buf);
            Self { count, item_id }
        }
    }

    // JAVA: net.minecraft.world.item.ItemStack#OPTIONAL_STREAM_CODEC encode
    pub fn write_optional(&self, buf: &mut impl BufMut) {
        if self.count <= 0 {
            write_var_int(buf, 0);
        } else {
            write_var_int(buf, self.count);
            write_var_int(buf, self.item_id);
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8() as u32;
        result |= (b & 0x7F) << shift;
        if (b & 0x80) == 0 {
            return result as i32;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return result as i32;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readContainerId
fn read_container_id(buf: &mut impl Buf) -> i32 {
    buf.get_u8() as i32
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeContainerId
fn write_container_id(buf: &mut impl BufMut, value: i32) {
    buf.put_u8(value as u8);
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleContainerSetSlot
    fn handle_container_set_slot(&mut self, packet: &ClientboundContainerSetSlotPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundContainerSetSlotPacket {
    container_id: i32,
    state_id: i32,
    slot: i32,
    item_stack: ItemStack,
}

impl ClientboundContainerSetSlotPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#<init>(int,int,int,ItemStack)
    pub fn new(container_id: i32, state_id: i32, slot: i32, item_stack: ItemStack) -> Self {
        Self {
            container_id,
            state_id,
            slot,
            item_stack: item_stack.copy(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#<init>(RegistryFriendlyByteBuf)
    pub fn read(buf: &mut impl Buf) -> Self {
        let container_id = read_container_id(buf);
        let state_id = read_var_int(buf);
        let slot = buf.get_i16() as i32;
        let item_stack = ItemStack::read_optional(buf);
        Self {
            container_id,
            state_id,
            slot,
            item_stack,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_container_id(buf, self.container_id);
        write_var_int(buf, self.state_id);
        buf.put_i16(self.slot as i16);
        self.item_stack.write_optional(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ClientboundContainerSetSlot
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_container_set_slot(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#getContainerId
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#getSlot
    pub fn get_slot(&self) -> i32 {
        self.slot
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#getItem
    pub fn get_item(&self) -> ItemStack {
        self.item_stack.copy()
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket#getStateId
    pub fn get_state_id(&self) -> i32 {
        self.state_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        called: bool,
        slot: i32,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_container_set_slot(&mut self, packet: &ClientboundContainerSetSlotPacket) {
            self.called = true;
            self.slot = packet.get_slot();
        }
    }

    #[test]
    fn round_trip_non_empty_item() {
        let packet = ClientboundContainerSetSlotPacket::new(1, 42, 13, ItemStack::new(7, 64));
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundContainerSetSlotPacket::read(&mut reader);
        assert_eq!(decoded, packet);
        assert_eq!(decoded.get_item(), ItemStack::new(7, 64));
    }

    #[test]
    fn round_trip_empty_item_and_negative_slot() {
        let packet = ClientboundContainerSetSlotPacket::new(255, -1, -1, ItemStack::new(0, 0));
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundContainerSetSlotPacket::read(&mut reader);
        assert_eq!(decoded.get_container_id(), 255);
        assert_eq!(decoded.get_state_id(), -1);
        assert_eq!(decoded.get_slot(), -1);
        assert_eq!(decoded.get_item().count, 0);
    }

    #[test]
    fn constructor_copies_item_stack() {
        let original = ItemStack::new(3, 10);
        let packet = ClientboundContainerSetSlotPacket::new(0, 1, 2, original.copy());
        let mut modified = original.copy();
        modified.count = 99;
        assert_eq!(packet.get_item().count, 10);
        assert_eq!(modified.count, 99);
    }

    #[test]
    fn handle_calls_listener() {
        let packet = ClientboundContainerSetSlotPacket::new(2, 5, 9, ItemStack::new(1, 1));
        let mut listener = TestListener { called: false, slot: -1 };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.slot, 9);
        assert_eq!(packet.packet_type(), PacketType::ClientboundContainerSetSlot);
    }
}
