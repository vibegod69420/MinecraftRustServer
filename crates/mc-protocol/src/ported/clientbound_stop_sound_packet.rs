use bytes::{Buf, BufMut, BytesMut};

const HAS_SOURCE: u8 = 1;
const HAS_SOUND: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub String);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
}

impl SoundSource {
    // JAVA: net.minecraft.sounds.SoundSource#ordinal
    pub fn id(self) -> i32 {
        match self {
            Self::Master => 0,
            Self::Music => 1,
            Self::Records => 2,
            Self::Weather => 3,
            Self::Blocks => 4,
            Self::Hostile => 5,
            Self::Neutral => 6,
            Self::Players => 7,
            Self::Ambient => 8,
            Self::Voice => 9,
        }
    }

    // JAVA: net.minecraft.sounds.SoundSource#values
    pub fn from_id(id: i32) -> Result<Self, DecodeError> {
        match id {
            0 => Ok(Self::Master),
            1 => Ok(Self::Music),
            2 => Ok(Self::Records),
            3 => Ok(Self::Weather),
            4 => Ok(Self::Blocks),
            5 => Ok(Self::Hostile),
            6 => Ok(Self::Neutral),
            7 => Ok(Self::Players),
            8 => Ok(Self::Ambient),
            9 => Ok(Self::Voice),
            _ => Err(DecodeError(format!("Unknown SoundSource id: {id}"))),
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_varint(buf: &mut &[u8]) -> Result<i32, DecodeError> {
    let mut value: i32 = 0;
    let mut position: i32 = 0;
    loop {
        if !buf.has_remaining() {
            return Err(DecodeError("VarInt too short".into()));
        }
        let read = buf.get_u8();
        value |= ((read & 0x7F) as i32) << position;
        if (read & 0x80) == 0 {
            return Ok(value);
        }
        position += 7;
        if position >= 32 {
            return Err(DecodeError("VarInt too big".into()));
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_varint(buf: &mut BytesMut, mut value: i32) {
    loop {
        let mut temp = (value as u32 & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            temp |= 0x80;
        }
        buf.put_u8(temp);
        if value == 0 {
            return;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readUtf
fn read_utf(buf: &mut &[u8], max_len: usize) -> Result<String, DecodeError> {
    let len = read_varint(buf)?;
    if len < 0 {
        return Err(DecodeError("Negative string length".into()));
    }
    let len = len as usize;
    let max_bytes = max_len.saturating_mul(3);
    if len > max_bytes {
        return Err(DecodeError("String too long".into()));
    }
    if buf.remaining() < len {
        return Err(DecodeError("String buffer underflow".into()));
    }
    let bytes = &buf[..len];
    buf.advance(len);
    let s = String::from_utf8(bytes.to_vec())
        .map_err(|e| DecodeError(format!("Invalid UTF-8: {e}")))?;
    if s.chars().count() > max_len {
        return Err(DecodeError("String too long".into()));
    }
    Ok(s)
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeUtf
fn write_utf(buf: &mut BytesMut, s: &str, max_len: usize) -> Result<(), DecodeError> {
    if s.chars().count() > max_len {
        return Err(DecodeError("String too long".into()));
    }
    let bytes = s.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
    Ok(())
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readIdentifier
fn read_identifier(buf: &mut &[u8]) -> Result<Identifier, DecodeError> {
    Ok(Identifier(read_utf(buf, 32767)?))
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeIdentifier
fn write_identifier(buf: &mut BytesMut, id: &Identifier) -> Result<(), DecodeError> {
    write_utf(buf, &id.0, 32767)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStopSoundPacket {
    name: Option<Identifier>,
    source: Option<SoundSource>,
}

impl ClientboundStopSoundPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundStopSoundPacket#<init>
    pub fn new(name: Option<Identifier>, source: Option<SoundSource>) -> Self {
        Self { name, source }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundStopSoundPacket#<init>
    pub fn read(buf: &mut &[u8]) -> Result<Self, DecodeError> {
        if !buf.has_remaining() {
            return Err(DecodeError("Buffer underflow".into()));
        }
        let flags = buf.get_u8();
        let source = if (flags & HAS_SOURCE) > 0 {
            let id = read_varint(buf)?;
            Some(SoundSource::from_id(id)?)
        } else {
            None
        };
        let name = if (flags & HAS_SOUND) > 0 {
            Some(read_identifier(buf)?)
        } else {
            None
        };
        Ok(Self { name, source })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundStopSoundPacket#write
    pub fn write(&self, buf: &mut BytesMut) -> Result<(), DecodeError> {
        match (&self.source, &self.name) {
            (Some(source), Some(name)) => {
                buf.put_u8(HAS_SOURCE | HAS_SOUND);
                write_varint(buf, source.id());
                write_identifier(buf, name)?;
            }
            (Some(source), None) => {
                buf.put_u8(HAS_SOURCE);
                write_varint(buf, source.id());
            }
            (None, Some(name)) => {
                buf.put_u8(HAS_SOUND);
                write_identifier(buf, name)?;
            }
            (None, None) => {
                buf.put_u8(0);
            }
        }
        Ok(())
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundStopSoundPacket#getName
    pub fn get_name(&self) -> Option<&Identifier> {
        self.name.as_ref()
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundStopSoundPacket#getSource
    pub fn get_source(&self) -> Option<SoundSource> {
        self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(pkt: &ClientboundStopSoundPacket) -> ClientboundStopSoundPacket {
        let mut buf = BytesMut::new();
        pkt.write(&mut buf).unwrap();
        let mut slice: &[u8] = &buf;
        let decoded = ClientboundStopSoundPacket::read(&mut slice).unwrap();
        assert!(!slice.has_remaining());
        decoded
    }

    #[test]
    fn round_trip_all_flag_combinations() {
        let none = ClientboundStopSoundPacket::new(None, None);
        let source_only =
            ClientboundStopSoundPacket::new(None, Some(SoundSource::Weather));
        let name_only = ClientboundStopSoundPacket::new(
            Some(Identifier("minecraft:music.game".into())),
            None,
        );
        let both = ClientboundStopSoundPacket::new(
            Some(Identifier("minecraft:block.note_block.harp".into())),
            Some(SoundSource::Records),
        );
        assert_eq!(round_trip(&none), none);
        assert_eq!(round_trip(&source_only), source_only);
        assert_eq!(round_trip(&name_only), name_only);
        assert_eq!(round_trip(&both), both);
    }

    #[test]
    fn wire_format_both_flags() {
        let pkt = ClientboundStopSoundPacket::new(
            Some(Identifier("minecraft:test".into())),
            Some(SoundSource::Master),
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf).unwrap();
        assert_eq!(buf[0], HAS_SOURCE | HAS_SOUND);
        assert_eq!(buf[1], 0); // SoundSource::Master id
    }

    #[test]
    fn decode_invalid_sound_source() {
        let mut buf = BytesMut::new();
        buf.put_u8(HAS_SOURCE);
        write_varint(&mut buf, 999);
        let mut slice: &[u8] = &buf;
        let err = ClientboundStopSoundPacket::read(&mut slice).unwrap_err();
        assert!(err.to_string().contains("Unknown SoundSource"));
    }

    #[test]
    fn decode_underflow_and_negative_length() {
        let mut empty: &[u8] = &[];
        assert!(ClientboundStopSoundPacket::read(&mut empty).is_err());

        let mut buf = BytesMut::new();
        buf.put_u8(HAS_SOUND);
        write_varint(&mut buf, -1);
        let mut slice: &[u8] = &buf;
        assert!(ClientboundStopSoundPacket::read(&mut slice).is_err());
    }

    #[test]
    fn getters_match_constructor() {
        let name = Identifier("minecraft:ambient.cave".into());
        let pkt = ClientboundStopSoundPacket::new(Some(name.clone()), Some(SoundSource::Ambient));
        assert_eq!(pkt.get_name(), Some(&name));
        assert_eq!(pkt.get_source(), Some(SoundSource::Ambient));
    }
}
