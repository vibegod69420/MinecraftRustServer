use bytes::{Buf, BufMut, BytesMut};
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item_id: i32,
    pub count: i32,
}

impl ItemStack {
    pub fn empty() -> Self {
        Self {
            item_id: 0,
            count: 0,
        }
    }

    fn write(&self, buf: &mut BytesMut) {
        write_varint(buf, self.count);
        if self.count > 0 {
            write_varint(buf, self.item_id);
        }
    }

    fn read(buf: &mut Cursor<&[u8]>) -> Option<Self> {
        let count = read_varint(buf)?;
        if count <= 0 {
            return Some(Self {
                item_id: 0,
                count: 0,
            });
        }
        let item_id = read_varint(buf)?;
        Some(Self { item_id, count })
    }
}

pub trait ServerGamePacketListener {
    fn handle_set_creative_mode_slot(&mut self, packet: &ServerboundSetCreativeModeSlotPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCreativeModeSlotPacket {
    slot_num: i16,
    item_stack: ItemStack,
}

impl ServerboundSetCreativeModeSlotPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket#<init>
    pub fn new(slot_num: i16, item_stack: ItemStack) -> Self {
        Self {
            slot_num,
            item_stack,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket#<init>
    pub fn from_i32(slot_num: i32, item_stack: ItemStack) -> Self {
        Self {
            slot_num: slot_num as i16,
            item_stack,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket#slotNum
    pub fn slot_num(&self) -> i16 {
        self.slot_num
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket#itemStack
    pub fn item_stack(&self) -> &ItemStack {
        &self.item_stack
    }

    pub fn write(&self, buf: &mut BytesMut) {
        buf.put_i16(self.slot_num);
        self.item_stack.write(buf);
    }

    pub fn read(buf: &mut Cursor<&[u8]>) -> Option<Self> {
        if buf.remaining() < 2 {
            return None;
        }
        let slot_num = buf.get_i16();
        let item_stack = ItemStack::read(buf)?;
        Some(Self {
            slot_num,
            item_stack,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:set_creative_mode_slot"
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket#handle
    pub fn handle<L: ServerGamePacketListener>(&self, listener: &mut L) {
        listener.handle_set_creative_mode_slot(self);
    }
}

fn write_varint(buf: &mut BytesMut, value: i32) {
    let mut val = value as u32;
    loop {
        let mut temp = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            temp |= 0x80;
        }
        buf.put_u8(temp);
        if val == 0 {
            break;
        }
    }
}

fn read_varint(buf: &mut Cursor<&[u8]>) -> Option<i32> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return None;
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as u32) << shift;
        if (b & 0x80) == 0 {
            return Some(result as i32);
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip_non_empty() {
        let pkt = ServerboundSetCreativeModeSlotPacket::new(
            36,
            ItemStack {
                item_id: 1,
                count: 64,
            },
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ServerboundSetCreativeModeSlotPacket::read(&mut cur).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn from_i32_truncates_like_java_short() {
        let pkt = ServerboundSetCreativeModeSlotPacket::from_i32(0x0001_0002, ItemStack::empty());
        assert_eq!(pkt.slot_num(), 2i16);

        let pkt_neg = ServerboundSetCreativeModeSlotPacket::from_i32(-1, ItemStack::empty());
        assert_eq!(pkt_neg.slot_num(), -1i16);

        let pkt_overflow = ServerboundSetCreativeModeSlotPacket::from_i32(i32::MAX, ItemStack::empty());
        assert_eq!(pkt_overflow.slot_num(), -1i16);
    }

    #[test]
    fn round_trip_negative_slot_and_empty_stack() {
        let pkt = ServerboundSetCreativeModeSlotPacket::new(-1, ItemStack::empty());
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut cur = Cursor::new(&buf[..]);
        let decoded = ServerboundSetCreativeModeSlotPacket::read(&mut cur).unwrap();
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.item_stack().count, 0);
    }

    #[test]
    fn handle_calls_listener() {
        struct Listener {
            slot: i16,
        }
        impl ServerGamePacketListener for Listener {
            fn handle_set_creative_mode_slot(
                &mut self,
                packet: &ServerboundSetCreativeModeSlotPacket,
            ) {
                self.slot = packet.slot_num();
            }
        }

        let pkt = ServerboundSetCreativeModeSlotPacket::new(5, ItemStack::empty());
        assert_eq!(pkt.packet_type(), "minecraft:set_creative_mode_slot");
        let mut listener = Listener { slot: 0 };
        pkt.handle(&mut listener);
        assert_eq!(listener.slot, 5);
    }
}
