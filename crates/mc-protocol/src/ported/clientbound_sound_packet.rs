pub const LOCATION_ACCURACY: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundSource(pub i32);

impl SoundSource {
    pub fn from_ordinal(ordinal: i32) -> Self {
        Self(ordinal)
    }

    pub fn ordinal(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSoundEvent {
    pub network_id: i32,
}

impl HolderSoundEvent {
    fn stream_decode(buf: &mut RegistryFriendlyByteBuf) -> Self {
        Self {
            network_id: buf.read_var_int(),
        }
    }

    fn stream_encode(buf: &mut RegistryFriendlyByteBuf, sound: &Self) {
        buf.write_var_int(sound.network_id);
    }
}

#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    read_pos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, read_pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn read_int(&mut self) -> i32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 4]);
        self.read_pos += 4;
        i32::from_be_bytes(bytes)
    }

    pub fn write_int(&mut self, v: i32) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    pub fn read_long(&mut self) -> i64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.read_pos..self.read_pos + 8]);
        self.read_pos += 8;
        i64::from_be_bytes(bytes)
    }

    pub fn write_long(&mut self, v: i64) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    pub fn read_float(&mut self) -> f32 {
        f32::from_bits(self.read_int() as u32)
    }

    pub fn write_float(&mut self, v: f32) {
        self.write_int(v.to_bits() as i32);
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut value: i32 = 0;
        let mut position: u32 = 0;
        loop {
            let current_byte = self.data[self.read_pos];
            self.read_pos += 1;
            value |= ((current_byte & 0x7F) as i32) << position;
            if (current_byte & 0x80) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("VarInt too big");
            }
        }
        value
    }

    pub fn write_var_int(&mut self, value: i32) {
        let mut val = value as u32;
        loop {
            if (val & !0x7F) == 0 {
                self.data.push(val as u8);
                return;
            }
            self.data.push(((val & 0x7F) | 0x80) as u8);
            val >>= 7;
        }
    }

    pub fn read_enum(&mut self) -> i32 {
        self.read_var_int()
    }

    pub fn write_enum(&mut self, ordinal: i32) {
        self.write_var_int(ordinal);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

pub trait ClientGamePacketListener {
    fn handle_sound_event(&mut self, packet: &ClientboundSoundPacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSoundPacket {
    sound: HolderSoundEvent,
    source: SoundSource,
    x: i32,
    y: i32,
    z: i32,
    volume: f32,
    pitch: f32,
    seed: i64,
}

impl ClientboundSoundPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#<init>
    pub fn new(
        sound: HolderSoundEvent,
        source: SoundSource,
        x: f64,
        y: f64,
        z: f64,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self {
            sound,
            source,
            x: (x * 8.0f64) as i32,
            y: (y * 8.0f64) as i32,
            z: (z * 8.0f64) as i32,
            volume,
            pitch,
            seed,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#<init>
    pub fn from_buf(buf: &mut RegistryFriendlyByteBuf) -> Self {
        let sound = HolderSoundEvent::stream_decode(buf);
        let source = SoundSource::from_ordinal(buf.read_enum());
        let x = buf.read_int();
        let y = buf.read_int();
        let z = buf.read_int();
        let volume = buf.read_float();
        let pitch = buf.read_float();
        let seed = buf.read_long();
        Self {
            sound,
            source,
            x,
            y,
            z,
            volume,
            pitch,
            seed,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        HolderSoundEvent::stream_encode(buf, &self.sound);
        buf.write_enum(self.source.ordinal());
        buf.write_int(self.x);
        buf.write_int(self.y);
        buf.write_int(self.z);
        buf.write_float(self.volume);
        buf.write_float(self.pitch);
        buf.write_long(self.seed);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#type
    pub fn get_type(&self) -> PacketType {
        PacketType("minecraft:sound")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#handle
    pub fn handle(&self, listener: &mut dyn ClientGamePacketListener) {
        listener.handle_sound_event(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getSound
    pub fn get_sound(&self) -> &HolderSoundEvent {
        &self.sound
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getSource
    pub fn get_source(&self) -> SoundSource {
        self.source
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getX
    pub fn get_x(&self) -> f64 {
        (self.x as f32 / 8.0f32) as f64
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getY
    pub fn get_y(&self) -> f64 {
        (self.y as f32 / 8.0f32) as f64
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getZ
    pub fn get_z(&self) -> f64 {
        (self.z as f32 / 8.0f32) as f64
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getVolume
    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getPitch
    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundSoundPacket#getSeed
    pub fn get_seed(&self) -> i64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_scaling_and_getters() {
        let pkt = ClientboundSoundPacket::new(
            HolderSoundEvent { network_id: 1 },
            SoundSource::from_ordinal(0),
            1.5,
            -2.25,
            0.125,
            1.0,
            1.0,
            42,
        );
        assert_eq!(pkt.x, 12);
        assert_eq!(pkt.y, -18);
        assert_eq!(pkt.z, 1);
        assert_eq!(pkt.get_x(), (12f32 / 8.0f32) as f64);
        assert_eq!(pkt.get_y(), (-18f32 / 8.0f32) as f64);
        assert_eq!(pkt.get_z(), (1f32 / 8.0f32) as f64);
    }

    #[test]
    fn write_read_roundtrip() {
        let pkt = ClientboundSoundPacket::new(
            HolderSoundEvent { network_id: 42 },
            SoundSource::from_ordinal(3),
            10.0,
            -20.5,
            30.25,
            0.75,
            1.25,
            -123_456_789_i64,
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundSoundPacket::from_buf(&mut read_buf);
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn negative_and_truncation_coords() {
        let pkt = ClientboundSoundPacket::new(
            HolderSoundEvent { network_id: 0 },
            SoundSource::from_ordinal(0),
            -0.125,
            -1.0,
            0.99,
            0.0,
            0.0,
            0,
        );
        assert_eq!(pkt.x, -1);
        assert_eq!(pkt.y, -8);
        assert_eq!(pkt.z, 7);
        assert_eq!(pkt.get_source(), SoundSource::from_ordinal(0));
    }

    #[test]
    fn float_bits_and_seed_preserved() {
        let vol_bits = 0x3F8C_CCCD_u32; // 1.1f32
        let pitch_bits = 0xBF80_0000_u32; // -1.0f32
        let pkt = ClientboundSoundPacket::new(
            HolderSoundEvent { network_id: 7 },
            SoundSource::from_ordinal(2),
            0.0,
            0.0,
            0.0,
            f32::from_bits(vol_bits),
            f32::from_bits(pitch_bits),
            i64::MIN,
        );
        let mut buf = RegistryFriendlyByteBuf::new();
        pkt.write(&mut buf);
        let mut read_buf = RegistryFriendlyByteBuf::from_bytes(buf.into_inner());
        let decoded = ClientboundSoundPacket::from_buf(&mut read_buf);
        assert_eq!(decoded.get_volume().to_bits(), vol_bits);
        assert_eq!(decoded.get_pitch().to_bits(), pitch_bits);
        assert_eq!(decoded.get_seed(), i64::MIN);
        assert_eq!(decoded.get_sound(), &HolderSoundEvent { network_id: 7 });
    }
}
