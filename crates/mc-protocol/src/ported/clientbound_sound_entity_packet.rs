use bytes::{Buf, BufMut};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct DecodeError(pub String);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecodeError {}

fn read_varint<B: Buf>(buf: &mut B) -> Result<i32, DecodeError> {
    let mut result: i32 = 0;
    for shift in [0, 7, 14, 21, 28] {
        if !buf.has_remaining() {
            return Err(DecodeError("VarInt too short".into()));
        }
        let byte = buf.get_u8();
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
    }
    Err(DecodeError("VarInt too big".into()))
}

fn write_varint<B: BufMut>(buf: &mut B, mut value: i32) {
    let mut val = value as u32;
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        buf.put_u8(byte);
        if val == 0 {
            break;
        }
    }
    let _ = &mut value;
}

fn read_utf<B: Buf>(buf: &mut B, max_len: usize) -> Result<String, DecodeError> {
    let len = read_varint(buf)? as usize;
    if len > max_len * 3 + 3 {
        return Err(DecodeError("String too long".into()));
    }
    if buf.remaining() < len {
        return Err(DecodeError("String buffer too short".into()));
    }
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    let s = String::from_utf8(bytes).map_err(|e| DecodeError(e.to_string()))?;
    if s.chars().count() > max_len {
        return Err(DecodeError("String too long".into()));
    }
    Ok(s)
}

fn write_utf<B: BufMut>(buf: &mut B, value: &str) {
    let bytes = value.as_bytes();
    write_varint(buf, bytes.len() as i32);
    buf.put_slice(bytes);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SoundSource {
    Master = 0,
    Music = 1,
    Records = 2,
    Weather = 3,
    Blocks = 4,
    Hostile = 5,
    Neutral = 6,
    Players = 7,
    Ambient = 8,
    Voice = 9,
    Ui = 10,
}

impl SoundSource {
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
            10 => Ok(Self::Ui),
            _ => Err(DecodeError(format!("Unknown SoundSource id: {id}"))),
        }
    }

    pub fn id(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SoundEventHolder {
    Reference(i32),
    Direct {
        location: String,
        fixed_range: Option<f32>,
    },
}

impl SoundEventHolder {
    // JAVA: net.minecraft.core.Holder#codec / SoundEvent.STREAM_CODEC (decode)
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, DecodeError> {
        let raw = read_varint(buf)?;
        if raw == 0 {
            let location = read_utf(buf, 32767)?;
            if !buf.has_remaining() {
                return Err(DecodeError("Missing fixed_range flag".into()));
            }
            let has_range = buf.get_u8() != 0;
            let fixed_range = if has_range {
                if buf.remaining() < 4 {
                    return Err(DecodeError("Missing fixed_range value".into()));
                }
                Some(buf.get_f32())
            } else {
                None
            };
            Ok(Self::Direct {
                location,
                fixed_range,
            })
        } else {
            Ok(Self::Reference(raw.wrapping_sub(1)))
        }
    }

    // JAVA: net.minecraft.core.Holder#codec / SoundEvent.STREAM_CODEC (encode)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        match self {
            Self::Reference(id) => {
                write_varint(buf, id.wrapping_add(1));
            }
            Self::Direct {
                location,
                fixed_range,
            } => {
                write_varint(buf, 0);
                write_utf(buf, location);
                match fixed_range {
                    Some(range) => {
                        buf.put_u8(1);
                        buf.put_f32(*range);
                    }
                    None => {
                        buf.put_u8(0);
                    }
                }
            }
        }
    }
}

