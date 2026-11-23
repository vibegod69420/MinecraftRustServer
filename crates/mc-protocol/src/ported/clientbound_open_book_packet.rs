use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundOpenBookPacket {
    hand: InteractionHand,
}

pub trait ClientGamePacketListener {
    fn handle_open_book(&mut self, packet: &ClientboundOpenBookPacket);
}

impl ClientboundOpenBookPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenBookPacket#<init>
    pub fn new(hand: InteractionHand) -> Self {
        Self { hand }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenBookPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Self {
        let hand = read_enum_hand(buf);
        Self { hand }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenBookPacket#write
    pub fn write(&self, buf: &mut impl BufMut) {
        write_enum_hand(buf, self.hand);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenBookPacket#type
    pub fn packet_type(&self) -> &'static str {
        "minecraft:open_book"
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenBookPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_open_book(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundOpenBookPacket#getHand
    pub fn get_hand(&self) -> InteractionHand {
        self.hand
    }
}

fn read_enum_hand(buf: &mut &[u8]) -> InteractionHand {
    match read_varint(buf) {
        0 => InteractionHand::MainHand,
        1 => InteractionHand::OffHand,
        other => panic!("invalid InteractionHand ordinal: {other}"),
    }
}

fn write_enum_hand(buf: &mut impl BufMut, hand: InteractionHand) {
    let ordinal = match hand {
        InteractionHand::MainHand => 0i32,
        InteractionHand::OffHand => 1i32,
    };
    write_varint(buf, ordinal);
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut &[u8]) -> i32 {
    let mut result: i32 = 0;
    let mut j: u32 = 0;
    loop {
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl((j.wrapping_mul(7)) & 31);
        j = j.wrapping_add(1);
        if j > 5 {
            panic!("VarInt too big");
        }
        if (b & 0x80) == 0 {
            return result;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut impl BufMut, mut value: i32) {
    loop {
        if (value as u32 & 0xFFFF_FF80) != 0 {
            buf.put_u8(((value as u8) & 0x7F) | 0x80);
            value = ((value as u32) >> 7) as i32;
        } else {
            buf.put_u8(value as u8);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct MockListener {
        called: bool,
        hand: Option<InteractionHand>,
    }

    impl ClientGamePacketListener for MockListener {
        fn handle_open_book(&mut self, packet: &ClientboundOpenBookPacket) {
            self.called = true;
            self.hand = Some(packet.get_hand());
        }
    }

    #[test]
    fn roundtrip_main_hand() {
        let pkt = ClientboundOpenBookPacket::new(InteractionHand::MainHand);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(&buf[..], &[0x00]);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundOpenBookPacket::read(&mut slice);
        assert_eq!(decoded.get_hand(), InteractionHand::MainHand);
        assert!(slice.is_empty());
    }

    #[test]
    fn roundtrip_off_hand() {
        let pkt = ClientboundOpenBookPacket::new(InteractionHand::OffHand);
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        assert_eq!(&buf[..], &[0x01]);
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundOpenBookPacket::read(&mut slice);
        assert_eq!(decoded.get_hand(), InteractionHand::OffHand);
    }

    #[test]
    fn handle_and_type() {
        let pkt = ClientboundOpenBookPacket::new(InteractionHand::MainHand);
        assert_eq!(pkt.packet_type(), "minecraft:open_book");
        let mut listener = MockListener {
            called: false,
            hand: None,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.hand, Some(InteractionHand::MainHand));
    }

    #[test]
    #[should_panic(expected = "invalid InteractionHand ordinal")]
    fn invalid_hand_ordinal_panics() {
        let data = [2u8];
        let mut slice: &[u8] = &data;
        let _ = ClientboundOpenBookPacket::read(&mut slice);
    }
}
