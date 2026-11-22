use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub u32);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const SERVERBOUND_PLAYER_INPUT: PacketType = PacketType(0x1A);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Input {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}

impl Input {
    // JAVA: net.minecraft.world.entity.player.Input#<init>
    pub fn new(
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        jump: bool,
        shift: bool,
        sprint: bool,
    ) -> Self {
        Self {
            forward,
            backward,
            left,
            right,
            jump,
            shift,
            sprint,
        }
    }

    // JAVA: net.minecraft.world.entity.player.Input#STREAM_CODEC
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        let mut flags: u8 = 0;
        if self.forward {
            flags |= 0x01;
        }
        if self.backward {
            flags |= 0x02;
        }
        if self.left {
            flags |= 0x04;
        }
        if self.right {
            flags |= 0x08;
        }
        if self.jump {
            flags |= 0x10;
        }
        if self.shift {
            flags |= 0x20;
        }
        if self.sprint {
            flags |= 0x40;
        }
        buf.put_u8(flags);
    }

    // JAVA: net.minecraft.world.entity.player.Input#STREAM_CODEC
    pub fn decode<B: Buf>(buf: &mut B) -> Self {
        let flags = buf.get_u8();
        Self {
            forward: (flags & 0x01) != 0,
            backward: (flags & 0x02) != 0,
            left: (flags & 0x04) != 0,
            right: (flags & 0x08) != 0,
            jump: (flags & 0x10) != 0,
            shift: (flags & 0x20) != 0,
            sprint: (flags & 0x40) != 0,
        }
    }
}

pub trait ServerGamePacketListener {
    fn handle_player_input(&mut self, packet: &ServerboundPlayerInputPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerInputPacket {
    input: Input,
}

impl ServerboundPlayerInputPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerInputPacket#<init>
    pub fn new(input: Input) -> Self {
        Self { input }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerInputPacket#input
    pub fn input(&self) -> Input {
        self.input
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerInputPacket#type
    pub fn packet_type(&self) -> PacketType {
        GamePacketTypes::SERVERBOUND_PLAYER_INPUT
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerInputPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_player_input(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerInputPacket#STREAM_CODEC
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        self.input.encode(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundPlayerInputPacket#STREAM_CODEC
    pub fn decode<B: Buf>(buf: &mut B) -> Self {
        Self::new(Input::decode(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        called: bool,
        forward: bool,
        sprint: bool,
    }

    impl ServerGamePacketListener for TestListener {
        fn handle_player_input(&mut self, packet: &ServerboundPlayerInputPacket) {
            self.called = true;
            let input = packet.input();
            self.forward = input.forward;
            self.sprint = input.sprint;
        }
    }

    #[test]
    fn roundtrip_all_flags_set() {
        let pkt = ServerboundPlayerInputPacket::new(Input::new(
            true, true, true, true, true, true, true,
        ));
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let decoded = ServerboundPlayerInputPacket::decode(&mut buf.freeze());
        assert_eq!(decoded, pkt);
        assert_eq!(decoded.input().jump, true);
    }

    #[test]
    fn roundtrip_no_flags_set() {
        let pkt = ServerboundPlayerInputPacket::new(Input::default());
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf);
        let decoded = ServerboundPlayerInputPacket::decode(&mut buf.freeze());
        assert_eq!(decoded, pkt);
        assert_eq!(decoded.packet_type(), GamePacketTypes::SERVERBOUND_PLAYER_INPUT);
    }

    #[test]
    fn decode_ignores_unknown_high_bit_and_handles_edge_byte() {
        let mut raw: &[u8] = &[0xFFu8];
        let decoded = Input::decode(&mut raw);
        assert!(decoded.forward);
        assert!(decoded.sprint);
        assert!(!decoded.backward == false); // all known bits set

        let mut out = BytesMut::new();
        decoded.encode(&mut out);
        assert_eq!(out[0], 0x7F);
    }

    #[test]
    fn handle_dispatches_to_listener() {
        let pkt = ServerboundPlayerInputPacket::new(Input::new(
            true, false, false, false, false, false, true,
        ));
        let mut listener = TestListener {
            called: false,
            forward: false,
            sprint: false,
        };
        pkt.handle(&mut listener);
        assert!(listener.called);
        assert!(listener.forward);
        assert!(listener.sprint);
    }

    #[test]
    fn float_bits_and_negative_edge_patterns() {
        let bits = f32::NAN.to_bits();
        let flags = (bits & 0x7F) as u8;
        let mut buf: &[u8] = &[flags];
        let decoded = Input::decode(&mut buf);
        let mut out = BytesMut::new();
        decoded.encode(&mut out);
        assert_eq!(out[0], flags);

        let neg_i8: i8 = -1;
        let neg_byte = neg_i8 as u8;
        let mut buf2: &[u8] = &[neg_byte];
        let decoded2 = Input::decode(&mut buf2);
        assert!(decoded2.forward);
        assert!(decoded2.shift);

        let overflow_wrapped = (u8::MAX as u16).wrapping_add(1) as u8;
        let mut buf3: &[u8] = &[overflow_wrapped];
        let decoded3 = Input::decode(&mut buf3);
        assert_eq!(decoded3, Input::default());
    }
}