pub trait EntityLike {
    fn get_id(&self) -> i32;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSoundEntityPacket {
    sound: SoundEventHolder,
    source: SoundSource,
    id: i32,
    volume: f32,
    pitch: f32,
    seed: i64,
}

impl ClientboundSoundEntityPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#<init>(Holder,SoundSource,Entity,float,float,long)
    pub fn new(
        sound: SoundEventHolder,
        source: SoundSource,
        entity_id: i32,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self {
            sound,
            source,
            id: entity_id,
            volume,
            pitch,
            seed,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#<init>(RegistryFriendlyByteBuf)
    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, DecodeError> {
        let sound = SoundEventHolder::read(buf)?;
        let source = SoundSource::from_id(read_varint(buf)?)?;
        let id = read_varint(buf)?;
        if buf.remaining() < 4 {
            return Err(DecodeError("Missing volume".into()));
        }
        let volume = buf.get_f32();
        if buf.remaining() < 4 {
            return Err(DecodeError("Missing pitch".into()));
        }
        let pitch = buf.get_f32();
        if buf.remaining() < 8 {
            return Err(DecodeError("Missing seed".into()));
        }
        let seed = buf.get_i64();
        Ok(Self {
            sound,
            source,
            id,
            volume,
            pitch,
            seed,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#write(RegistryFriendlyByteBuf)
    pub fn write<B: BufMut>(&self, buf: &mut B) {
        self.sound.write(buf);
        write_varint(buf, self.source.id());
        write_varint(buf, self.id);
        buf.put_f32(self.volume);
        buf.put_f32(self.pitch);
        buf.put_i64(self.seed);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#getSound()
    pub fn get_sound(&self) -> &SoundEventHolder {
        &self.sound
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#getSource()
    pub fn get_source(&self) -> SoundSource {
        self.source
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#getId()
    pub fn get_id(&self) -> i32 {
        self.id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#getVolume()
    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#getPitch()
    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundEntityPacket#getSeed()
    pub fn get_seed(&self) -> i64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_reference_sound() {
        let pkt = ClientboundSoundEntityPacket::new(
            SoundEventHolder::Reference(42),
            SoundSource::Players,
            123,
            1.0,
            0.5,
            0x0123_4567_89AB_CDEF_i64,
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSoundEntityPacket::read(&mut reader).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_direct_sound_with_range() {
        let pkt = ClientboundSoundEntityPacket::new(
            SoundEventHolder::Direct {
                location: "minecraft:entity.pig.ambient".to_string(),
                fixed_range: Some(16.0),
            },
            SoundSource::Neutral,
            -1,
            0.0,
            2.0,
            -1,
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSoundEntityPacket::read(&mut reader).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn roundtrip_direct_sound_without_range_and_negative_values() {
        let pkt = ClientboundSoundEntityPacket::new(
            SoundEventHolder::Direct {
                location: "custom:sound".to_string(),
                fixed_range: None,
            },
            SoundSource::Master,
            i32::MIN,
            -f32::INFINITY,
            f32::NAN,
            i64::MIN,
        );
        let mut buf = BytesMut::new();
        pkt.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundSoundEntityPacket::read(&mut reader).unwrap();
        assert_eq!(decoded.get_id(), i32::MIN);
        assert_eq!(decoded.get_seed(), i64::MIN);
        assert!(decoded.get_volume().is_infinite() && decoded.get_volume().is_sign_negative());
        assert!(decoded.get_pitch().is_nan());
        assert_eq!(decoded.get_sound(), pkt.get_sound());
    }

    #[test]
    fn varint_roundtrip_edge_values() {
        for v in [0, 1, 127, 128, 255, -1, -2147483648_i32, 2147483647_i32] {
            let mut buf = BytesMut::new();
            write_varint(&mut buf, v);
            let mut reader = buf.freeze();
            let decoded = read_varint(&mut reader).unwrap();
            assert_eq!(decoded, v, "varint mismatch for {v}");
        }
    }

    #[test]
    fn sound_source_from_id_rejects_invalid() {
        assert!(SoundSource::from_id(-1).is_err());
        assert!(SoundSource::from_id(11).is_err());
        assert_eq!(SoundSource::from_id(10).unwrap(), SoundSource::Ui);
    }
}
