use bytes::{Buf, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    ClientboundExplode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    // JAVA: net.minecraft.world.phys.Vec3#<init>
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.world.phys.Vec3#STREAM_CODEC decode
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            x: buf.get_f64(),
            y: buf.get_f64(),
            z: buf.get_f64(),
        }
    }

    // JAVA: net.minecraft.world.phys.Vec3#STREAM_CODEC encode
    pub fn write(&self, buf: &mut impl BufMut) {
        buf.put_f64(self.x);
        buf.put_f64(self.y);
        buf.put_f64(self.z);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleOptions {
    pub type_id: i32,
}

impl ParticleOptions {
    // JAVA: net.minecraft.core.particles.ParticleTypes#STREAM_CODEC decode
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            type_id: read_var_int(buf),
        }
    }

    // JAVA: net.minecraft.core.particles.ParticleTypes#STREAM_CODEC encode
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.type_id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundEvent {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T> {
    pub value: T,
}

impl Holder<SoundEvent> {
    // JAVA: net.minecraft.sounds.SoundEvent#STREAM_CODEC decode
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            value: SoundEvent {
                id: read_var_int(buf),
            },
        }
    }

    // JAVA: net.minecraft.sounds.SoundEvent#STREAM_CODEC encode
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.value.id);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionParticleInfo {
    pub particle: ParticleOptions,
    pub scaling: f32,
    pub speed: f32,
}

impl ExplosionParticleInfo {
    // JAVA: net.minecraft.core.particles.ExplosionParticleInfo#STREAM_CODEC decode
    pub fn read(buf: &mut impl Buf) -> Self {
        Self {
            particle: ParticleOptions::read(buf),
            scaling: buf.get_f32(),
            speed: buf.get_f32(),
        }
    }

