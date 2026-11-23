use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub String);

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecodeError {}

/// Minimal self-contained byte buffer mirroring the methods used by this packet.
/// Wire format: booleans as 1 byte, numeric primitives big-endian,
/// float/double as raw IEEE 754 bits, VarInt as Minecraft variable-length int.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    rpos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, rpos: 0 }
    }

    pub fn writer() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.rpos)
    }

    fn read_exact_bytes(&mut self, n: usize) -> Result<&[u8], DecodeError> {
        if self.rpos.saturating_add(n) > self.data.len() {
            return Err(DecodeError("unexpected EOF".into()));
        }
        let start = self.rpos;
        self.rpos += n;
        Ok(&self.data[start..self.rpos])
    }

    pub fn read_boolean(&mut self) -> Result<bool, DecodeError> {
        Ok(self.read_exact_bytes(1)?[0] != 0)
    }

    pub fn write_boolean(&mut self, v: bool) {
        self.data.push(if v { 1 } else { 0 });
    }

    pub fn read_double(&mut self) -> Result<f64, DecodeError> {
        let b = self.read_exact_bytes(8)?;
        Ok(f64::from_bits(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ])))
    }

    pub fn write_double(&mut self, v: f64) {
        self.data.extend_from_slice(&v.to_bits().to_be_bytes());
    }

    pub fn read_float(&mut self) -> Result<f32, DecodeError> {
        let b = self.read_exact_bytes(4)?;
        Ok(f32::from_bits(u32::from_be_bytes([b[0], b[1], b[2], b[3]])))
    }

    pub fn write_float(&mut self, v: f32) {
        self.data.extend_from_slice(&v.to_bits().to_be_bytes());
    }

    pub fn read_int(&mut self) -> Result<i32, DecodeError> {
        let b = self.read_exact_bytes(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn write_int(&mut self, v: i32) {
        self.data.extend_from_slice(&v.to_be_bytes());
    }

    pub fn read_varint(&mut self) -> Result<i32, DecodeError> {
        let mut val: i32 = 0;
        for pos in 0..5u32 {
            let b = self.read_exact_bytes(1)?[0];
            val |= ((b & 0x7F) as i32) << (pos * 7);
            if (b & 0x80) == 0 {
                return Ok(val);
            }
        }
        Err(DecodeError("VarInt too big".into()))
    }

    pub fn write_varint(&mut self, val: i32) {
        let mut v = val as u32;
        loop {
            if (v & !0x7F) == 0 {
                self.data.push(v as u8);
                return;
            }
            self.data.push(((v & 0x7F) as u8) | 0x80);
            v >>= 7;
        }
    }
}

/// Self-contained particle options stub.
/// The real Java codec is registry-backed; this preserves packet structure and
/// roundtrips a particle type id as a VarInt for standalone compilation/tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleOptions {
    pub type_id: i32,
}

/// Minimal stand-in for `net.minecraft.core.particles.ParticleTypes` stream codec usage.
pub struct ParticleTypes;

impl ParticleTypes {
    fn stream_codec_decode(buf: &mut RegistryFriendlyByteBuf) -> Result<ParticleOptions, DecodeError> {
        let type_id = buf.read_varint()?;
        Ok(ParticleOptions { type_id })
    }

    fn stream_codec_encode(buf: &mut RegistryFriendlyByteBuf, particle: &ParticleOptions) {
        buf.write_varint(particle.type_id);
    }
}

/// Minimal packet type marker for self-contained compilation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketType(pub &'static str);

