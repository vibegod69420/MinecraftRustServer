use bytes::{Buf, BufMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub &'static str);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for DecodeError {}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
pub fn read_var_int(buf: &mut impl Buf) -> Result<i32, DecodeError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecodeError("VarInt too short"));
        }
        let b = buf.get_u8();
        result |= ((b & 0x7F) as i32).wrapping_shl(shift & 31);
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift = shift.wrapping_add(7);
        if shift >= 35 {
            return Err(DecodeError("VarInt too big"));
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
pub fn write_var_int(buf: &mut impl BufMut, mut value: i32) {
    loop {
        let mut b = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            b |= 0x80;
            buf.put_u8(b);
        } else {
            buf.put_u8(b);
            return;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readBoolean
pub fn read_bool(buf: &mut impl Buf) -> Result<bool, DecodeError> {
    if !buf.has_remaining() {
        return Err(DecodeError("Not enough bytes for boolean"));
    }
    Ok(buf.get_u8() != 0)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeBoolean
pub fn write_bool(buf: &mut impl BufMut, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    // JAVA: net.minecraft.world.Difficulty#getId
    pub fn id(self) -> i32 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }

    // JAVA: net.minecraft.world.Difficulty#byId
    pub fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            3 => Self::Hard,
            _ => Self::Peaceful,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundChangeDifficultyPacket {
    pub difficulty: Difficulty,
    pub locked: bool,
}

pub trait ClientGamePacketListener {
    fn handle_change_difficulty(&mut self, packet: &ClientboundChangeDifficultyPacket);
}

impl ClientboundChangeDifficultyPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket#<init>
    pub fn new(difficulty: Difficulty, locked: bool) -> Self {
        Self { difficulty, locked }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket#STREAM_CODEC
    pub fn read(buf: &mut impl Buf) -> Result<Self, DecodeError> {
        let difficulty = Difficulty::by_id(read_var_int(buf)?);
        let locked = read_bool(buf)?;
        Ok(Self::new(difficulty, locked))
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket#STREAM_CODEC
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.difficulty.id());
        write_bool(buf, self.locked);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:change_difficulty")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_change_difficulty(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BytesMut};

    #[test]
    fn difficulty_id_roundtrip() {
        for d in [
            Difficulty::Peaceful,
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
        ] {
            assert_eq!(Difficulty::by_id(d.id()), d);
        }
    }

    #[test]
    fn difficulty_unknown_id_defaults() {
        assert_eq!(Difficulty::by_id(-1), Difficulty::Peaceful);
        assert_eq!(Difficulty::by_id(4), Difficulty::Peaceful);
        assert_eq!(Difficulty::by_id(i32::MIN), Difficulty::Peaceful);
        assert_eq!(Difficulty::by_id(i32::MAX), Difficulty::Peaceful);
    }

    #[test]
    fn packet_roundtrip() {
        let packet = ClientboundChangeDifficultyPacket::new(Difficulty::Hard, true);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut slice = &buf[..];
        let decoded = ClientboundChangeDifficultyPacket::read(&mut slice).unwrap();
        assert_eq!(decoded, packet);
        assert!(!slice.has_remaining());
    }

    #[test]
    fn packet_roundtrip_all_combinations() {
        for d in [
            Difficulty::Peaceful,
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
        ] {
            for locked in [false, true] {
                let packet = ClientboundChangeDifficultyPacket::new(d, locked);
                let mut buf = BytesMut::new();
                packet.write(&mut buf);
                let mut slice = &buf[..];
                let decoded = ClientboundChangeDifficultyPacket::read(&mut slice).unwrap();
                assert_eq!(decoded, packet);
            }
        }
    }

    #[test]
    fn handle_dispatches() {
        struct MockListener(Option<(Difficulty, bool)>);
        impl ClientGamePacketListener for MockListener {
            fn handle_change_difficulty(
                &mut self,
                packet: &ClientboundChangeDifficultyPacket,
            ) {
                self.0 = Some((packet.difficulty, packet.locked));
            }
        }
        let packet = ClientboundChangeDifficultyPacket::new(Difficulty::Normal, false);
        let mut listener = MockListener(None);
        packet.handle(&mut listener);
        assert_eq!(listener.0, Some((Difficulty::Normal, false)));
        assert_eq!(
            packet.packet_type(),
            PacketType("minecraft:change_difficulty")
        );
    }
}