    // JAVA: net.minecraft.core.particles.ExplosionParticleInfo#STREAM_CODEC encode
    pub fn write(&self, buf: &mut impl BufMut) {
        self.particle.write(buf);
        buf.put_f32(self.scaling);
        buf.put_f32(self.speed);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEntry<T> {
    pub data: T,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedList<T> {
    pub entries: Vec<WeightedEntry<T>>,
}

impl WeightedList<ExplosionParticleInfo> {
    // JAVA: net.minecraft.util.random.WeightedList#streamCodec decode
    pub fn read(buf: &mut impl Buf) -> Self {
        let count = read_var_int(buf).max(0) as usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let data = ExplosionParticleInfo::read(buf);
            let weight = read_var_int(buf);
            entries.push(WeightedEntry { data, weight });
        }
        Self { entries }
    }

    // JAVA: net.minecraft.util.random.WeightedList#streamCodec encode
    pub fn write(&self, buf: &mut impl BufMut) {
        write_var_int(buf, self.entries.len() as i32);
        for entry in &self.entries {
            entry.data.write(buf);
            write_var_int(buf, entry.weight);
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#readVarInt
fn read_var_int(buf: &mut impl Buf) -> i32 {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    loop {
        let b = buf.get_u8() as u32;
        result |= (b & 0x7F) << shift;
        if (b & 0x80) == 0 {
            return result as i32;
        }
        shift = shift.wrapping_add(7);
        if shift >= 32 {
            return result as i32;
        }
    }
}

// JAVA: net.minecraft.network.FriendlyByteBuf#writeVarInt
fn write_var_int(buf: &mut impl BufMut, value: i32) {
    let mut val = value as u32;
    loop {
        if (val & !0x7F) == 0 {
            buf.put_u8(val as u8);
            return;
        }
        buf.put_u8(((val & 0x7F) as u8) | 0x80);
        val >>= 7;
    }
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#optional decode
fn read_optional_vec3(buf: &mut impl Buf) -> Option<Vec3> {
    if buf.get_u8() != 0 {
        Some(Vec3::read(buf))
    } else {
        None
    }
}

// JAVA: net.minecraft.network.codec.ByteBufCodecs#optional encode
fn write_optional_vec3(buf: &mut impl BufMut, value: &Option<Vec3>) {
    match value {
        Some(v) => {
            buf.put_u8(1);
            v.write(buf);
        }
        None => {
            buf.put_u8(0);
        }
    }
}

pub trait ClientGamePacketListener {
    // JAVA: net.minecraft.network.protocol.game.ClientGamePacketListener#handleExplosion
    fn handle_explosion(&mut self, packet: &ClientboundExplodePacket);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundExplodePacket {
    pub center: Vec3,
    pub radius: f32,
    pub block_count: i32,
    pub player_knockback: Option<Vec3>,
    pub explosion_particle: ParticleOptions,
    pub explosion_sound: Holder<SoundEvent>,
    pub block_particles: WeightedList<ExplosionParticleInfo>,
}

impl ClientboundExplodePacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#<init>
    pub fn new(
        center: Vec3,
        radius: f32,
        block_count: i32,
        player_knockback: Option<Vec3>,
        explosion_particle: ParticleOptions,
        explosion_sound: Holder<SoundEvent>,
        block_particles: WeightedList<ExplosionParticleInfo>,
    ) -> Self {
        Self {
            center,
            radius,
            block_count,
            player_knockback,
            explosion_particle,
            explosion_sound,
            block_particles,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#STREAM_CODEC decode
    pub fn read(buf: &mut impl Buf) -> Self {
        let center = Vec3::read(buf);
        let radius = buf.get_f32();
        let block_count = buf.get_i32();
        let player_knockback = read_optional_vec3(buf);
        let explosion_particle = ParticleOptions::read(buf);
        let explosion_sound = Holder::<SoundEvent>::read(buf);
        let block_particles = WeightedList::<ExplosionParticleInfo>::read(buf);
        Self::new(
            center,
            radius,
            block_count,
            player_knockback,
            explosion_particle,
            explosion_sound,
            block_particles,
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#STREAM_CODEC encode
    pub fn write(&self, buf: &mut impl BufMut) {
        self.center.write(buf);
        buf.put_f32(self.radius);
        buf.put_i32(self.block_count);
        write_optional_vec3(buf, &self.player_knockback);
        self.explosion_particle.write(buf);
        self.explosion_sound.write(buf);
        self.block_particles.write(buf);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#type
    pub fn packet_type(&self) -> PacketType {
        PacketType::ClientboundExplode
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#handle
    pub fn handle(&self, listener: &mut impl ClientGamePacketListener) {
        listener.handle_explosion(self);
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#center
    pub fn center(&self) -> Vec3 {
        self.center
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#radius
    pub fn radius(&self) -> f32 {
        self.radius
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#blockCount
    pub fn block_count(&self) -> i32 {
        self.block_count
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#playerKnockback
    pub fn player_knockback(&self) -> Option<Vec3> {
        self.player_knockback
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#explosionParticle
    pub fn explosion_particle(&self) -> &ParticleOptions {
        &self.explosion_particle
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#explosionSound
    pub fn explosion_sound(&self) -> &Holder<SoundEvent> {
        &self.explosion_sound
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundExplodePacket#blockParticles
    pub fn block_particles(&self) -> &WeightedList<ExplosionParticleInfo> {
        &self.block_particles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    struct TestListener {
        called: bool,
        radius: f32,
    }

    impl ClientGamePacketListener for TestListener {
        fn handle_explosion(&mut self, packet: &ClientboundExplodePacket) {
            self.called = true;
            self.radius = packet.radius();
        }
    }

    fn sample_packet(knockback: Option<Vec3>) -> ClientboundExplodePacket {
        ClientboundExplodePacket::new(
            Vec3::new(1.5, -2.0, 3.25),
            4.0,
            42,
            knockback,
            ParticleOptions { type_id: 7 },
            Holder {
                value: SoundEvent { id: 3 },
            },
            WeightedList {
                entries: vec![WeightedEntry {
                    data: ExplosionParticleInfo {
                        particle: ParticleOptions { type_id: 8 },
                        scaling: 1.0,
                        speed: 0.5,
                    },
                    weight: 10,
                }],
            },
        )
    }

    #[test]
    fn round_trip_with_knockback() {
        let packet = sample_packet(Some(Vec3::new(-0.5, 1.0, 0.25)));
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundExplodePacket::read(&mut reader);
        assert_eq!(decoded, packet);
    }

    #[test]
    fn round_trip_without_knockback_and_negative_values() {
        let packet = sample_packet(None);
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundExplodePacket::read(&mut reader);
        assert_eq!(decoded.player_knockback(), None);
        assert_eq!(decoded.center().x.to_bits(), 1.5f64.to_bits());
        assert_eq!(decoded.center().y.to_bits(), (-2.0f64).to_bits());
    }

    #[test]
    fn float_bits_preserved() {
        let packet = ClientboundExplodePacket::new(
            Vec3::new(0.0, 0.0, 0.0),
            f32::from_bits(0x7FC0_0001),
            -1,
            None,
            ParticleOptions { type_id: 0 },
            Holder {
                value: SoundEvent { id: 0 },
            },
            WeightedList { entries: vec![] },
        );
        let mut buf = BytesMut::new();
        packet.write(&mut buf);
        let mut reader = buf.freeze();
        let decoded = ClientboundExplodePacket::read(&mut reader);
        assert_eq!(decoded.radius().to_bits(), 0x7FC0_0001);
        assert_eq!(decoded.block_count(), -1);
    }

    #[test]
    fn handle_calls_listener() {
        let packet = sample_packet(None);
        let mut listener = TestListener {
            called: false,
            radius: 0.0,
        };
        packet.handle(&mut listener);
        assert!(listener.called);
        assert_eq!(listener.radius.to_bits(), 4.0f32.to_bits());
        assert_eq!(packet.packet_type(), PacketType::ClientboundExplode);
    }
}
