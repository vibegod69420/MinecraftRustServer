use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketType(pub u32);

pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const SERVERBOUND_CHANGE_DIFFICULTY: PacketType = PacketType(0x02);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Difficulty {
    Peaceful = 0,
    Easy = 1,
    Normal = 2,
    Hard = 3,
}

impl Difficulty {
    // JAVA: net.minecraft.world.Difficulty#byId
    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::Easy,
            2 => Self::Normal,
            3 => Self::Hard,
            _ => Self::Peaceful,
        }
    }

    // JAVA: net.minecraft.world.Difficulty#getId
    pub fn id(self) -> i32 {
        self as i32
    }

    // JAVA: net.minecraft.world.Difficulty#STREAM_CODEC
    pub fn encode<B: BufMut>(self, buf: &mut B) {
        buf.put_u8(self.id() as u8);
    }

    // JAVA: net.minecraft.world.Difficulty#STREAM_CODEC
    pub fn decode<B: Buf>(buf: &mut B) -> Self {
        Self::by_id(i32::from(buf.get_u8()))
    }
}

pub trait ServerGamePacketListener {
    fn handle_change_difficulty(&mut self, packet: &ServerboundChangeDifficultyPacket);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundChangeDifficultyPacket {
    difficulty: Difficulty,
}

impl ServerboundChangeDifficultyPacket {
    // JAVA: net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket#<init>
    pub fn new(difficulty: Difficulty) -> Self {
        Self { difficulty }
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket#difficulty
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket#type
    pub fn packet_type(&self) -> PacketType {
        GamePacketTypes::SERVERBOUND_CHANGE_DIFFICULTY
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket#handle
    pub fn handle(&self, listener: &mut dyn ServerGamePacketListener) {
        listener.handle_change_difficulty(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket#STREAM_CODEC
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        self.difficulty.encode(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket#STREAM_CODEC
    pub fn decode<B: Buf>(buf: &mut B) -> Self {
        Self::new(Difficulty::decode(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        seen: Option<Difficulty>,
    }

    impl ServerGamePacketListener for TestListener {
        fn handle_change_difficulty(&mut self, packet: &ServerboundChangeDifficultyPacket) {
            self.seen = Some(packet.difficulty());
        }
    }

    #[test]
    fn roundtrip_all_difficulties() {
        for d in [
            Difficulty::Peaceful,
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
        ] {
            let pkt = ServerboundChangeDifficultyPacket::new(d);
            let mut buf = BytesMut::new();
            pkt.encode(&mut buf);
            let decoded = ServerboundChangeDifficultyPacket::decode(&mut buf.freeze());
            assert_eq!(decoded, pkt);
        }
    }

    #[test]
    fn by_id_negative_and_overflow_edges() {
        assert_eq!(Difficulty::by_id(-1), Difficulty::Peaceful);
        assert_eq!(Difficulty::by_id(i32::MIN), Difficulty::Peaceful);
        assert_eq!(Difficulty::by_id(i32::MAX), Difficulty::Peaceful);
        assert_eq!(Difficulty::by_id(4), Difficulty::Peaceful);
        let wrapped = 3i32.wrapping_add(1);
        assert_eq!(Difficulty::by_id(wrapped), Difficulty::Peaceful);
    }

    #[test]
    fn by_id_float_bits_pattern() {
        let nan_bits = f32::NAN.to_bits() as i32;
        let inf_bits = f32::INFINITY.to_bits() as i32;
        let neg_inf_bits = f32::NEG_INFINITY.to_bits() as i32;
        for id in [nan_bits, inf_bits, neg_inf_bits] {
            let d = Difficulty::by_id(id);
            assert!(matches!(
                d,
                Difficulty::Peaceful | Difficulty::Easy | Difficulty::Normal | Difficulty::Hard
            ));
        }
    }

    #[test]
    fn handle_and_type() {
        let pkt = ServerboundChangeDifficultyPacket::new(Difficulty::Hard);
        assert_eq!(
            pkt.packet_type(),
            GamePacketTypes::SERVERBOUND_CHANGE_DIFFICULTY
        );
        let mut listener = TestListener { seen: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen, Some(Difficulty::Hard));
    }
}