/// Minimal listener stub for self-contained compilation.
pub trait ClientGamePacketListener {
    fn handle_particle_event(&mut self, packet: &ClientboundLevelParticlesPacket);
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundLevelParticlesPacket {
    x: f64,
    y: f64,
    z: f64,
    x_dist: f32,
    y_dist: f32,
    z_dist: f32,
    max_speed: f32,
    count: i32,
    override_limiter: bool,
    always_show: bool,
    particle: ParticleOptions,
}

impl ClientboundLevelParticlesPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#<init>
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        particle: ParticleOptions,
        override_limiter: bool,
        always_show: bool,
        x: f64,
        y: f64,
        z: f64,
        x_dist: f32,
        y_dist: f32,
        z_dist: f32,
        max_speed: f32,
        count: i32,
    ) -> Self {
        Self {
            particle,
            override_limiter,
            always_show,
            x,
            y,
            z,
            x_dist,
            y_dist,
            z_dist,
            max_speed,
            count,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#<init>
    pub fn read(buf: &mut RegistryFriendlyByteBuf) -> Result<Self, DecodeError> {
        let override_limiter = buf.read_boolean()?;
        let always_show = buf.read_boolean()?;
        let x = buf.read_double()?;
        let y = buf.read_double()?;
        let z = buf.read_double()?;
        let x_dist = buf.read_float()?;
        let y_dist = buf.read_float()?;
        let z_dist = buf.read_float()?;
        let max_speed = buf.read_float()?;
        let count = buf.read_int()?;
        let particle = ParticleTypes::stream_codec_decode(buf)?;
        Ok(Self {
            x,
            y,
            z,
            x_dist,
            y_dist,
            z_dist,
            max_speed,
            count,
            override_limiter,
            always_show,
            particle,
        })
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        buf.write_boolean(self.override_limiter);
        buf.write_boolean(self.always_show);
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_float(self.x_dist);
        buf.write_float(self.y_dist);
        buf.write_float(self.z_dist);
        buf.write_float(self.max_speed);
        buf.write_int(self.count);
        ParticleTypes::stream_codec_encode(buf, &self.particle);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType("minecraft:clientbound_level_particles")
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#handle
    pub fn handle<L: ClientGamePacketListener>(&self, listener: &mut L) {
        listener.handle_particle_event(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#isOverrideLimiter
    pub fn is_override_limiter(&self) -> bool {
        self.override_limiter
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#alwaysShow
    pub fn always_show(&self) -> bool {
        self.always_show
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getX
    pub fn get_x(&self) -> f64 {
        self.x
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getY
    pub fn get_y(&self) -> f64 {
        self.y
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getZ
    pub fn get_z(&self) -> f64 {
        self.z
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getXDist
    pub fn get_x_dist(&self) -> f32 {
        self.x_dist
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getYDist
    pub fn get_y_dist(&self) -> f32 {
        self.y_dist
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getZDist
    pub fn get_z_dist(&self) -> f32 {
        self.z_dist
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getMaxSpeed
    pub fn get_max_speed(&self) -> f32 {
        self.max_speed
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getCount
    pub fn get_count(&self) -> i32 {
        self.count
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket#getParticle
    pub fn get_particle(&self) -> &ParticleOptions {
        &self.particle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_fields_and_order() {
        let packet = ClientboundLevelParticlesPacket::new(
            ParticleOptions { type_id: 42 },
            true,
            false,
            -123.456,
            789.0,
            -0.5,
            1.25,
            -2.5,
            3.75,
            0.125,
            -100,
        );
        let mut buf = RegistryFriendlyByteBuf::writer();
        packet.write(&mut buf);
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundLevelParticlesPacket::read(&mut reader).unwrap();
        assert_eq!(decoded, packet);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn float_and_double_bits_preserved() {
        let nan_f32 = f32::from_bits(0x7FC0_1234);
        let neg_zero_f32 = f32::from_bits(0x8000_0000);
        let nan_f64 = f64::from_bits(0x7FF8_0000_0000_1234u64);
        let neg_zero_f64 = f64::from_bits(0x8000_0000_0000_0000u64);
        let packet = ClientboundLevelParticlesPacket::new(
            ParticleOptions { type_id: 1 },
            false,
            true,
            nan_f64,
            f64::NEG_INFINITY,
            neg_zero_f64,
            nan_f32,
            neg_zero_f32,
            f32::INFINITY,
            f32::from_bits(0xFFC0_0001),
            0,
        );
        let mut buf = RegistryFriendlyByteBuf::writer();
        packet.write(&mut buf);
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundLevelParticlesPacket::read(&mut reader).unwrap();
        assert!(decoded.get_x().is_nan());
        assert_eq!(decoded.get_x().to_bits(), nan_f64.to_bits());
        assert_eq!(decoded.get_y().to_bits(), f64::NEG_INFINITY.to_bits());
        assert_eq!(decoded.get_z().to_bits(), neg_zero_f64.to_bits());
        assert!(decoded.get_x_dist().is_nan());
        assert_eq!(decoded.get_x_dist().to_bits(), nan_f32.to_bits());
        assert_eq!(decoded.get_y_dist().to_bits(), neg_zero_f32.to_bits());
        assert_eq!(decoded.get_z_dist().to_bits(), f32::INFINITY.to_bits());
        assert_eq!(decoded.get_max_speed().to_bits(), 0xFFC0_0001);
    }

    #[test]
    fn integer_edge_values_roundtrip() {
        let packet = ClientboundLevelParticlesPacket::new(
            ParticleOptions { type_id: i32::MIN },
            true,
            true,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            i32::MIN,
        );
        let mut buf = RegistryFriendlyByteBuf::writer();
        packet.write(&mut buf);
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundLevelParticlesPacket::read(&mut reader).unwrap();
        assert_eq!(decoded.get_count(), i32::MIN);
        assert_eq!(decoded.get_particle().type_id, i32::MIN);

        let packet_max = ClientboundLevelParticlesPacket::new(
            ParticleOptions { type_id: i32::MAX },
            false,
            false,
            f64::MAX,
            f64::MIN,
            0.0,
            f32::MAX,
            f32::MIN,
            0.0,
            0.0,
            i32::MAX,
        );
        let mut buf = RegistryFriendlyByteBuf::writer();
        packet_max.write(&mut buf);
        let mut reader = RegistryFriendlyByteBuf::new(buf.into_inner());
        let decoded = ClientboundLevelParticlesPacket::read(&mut reader).unwrap();
        assert_eq!(decoded.get_count(), i32::MAX);
        assert_eq!(decoded.get_particle().type_id, i32::MAX);
        assert_eq!(decoded.get_x().to_bits(), f64::MAX.to_bits());
        assert_eq!(decoded.get_x_dist().to_bits(), f32::MAX.to_bits());
    }

    #[test]
    fn handle_type_and_booleans() {
        struct Listener {
            seen_count: Option<i32>,
        }
        impl ClientGamePacketListener for Listener {
            fn handle_particle_event(&mut self, packet: &ClientboundLevelParticlesPacket) {
                self.seen_count = Some(packet.get_count());
            }
        }

        let packet = ClientboundLevelParticlesPacket::new(
            ParticleOptions { type_id: 7 },
            true,
            false,
            1.0,
            2.0,
            3.0,
            4.0,
            5.0,
            6.0,
            7.0,
            8,
        );
        assert_eq!(
            packet.packet_type(),
            PacketType("minecraft:clientbound_level_particles")
        );
        assert!(packet.is_override_limiter());
        assert!(!packet.always_show());

        let mut listener = Listener { seen_count: None };
        packet.handle(&mut listener);
        assert_eq!(listener.seen_count, Some(8));
    }
}
