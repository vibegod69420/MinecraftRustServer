use bytes::{Buf, BufMut};

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int<B: Buf>(buf: &mut B) -> i32 {
    let mut val: u32 = 0;
    let mut pos: u32 = 0;
    loop {
        let b = buf.get_u8();
        val |= ((b & 0x7F) as u32).wrapping_shl(pos.wrapping_mul(7));
        pos = pos.wrapping_add(1);
        if pos > 5 {
            panic!("VarInt too big");
        }
        if (b & 0x80) == 0 {
            break;
        }
    }
    val as i32
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int<B: BufMut>(buf: &mut B, mut value: i32) {
    loop {
        if (value as u32 & !0x7F_u32) == 0 {
            buf.put_u8(value as u8);
            return;
        }
        buf.put_u8(((value as u8) & 0x7F) | 0x80);
        value = ((value as u32) >> 7) as i32;
    }
}

pub trait ClientGamePacketListener {
    fn handle_take_item_entity(&mut self, packet: &ClientboundTakeItemEntityPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundTakeItemEntityPacket {
    item_id: i32,
    player_id: i32,
    amount: i32,
}

impl ClientboundTakeItemEntityPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#<init>(int, int, int)
    pub fn new(item_id: i32, player_id: i32, amount: i32) -> Self {
        Self {
            item_id,
            player_id,
            amount,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#<init>(FriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Self {
        let item_id = read_var_int(buf);
        let player_id = read_var_int(buf);
        let amount = read_var_int(buf);
        Self {
            item_id,
            player_id,
            amount,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#write
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        write_var_int(buf, self.item_id);
        write_var_int(buf, self.player_id);
        write_var_int(buf, self.amount);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:take_item_entity"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_take_item_entity(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#getItemId
    pub fn get_item_id(&self) -> i32 {
        self.item_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#getPlayerId
    pub fn get_player_id(&self) -> i32 {
        self.player_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket#getAmount
    pub fn get_amount(&self) -> i32 {
        self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn write_read_roundtrip() {
        let packet = ClientboundTakeItemEntityPacket::new(10, 20, 64);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundTakeItemEntityPacket::read(&mut reader);
        assert_eq!(packet, decoded);
    }

    #[test]
    fn negative_and_min_values_roundtrip() {
        let packet = ClientboundTakeItemEntityPacket::new(-1, -100, i32::MIN);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundTakeItemEntityPacket::read(&mut reader);
        assert_eq!(decoded.get_item_id(), -1);
        assert_eq!(decoded.get_player_id(), -100);
        assert_eq!(decoded.get_amount(), i32::MIN);
    }

    #[test]
    fn getters() {
        let packet = ClientboundTakeItemEntityPacket::new(1, 2, 3);
        assert_eq!(packet.get_item_id(), 1);
        assert_eq!(packet.get_player_id(), 2);
        assert_eq!(packet.get_amount(), 3);
    }

    #[test]
    fn handle_calls_listener() {
        struct L {
            item: i32,
            player: i32,
            amount: i32,
        }
        impl ClientGamePacketListener for L {
            fn handle_take_item_entity(&mut self, packet: &ClientboundTakeItemEntityPacket) {
                self.item = packet.get_item_id();
                self.player = packet.get_player_id();
                self.amount = packet.get_amount();
            }
        }
        let packet = ClientboundTakeItemEntityPacket::new(7, 8, 9);
        let mut l = L {
            item: 0,
            player: 0,
            amount: 0,
        };
        packet.handle(&mut l);
        assert_eq!((l.item, l.player, l.amount), (7, 8, 9));
    }
}
