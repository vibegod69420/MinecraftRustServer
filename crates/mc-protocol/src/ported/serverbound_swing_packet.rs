use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FriendlyByteBuf {
    pub data: BytesMut,
    pub reader_index: usize,
}

impl FriendlyByteBuf {
    pub fn new(data: BytesMut) -> Self {
        Self {
            data,
            reader_index: 0,
        }
    }

    fn read_byte(&mut self) -> i8 {
        let b = self.data[self.reader_index] as i8;
        self.reader_index += 1;
        b
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
    pub fn read_var_int(&mut self) -> i32 {
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        loop {
            let b0 = self.read_byte();
            i |= ((b0 as i32) & 127) << (j.wrapping_mul(7) & 31);
            j = j.wrapping_add(1);
            if j > 5 {
                panic!("VarInt too big");
            }
            if ((b0 as u8) & 0x80) == 0 {
                break;
            }
        }
        i
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let mut b = (value & 0x7F) as u8;
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

    // JAVA: net.minecraft.network.FriendlyByteBuf#readEnum
    pub fn read_enum_interaction_hand(&mut self) -> InteractionHand {
        match self.read_var_int() {
            0 => InteractionHand::MainHand,
            1 => InteractionHand::OffHand,
            v => panic!("Invalid InteractionHand ordinal: {}", v),
        }
    }

    // JAVA: net.minecraft.network.FriendlyByteBuf#writeEnum
    pub fn write_enum_interaction_hand(&mut self, hand: InteractionHand) {
        self.write_var_int(hand.ordinal());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    MainHand,
    OffHand,
}

impl InteractionHand {
    pub fn ordinal(self) -> i32 {
        match self {
            InteractionHand::MainHand => 0,
            InteractionHand::OffHand => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType {
    pub id: &'static str,
}

pub const SERVERBOUND_SWING_TYPE: PacketType = PacketType {
    id: "minecraft:swing",
};

pub trait ServerGamePacketListener {
    fn handle_animate(&mut self, packet: &ServerboundSwingPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSwingPacket {
    hand: InteractionHand,
}

impl ServerboundSwingPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundSwingPacket#<init>
    pub fn new(hand: InteractionHand) -> Self {
        Self { hand }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSwingPacket#<init>
    pub fn from_buf(buf: &mut FriendlyByteBuf) -> Self {
        Self {
            hand: buf.read_enum_interaction_hand(),
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSwingPacket#write
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_enum_interaction_hand(self.hand);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSwingPacket#type
    pub fn get_type(&self) -> PacketType {
        SERVERBOUND_SWING_TYPE
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSwingPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_animate(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundSwingPacket#getHand
    pub fn get_hand(&self) -> InteractionHand {
        self.hand
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swing_round_trip_main_hand() {
        let pkt = ServerboundSwingPacket::new(InteractionHand::MainHand);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        buf.reader_index = 0;
        let decoded = ServerboundSwingPacket::from_buf(&mut buf);
        assert_eq!(pkt, decoded);
        assert_eq!(decoded.get_hand(), InteractionHand::MainHand);
    }

    #[test]
    fn swing_round_trip_off_hand() {
        let pkt = ServerboundSwingPacket::new(InteractionHand::OffHand);
        let mut buf = FriendlyByteBuf::default();
        pkt.write(&mut buf);
        buf.reader_index = 0;
        let decoded = ServerboundSwingPacket::from_buf(&mut buf);
        assert_eq!(decoded.get_hand(), InteractionHand::OffHand);
        assert_eq!(decoded.get_type(), SERVERBOUND_SWING_TYPE);
    }

    #[test]
    fn varint_round_trip_negative_and_edges() {
        for v in [0, 1, 127, 128, 255, -1, -128, i32::MAX, i32::MIN] {
            let mut buf = FriendlyByteBuf::default();
            buf.write_var_int(v);
            buf.reader_index = 0;
            assert_eq!(buf.read_var_int(), v, "varint mismatch for {}", v);
        }
    }

    #[test]
    fn varint_encoding_negative_one_is_five_bytes() {
        let mut buf = FriendlyByteBuf::default();
        buf.write_var_int(-1);
        assert_eq!(buf.data.as_ref(), &[0xff, 0xff, 0xff, 0xff, 0x0f]);
    }

    struct MockListener {
        called: bool,
        hand: Option<InteractionHand>,
    }

    impl ServerGamePacketListener for MockListener {
        fn handle_animate(&mut self, packet: &ServerboundSwingPacket) {
            self.called = true;
            self.hand = Some(packet.get_hand());
        }
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ServerboundSwingPacket::new(InteractionHand::OffHand);
        let mut listener = MockListener {
            called: false,
            hand: None,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.hand, Some(InteractionHand::OffHand));
    }
}
